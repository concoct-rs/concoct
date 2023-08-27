use super::View;
use crate::Context;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::Element;

pub trait Attribute<E> {
    type State;

    fn build(self, cx: &mut Context<E>, elem: &mut Element) -> Self::State;

    fn rebuild(self, cx: &mut Context<E>, elem: &mut Element, state: &mut Self::State);
}

pub fn on<F>(name: &str, make: F) -> OnAttr<F> {
    OnAttr { name, make }
}

pub struct OnAttr<'a, F> {
    name: &'a str,
    make: F,
}

impl<'a, F, E> Attribute<E> for OnAttr<'a, F>
where
    F: Fn() -> E + 'static,
    E: 'static,
{
    type State = (&'a str, Closure<dyn FnMut()>);

    fn build(self, cx: &mut Context<E>, elem: &mut Element) -> Self::State {
        let update = cx.update.clone();

        let f: Closure<dyn FnMut()> = Closure::new(move || {
            update.borrow_mut().as_mut().unwrap()((self.make)());
        });
        elem.add_event_listener_with_callback(self.name, f.as_ref().unchecked_ref())
            .unwrap();
        (self.name, f)
    }

    fn rebuild(self, cx: &mut Context<E>, elem: &mut Element, state: &mut Self::State) {}
}

impl<E> Attribute<E> for () {
    type State = ();

    fn build(self, cx: &mut Context<E>, elem: &mut Element) -> Self::State {}

    fn rebuild(self, cx: &mut Context<E>, elem: &mut Element, state: &mut Self::State) {}
}

macro_rules! html_tags {
    ($($tag:ident),+) => {
        $(
            pub fn $tag<V>(child: V) -> Html<'static, (), V> {
                Html::new(stringify!($tag), (), child)
            }
        )+
    };
}

html_tags!(
    a, abbr, address, area, article, aside, audio, b, base, bdi, bdo, blockquote, body, br, button,
    canvas, caption, cite, code, col, colgroup, data, datalist, dd, del, details, dfn, dialog, div,
    dl, dt, em, embed, fieldset, figcaption, figure, footer, form, h1, h2, h3, h4, h5, h6, head,
    header, hgroup, hr, html, i, iframe, img, input, ins, kbd, label, legend, li, link, main, map,
    mark, meta, meter, nav, noscript, object, ol, optgroup, option, output, p, param, picture, pre,
    progress, q, rp, rt, ruby, s, samp, script, section, select, small, source, span, strong,
    style, sub, summary, sup, table, tbody, td, template, textarea, tfoot, th, thead, time, title,
    tr, track, u, ul, var, video, wbr
);

pub struct Html<'a, A, V> {
    tag: &'a str,
    attributes: A,
    child: V,
}

impl<'a, A, V> Html<'a, A, V> {
    pub fn new(tag: &'a str, attributes: A, child: V) -> Self {
        Self {
            tag,
            attributes,
            child,
        }
    }

    pub fn modify<A2>(self, attributes: A2) -> Html<'a, A2, V> {
        Html::new(self.tag, attributes, self.child)
    }

    pub fn child<V2>(self, child: V2) -> Html<'a, A, V2> {
        Html::new(self.tag, self.attributes, child)
    }
}

impl<'a, A, V, E> View<E> for Html<'a, A, V>
where
    A: Attribute<E>,
    V: View<E>,
    E: 'static,
{
    type State = (A::State, Element, V::State);

    fn build(self, cx: &mut Context<E>) -> Self::State {
        let mut elem = cx.document.create_element(self.tag).unwrap();
        cx.insert(&elem);

        let attrs = self.attributes.build(cx, &mut elem);

        cx.stack.push((elem, 0));
        let state = self.child.build(cx);
        let (elem, _) = cx.stack.pop().unwrap();

        (attrs, elem, state)
    }

    fn rebuild(self, cx: &mut Context<E>, state: &mut Self::State) {
        self.attributes.rebuild(cx, &mut state.1, &mut state.0);

        cx.skip();
        cx.stack.push((state.1.clone(), 0));
        self.child.rebuild(cx, &mut state.2);
        cx.stack.pop();
    }

    fn remove(_cx: &mut Context<E>, state: &mut Self::State) {
        state.1.remove();
    }
}
