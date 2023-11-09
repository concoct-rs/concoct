use crate::{
    runtime::Runtime, use_context, use_context_provider, use_hook, InputEvent, MouseEvent, View,
};
use std::{borrow::Cow, cell::RefCell, collections::HashMap, rc::Rc};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{window, Element, Event};

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
                self.on_event($name, move |event| {
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
    attributes: HashMap<Cow<'static, str>, Cow<'static, str>>,
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
            attributes: HashMap::new(),
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

    pub fn attr(
        mut self,
        name: impl Into<Cow<'static, str>>,
        value: impl Into<Cow<'static, str>>,
    ) -> Self {
        self.attributes.insert(name.into(), value.into());
        self
    }
}

impl View for Html {
    fn view(&mut self) {
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

        let callbacks = use_hook(|| {
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

        let mut attrs = use_hook(|| HashMap::new());
        for (name, value) in self.attributes.iter() {
            if let Some(last_value) = attrs.get_mut(name) {
                if value != last_value {
                    *last_value = value.clone();
                    elem.set_attribute(name, value).unwrap();
                }
            } else {
                attrs.insert(name.clone(), value.clone());
                elem.set_attribute(name, value).unwrap();
            }
        }

        let mut child = use_hook(|| None);
        if let Some(view) = self.view.take() {
            *child = Some(Runtime::current().spawn(view));
        } else {
            *child = None;
        }
    }
}
