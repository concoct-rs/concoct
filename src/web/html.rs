use super::{Data, WebContext};
use crate::{
    body::Child,
    hook::{use_context, use_provider, use_ref},
    Body, View,
};
use std::{borrow::Cow, cell::RefCell, rc::Rc};
use web_sys::{
    wasm_bindgen::{closure::Closure, JsCast},
    Event,
};

macro_rules! make_tag_fns {
    ($($name:tt),*) => {
        $(
            pub fn $name<C: Body>(child: C) -> Html<C> {
                Html::new(stringify!($name), child)
            }
        )*
    };
}

make_tag_fns!(
    a, abbr, address, area, article, aside, audio, b, base, bdi, bdo, blockquote, body, br, button,
    canvas, caption, cite, code, col, colgroup, data, datalist, dd, del, details, dfn, dialog, div,
    dl, dt, em, embed, fieldset, figcaption, figure, footer, form, h1, h2, h3, h4, h5, h6, head,
    header, hr, html, i, iframe, img, input, ins, kbd, label, legend, li, link, main, map, mark,
    meta, meter, nav, noscript, object, ol, optgroup, option, output, p, param, picture, pre,
    progress, q, rp, rt, ruby, s, samp, script, section, select, small, source, span, strong, sub,
    summary, sup, svg, table, tbody, td, template, textarea, tfoot, th, thead, time, title, tr,
    track, u, ul, var, video, wbr
);

pub struct Html<C> {
    tag: Cow<'static, str>,
    handlers: Vec<(Cow<'static, str>, Rc<RefCell<dyn FnMut(Event)>>)>,
    child: Child<C>,
}

impl<C> Html<C> {
    pub fn new(tag: impl Into<Cow<'static, str>>, child: C) -> Self {
        Self {
            tag: tag.into(),
            handlers: Vec::new(),
            child: Child::new(child),
        }
    }

    pub fn on_click(mut self, handler: impl FnMut(Event) + 'static) -> Self {
        self.handlers
            .push((Cow::Borrowed("click"), Rc::new(RefCell::new(handler))));
        self
    }
}

impl<C: Body> View for Html<C> {
    fn body(&self) -> impl Body {
        let data = use_ref(|| RefCell::new(Data::default()));
        let mut data_ref = data.borrow_mut();

        let web_cx = use_context::<WebContext>().unwrap();

        if data_ref.element.is_none() {
            let elem = web_cx.document.create_element(&self.tag).unwrap();
            web_cx.parent.append_child(&elem).unwrap();

            for (name, handler) in &self.handlers {
                let handler = Rc::new(RefCell::new(handler.clone()));
                let handler_clone = handler.clone();
                let callback: Closure<dyn FnMut(Event)> = Closure::wrap(Box::new(move |event| {
                    handler.borrow_mut().borrow_mut()(event)
                }));
                elem.add_event_listener_with_callback(&name, callback.as_ref().unchecked_ref())
                    .unwrap();
                data_ref.callbacks.push((callback, handler_clone));
            }

            data_ref.element = Some(elem);
        } else {
            for ((_name, handler), (_callback, cell)) in
                self.handlers.iter().zip(&data_ref.callbacks)
            {
                *cell.borrow_mut() = handler.clone();
            }
        }

        use_provider(WebContext {
            window: web_cx.window.clone(),
            document: web_cx.document.clone(),
            parent: data_ref.element.as_ref().unwrap().clone().into(),
        });

        self.child.clone()
    }
}
