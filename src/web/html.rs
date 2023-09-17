use super::{
    attr::{attr, Attr},
    class, on, value, On, Value, Web,
};
use crate::{view::View, Modify, Platform};
use std::borrow::Cow;
use web_sys::{Element, Event};

/// State for the [`Html`] view.
pub struct State<M, V> {
    element: Element,
    modify: M,
    view: V,
}

/// Html element view.
pub struct Html<A, V> {
    tag: Cow<'static, str>,
    modify: A,
    view: V,
}

impl Html<(), ()> {
    pub fn new(tag: impl Into<Cow<'static, str>>) -> Self {
        Self {
            tag: tag.into(),
            modify: (),
            view: (),
        }
    }
}

impl<A, V> Html<A, V> {
    pub fn modify<A2>(self, modify: A2) -> Html<(A, A2), V> {
        Html {
            tag: self.tag,
            modify: (self.modify, modify),
            view: self.view,
        }
    }

    pub fn on<F, E>(self, name: impl Into<Cow<'static, str>>, handler: F) -> Html<(A, On<F>), V>
    where
        F: Fn(Event) -> E + 'static,
        E: 'static,
    {
        self.modify(on(name, handler))
    }

    pub fn attr(
        self,
        name: impl Into<Cow<'static, str>>,
        value: impl Into<Cow<'static, str>>,
    ) -> Html<(A, Attr), V> {
        self.modify(attr(name, value))
    }

    pub fn class(self, value: impl Into<Cow<'static, str>>) -> Html<(A, Attr), V> {
        self.modify(class(value))
    }

    pub fn value(self, value: impl Into<Cow<'static, str>>) -> Html<(A, Value), V> {
        self.modify(value::value(value))
    }

    pub fn view<V2>(self, view: V2) -> Html<A, (V, V2)> {
        Html {
            tag: self.tag,
            modify: self.modify,
            view: (self.view, view),
        }
    }
}

macro_rules! html_tags {
    ($($tag:ident),+) => {
        $(
            pub fn $tag() -> Self {
                Html::new(stringify!($tag))
            }
        )+
    };
}

impl Html<(), ()> {
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

impl<A, V, E> View<Web<E>> for Html<A, V>
where
    A: Modify<Web<E>, Element>,
    V: View<Web<E>>,
    E: 'static,
{
    type State = State<A::State, V::State>;

    fn build(self, cx: &mut Web<E>) -> Self::State {
        let mut element = cx.document.create_element(&self.tag).unwrap();
        cx.insert(&element);

        let modify = self.modify.build(cx, &mut element);
        let (element, _, view) = cx.with_nested(element, |cx| self.view.build(cx));

        State {
            element,
            modify,
            view,
        }
    }

    fn rebuild(self, cx: &mut Web<E>, state: &mut Self::State) {
        self.modify
            .rebuild(cx, &mut state.element, &mut state.modify);

        cx.advance();
        cx.with_nested(state.element.clone(), |cx| {
            self.view.rebuild(cx, &mut state.view);
        });
    }

    fn remove(cx: &mut Web<E>, state: &mut Self::State) {
        // Remove the child view
        V::remove(cx, &mut state.view);

        // Remove the html element
        state.element.remove();
    }
}
