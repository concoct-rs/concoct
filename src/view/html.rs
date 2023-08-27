use super::View;
use crate::Context;
use impl_trait_for_tuples::impl_for_tuples;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{Element, Event, HtmlInputElement, KeyboardEvent};

pub trait Attribute<E> {
    type State;

    fn build(self, cx: &mut Context<E>, elem: &mut Element) -> Self::State;

    fn rebuild(self, cx: &mut Context<E>, elem: &mut Element, state: &mut Self::State);
}

#[impl_for_tuples(16)]
impl<E> Attribute<E> for Tuple {
    for_tuples!( type State = ( #( Tuple::State ),* ); );

    fn build(self, cx: &mut Context<E>, elem: &mut Element) -> Self::State {
        for_tuples!( (#( self.Tuple.build(cx, elem) ),*) )
    }

    fn rebuild(self, cx: &mut Context<E>, elem: &mut Element, state: &mut Self::State) {
        for_tuples!( #( self.Tuple.rebuild(cx, elem, &mut state.Tuple); )* )
    }
}

pub fn value(value: String) -> ValueAttr {
    ValueAttr { value }
}

pub struct ValueAttr {
    value: String,
}

impl<E> Attribute<E> for ValueAttr {
    type State = ();

    fn build(self, cx: &mut Context<E>, elem: &mut Element) -> Self::State {
        elem.unchecked_ref::<HtmlInputElement>()
            .set_value(&self.value);
    }

    fn rebuild(self, cx: &mut Context<E>, elem: &mut Element, state: &mut Self::State) {
        elem.unchecked_ref::<HtmlInputElement>()
            .set_value(&self.value);
    }
}

pub fn event_target_value(event: &Event) -> String {
    event
        .target()
        .unwrap()
        .unchecked_into::<web_sys::HtmlInputElement>()
        .value()
}

pub fn event_key_code(event: &Event) -> u32 {
    event.unchecked_ref::<KeyboardEvent>().key_code()
}

pub fn on<F, E>(name: &str, make: F) -> OnAttr<F>
where
    F: Fn(Event) -> E + 'static,
    E: 'static,
{
    OnAttr { name, make }
}

pub struct OnAttr<'a, F> {
    name: &'a str,
    make: F,
}

impl<'a, F, E> Attribute<E> for OnAttr<'a, F>
where
    F: Fn(Event) -> E + 'static,
    E: 'static,
{
    type State = (&'a str, Closure<dyn FnMut(Event)>);

    fn build(self, cx: &mut Context<E>, elem: &mut Element) -> Self::State {
        let update = cx.update.clone();

        let f: Closure<dyn FnMut(Event)> = Closure::new(move |event| {
            update.borrow_mut().as_mut().unwrap()((self.make)(event));
        });
        elem.add_event_listener_with_callback(self.name, f.as_ref().unchecked_ref())
            .unwrap();
        (self.name, f)
    }

    fn rebuild(self, cx: &mut Context<E>, elem: &mut Element, state: &mut Self::State) {}
}

macro_rules! html_tags {
    ($($tag:ident),+) => {
        $(
            pub fn $tag() -> Html<'static, (), ()> {
                Html::new(stringify!($tag), (), ())
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
    view: V,
}

impl<'a, A, V> Html<'a, A, V> {
    pub fn new(tag: &'a str, attributes: A, view: V) -> Self {
        Self {
            tag,
            attributes,
            view,
        }
    }

    pub fn modify<A2>(self, attributes: A2) -> Html<'a, A2, V> {
        Html::new(self.tag, attributes, self.view)
    }

    pub fn then<V2>(self, view: V2) -> Html<'a, A, (V, V2)> {
        Html::new(self.tag, self.attributes, (self.view, view))
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
        let state = self.view.build(cx);
        let (elem, _) = cx.stack.pop().unwrap();

        (attrs, elem, state)
    }

    fn rebuild(self, cx: &mut Context<E>, state: &mut Self::State) {
        self.attributes.rebuild(cx, &mut state.1, &mut state.0);

        cx.skip();
        cx.stack.push((state.1.clone(), 0));
        self.view.rebuild(cx, &mut state.2);
        cx.stack.pop();
    }

    fn remove(_cx: &mut Context<E>, state: &mut Self::State) {
        state.1.remove();
    }
}
