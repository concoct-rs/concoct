use concoct::{
    hook::{use_context, use_provider, use_ref},
     IntoAction, View,
};
use std::{borrow::Cow, cell::RefCell, rc::Rc};
use web_sys::{
    wasm_bindgen::{closure::Closure, JsCast},
    Element, Event,
};

use crate::WebContext;

macro_rules! make_tag_fns {
    ($($name:tt),*) => {
        $(
            pub fn $name<T, A, C: View<T, A>>(content: C) -> Html<C, T, A> {
                Html::new(stringify!($name), content)
            }
        )*
    };
}

make_tag_fns!(
    a, abbr, address, area, article, aside, audio, b, base, bdi, bdo, blockquote, body, br, button,
    canvas, caption, cite, code, col, colgroup, data, datalist, dd, del, details, dfn, dialog, div,
    dl, dt, em, embed, fieldset, figcaption, figure, footer, form, h1, h2, h3, h4, h5, h6, head,
    header, hr, i, iframe, img, input, ins, kbd, label, legend, li, link, main, map, mark, meta,
    meter, nav, noscript, object, ol, optgroup, option, output, p, param, picture, pre, progress,
    q, rp, rt, ruby, s, samp, script, section, select, small, source, span, strong, sub, summary,
    sup, svg, table, tbody, td, template, textarea, tfoot, th, thead, time, title, tr, track, u,
    ul, var, video, wbr
);

struct Data<T, A> {
    element: Option<Element>,
    callbacks: Vec<(
        Closure<dyn FnMut(Event)>,
        Rc<RefCell<Rc<RefCell<dyn FnMut(&mut T, Event) -> Option<A>>>>>,
    )>,
}

pub struct Html<C, T, A> {
    tag: Cow<'static, str>,
    attrs: Vec<(Cow<'static, str>, Cow<'static, str>)>,
    handlers: Vec<(
        Cow<'static, str>,
        Rc<RefCell<dyn FnMut(&mut T, Event) -> Option<A>>>,
    )>,
    content: C,
}

macro_rules! impl_attr_methods {
    ($($fn_name: tt: $name: tt),*) => {
        $(
            pub fn $fn_name(self, value: impl Into<Cow<'static, str>>,) -> Self {
                self.attr($name, value)
            }
        )*
    };
}

macro_rules! impl_handler_methods {
    ($($fn_name: tt: $name: tt),*) => {
        $(
            pub fn $fn_name<R: IntoAction<A>>(self, handler: impl FnMut(&mut T, Event) -> R + 'static) -> Self {
                self.handler($name, handler)
            }
        )*
    };
}

impl<C, T, A> Html<C, T, A> {
    pub fn new(tag: impl Into<Cow<'static, str>>, content: C) -> Html<C, T, A>
    where
        C: View<T, A>,
    {
        Html {
            tag: tag.into(),
            attrs: Vec::new(),
            handlers: Vec::new(),
            content,
        }
    }

    pub fn attr(
        mut self,
        name: impl Into<Cow<'static, str>>,
        value: impl Into<Cow<'static, str>>,
    ) -> Self {
        self.attrs.push((name.into(), value.into()));
        self
    }

    pub fn handler<R: IntoAction<A>>(
        mut self,
        name: impl Into<Cow<'static, str>>,
        mut handler: impl FnMut(&mut T, Event) -> R + 'static,
    ) -> Self {
        self.handlers.push((
            name.into(),
            Rc::new(RefCell::new(move |state: &mut T, event| {
                handler(state, event).into_action()
            })),
        ));
        self
    }

    impl_attr_methods!(
        class: "class",
        kind: "type"
    );

    impl_handler_methods!(
        on_click: "click",
        on_input: "input",
        on_submit: "submit"
    );
}

impl<T, A, C> View<T, A> for Html<C, T, A>
where
    T: 'static,
    A: 'static,
    C: View<T, A>,
{
    fn body(&mut self, cx: &concoct::Scope<T, A>) -> impl View<T, A> {
        let data = use_ref(cx, || {
            Rc::new(RefCell::new(Data {
                element: Default::default(),
                callbacks: Default::default(),
            }))
        });
        let mut data_ref = data.borrow_mut();

        let web_cx: Rc<WebContext> = use_context(cx);
        let _data_clone = data.clone();

        /*
         use_on_drop(move || {
            if let Some(element) = &data_clone.borrow_mut().element {
                element.remove();
            }
        });
         */

        if data_ref.element.is_none() {
            let elem = web_cx.document.create_element(&self.tag).unwrap();
            web_cx.parent.append_child(&elem).unwrap();

            for (name, value) in &self.attrs {
                elem.set_attribute(name, value).unwrap();
            }

            for (name, handler) in &self.handlers {
                let handler_cell = Rc::new(RefCell::new(handler.clone()));
                let handler_cell_clone = handler_cell.clone();

                let handle = cx.handle();
                let callback: Closure<dyn FnMut(Event)> = Closure::wrap(Box::new(move |event| {
                    let handler_cell_clone = handler_cell.clone();
                    handle.update(Rc::new(move |state| {
                        handler_cell_clone.borrow().borrow_mut()(state, event.clone())
                    }))
                }));
                elem.add_event_listener_with_callback(&name, callback.as_ref().unchecked_ref())
                    .unwrap();

                data_ref.callbacks.push((callback, handler_cell_clone));
            }

            data_ref.element = Some(elem);
        } else {
            for ((_name, handler), (_callback, cell)) in
                self.handlers.iter().zip(&data_ref.callbacks)
            {
                *cell.borrow_mut() = handler.clone();
            }
        }

        use_provider(cx, || WebContext {
            window: web_cx.window.clone(),
            document: web_cx.document.clone(),
            body: web_cx.body.clone(),
            parent: data_ref.element.as_ref().unwrap().clone().unchecked_into(),
        });

        &mut self.content
    }
}
