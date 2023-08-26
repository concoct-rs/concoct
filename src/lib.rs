// #[cfg(feature = "gl")]
// mod renderer;
// #[cfg(feature = "gl")]
// pub use renderer::{Event, Renderer};

// pub mod view;

use std::num::NonZeroU64;

pub struct Id(NonZeroU64);

pub struct BuildContext {
    next_id: NonZeroU64,
    unused_ids: Vec<Id>,
}

impl Default for BuildContext {
    fn default() -> Self {
        Self {
            next_id: NonZeroU64::MIN,
            unused_ids: Vec::new(),
        }
    }
}

impl BuildContext {
    pub fn insert(&mut self) -> Id {
        self.unused_ids.pop().unwrap_or_else(|| {
            let id = Id(self.next_id);
            self.next_id = self.next_id.checked_add(1).unwrap();
            id
        })
    }

    pub fn remove(&mut self, id: Id) {
        self.unused_ids.push(id);
    }
}

pub trait View {
    type State;

    type Element: Element;

    fn build(&self, cx: &mut BuildContext) -> (Id, Self::State, Self::Element);
}

pub trait Element {
    fn build(&self, cx: &mut ElementContext);
}

use web_sys::{Document, HtmlElement, Node};

pub struct Html<'a, V> {
    tag: &'a str,
    child: V,
}

impl<'a, V> Html<'a, V> {
    pub fn new(tag: &'a str, child: V) -> Self {
        Self { tag, child }
    }
}

impl<'a, V: View> View for Html<'a, V> {
    type State = V::State;

    type Element = DomElement<'a, V::Element>;

    fn build(&self, cx: &mut BuildContext) -> (Id, Self::State, Self::Element) {
        let id = cx.insert();

        let (child_id, child_state, child_elem) = self.child.build(cx);

        let elem = DomElement {
            tag: self.tag,
            child: child_elem,
            child_id,
        };

        (id, child_state, elem)
    }
}

impl<'a> View for &'a str {
    type State = ();

    type Element = TextElement<'a>;

    fn build(&self, cx: &mut BuildContext) -> (Id, Self::State, Self::Element) {
        let id = cx.insert();
        let elem = TextElement { content: self };
        (id, (), elem)
    }
}

pub struct TextElement<'a> {
    content: &'a str,
}

impl Element for TextElement<'_> {
    fn build(&self, cx: &mut ElementContext) {
        let elem = cx.document.create_text_node(self.content);
        cx.stack.last_mut().unwrap().append_child(&elem).unwrap();
    }
}

pub struct ElementContext {
    document: Document,
    stack: Vec<web_sys::Element>,
}

impl ElementContext {
    pub fn new() -> Self {
        let window = web_sys::window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");
        let body = document.body().expect("HTML document missing body");

        Self {
            document,

            stack: vec![body.into()],
        }
    }
}

pub struct DomElement<'a, C> {
    tag: &'a str,
    child: C,
    child_id: Id,
}

impl<'a, C> Element for DomElement<'a, C>
where
    C: Element,
{
    fn build(&self, cx: &mut ElementContext) {
        let elem = cx.document.create_element(self.tag).unwrap();
        cx.stack.last_mut().unwrap().append_child(&elem).unwrap();

        cx.stack.push(elem);
        self.child.build(cx);
        cx.stack.pop();
    }
}

pub struct App {
    build_cx: BuildContext,
    element_cx: ElementContext,
}

impl App {
    pub fn new() -> Self {
        Self {
            build_cx: BuildContext::default(),
            element_cx: ElementContext::new(),
        }
    }

    pub fn run(&mut self, view: impl View) {
        let (_id, _state, elem) = view.build(&mut self.build_cx);
        elem.build(&mut self.element_cx);
    }
}
