use crate::{runtime::Runtime, use_context, use_context_provider, Node, Scope, View};

use std::{borrow::Cow, cell::RefCell, rc::Rc};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{window, Element};

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
}

#[derive(Clone)]
pub struct Html {
    tag: Cow<'static, str>,
    view: Option<Rc<RefCell<dyn View>>>,
    on_click: Option<Rc<RefCell<dyn FnMut()>>>,
    callback: Option<Rc<Closure<dyn FnMut()>>>,
}

impl Html {
    pub fn new(tag: impl Into<Cow<'static, str>>) -> Self {
        Self {
            tag: tag.into(),
            view: None,
            on_click: None,
            callback: None,
        }
    }

    pub fn view(mut self, view: impl View + 'static) -> Self {
        self.view = Some(Rc::new(RefCell::new(view)));
        self
    }

    pub fn on_click(mut self, f: impl FnMut() + 'static) -> Self {
        self.on_click = Some(Rc::new(RefCell::new(f)));
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
        let callback = scope
            .use_hook(|| {
                if let Some(f) = self.on_click.take() {
                    let closure: Closure<dyn FnMut()> =
                        Closure::wrap(Box::new(move || f.borrow_mut()()));
                    Some(Rc::new(closure))
                } else {
                    None
                }
            })
            .clone();
        drop(scope);

        let elem = use_context_provider(|| {
            let elem = window()
                .unwrap()
                .document()
                .unwrap()
                .create_element(&self.tag)
                .unwrap();

            if let Some(f) = callback {
                elem.add_event_listener_with_callback("click", f.as_ref().as_ref().unchecked_ref())
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
