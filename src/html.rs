use crate::{runtime::Runtime, use_context, use_context_provider, Node, Scope, View, MouseEvent, InputEvent};
use std::{borrow::Cow, cell::RefCell, collections::HashMap, rc::Rc, ops::{DerefMut, Deref}};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{window, Element, Event, HtmlInputElement};

pub struct Parent(pub Element);

macro_rules! html_tags {
    ($($tag:ident),+) => {
        $(
            pub fn $tag() -> Self {
                Html::new(stringify!($tag))
            }
        )+
    };
}

macro_rules! handlers {
    ($(($fn_name:ident, $name:expr, $event:ident)),+) => {
        $(
            pub fn $fn_name(self, mut handler: impl FnMut($event) + 'static) -> Self {
                self.on_event("input", move |event| {
                    handler($event::from(event));
                })
             }
        )+
    };
}

#[derive(Clone)]
pub struct Html {
    tag: Cow<'static, str>,
    view: Option<Rc<RefCell<dyn View>>>,
    event_handlers: HashMap<Cow<'static, str>, Rc<RefCell<dyn FnMut(Event)>>>,
}

impl Html {
    html_tags!(
        a, abbr, address, area, article, aside, audio, b, base, bdi, bdo, blockquote, body, br,
        button, canvas, caption, cite, code, col, colgroup, data, datalist, dd, del, details, dfn,
        dialog, div, dl, dt, em, embed, fieldset, figcaption, figure, footer, form, h1, h2, h3, h4,
        h5, h6, head, header, hgroup, hr, html, i, iframe, img, input, ins, kbd, label, legend, li,
        link, main, map, mark, meta, meter, nav, noscript, object, ol, optgroup, option, output, p,
        param, picture, pre, progress, q, rp, rt, ruby, s, samp, script, section, select, small,
        source, span, strong, style, sub, summary, sup, table, tbody, td, template, textarea,
        tfoot, th, thead, time, title, tr, track, u, ul, var, video, wbr
    );

    pub fn new(tag: impl Into<Cow<'static, str>>) -> Self {
        Self {
            tag: tag.into(),
            view: None,
            event_handlers: HashMap::new(),
        }
    }

    pub fn view(mut self, view: impl View + 'static) -> Self {
        self.view = Some(Rc::new(RefCell::new(view)));
        self
    }

    handlers!(
        (on_click, "click", MouseEvent),
        (on_input, "input", InputEvent)
    );

    pub fn on_event(
        mut self,
        name: impl Into<Cow<'static, str>>,
        handler: impl FnMut(Event) + 'static,
    ) -> Self {
        self.event_handlers
            .insert(name.into(), Rc::new(RefCell::new(handler)));
        self
    }
}

impl View for Html {
    fn view(&mut self) -> Option<Node> {
        let parent = use_context::<Parent>()
            .map(|cx| cx.0.clone())
            .unwrap_or_else(|| {
                window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .body()
                    .unwrap()
                    .unchecked_into()
            });

        let scope = Scope::current();
        let callbacks = scope.use_hook(|| {
            self.event_handlers
                .iter()
                .map(|(name, handler)| {
                    let handler = handler.clone();
                    let closure: Closure<dyn FnMut(Event)> =
                        Closure::wrap(Box::new(move |event| handler.borrow_mut()(event)));
                    (name.clone(), Rc::new(closure))
                })
                .collect::<Vec<_>>()
        });

        let elem = use_context_provider(|| {
            let elem = window()
                .unwrap()
                .document()
                .unwrap()
                .create_element(&self.tag)
                .unwrap();

            for (name, handler) in callbacks.iter() {
                elem.add_event_listener_with_callback(
                    name,
                    handler.as_ref().as_ref().unchecked_ref(),
                )
                .unwrap();
            }

            parent.append_child(&elem).unwrap();
            Parent(elem)
        })
        .0
        .clone();

        if let Some(view) = self.view.take() {
            Runtime::current().spawn(view)
        }

        Some(Node::Element(elem))
    }

    fn child(&mut self) -> Option<Rc<RefCell<Box<dyn View>>>> {
        todo!()
    }

    fn remove(&mut self) {
        todo!()
    }
}
