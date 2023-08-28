use super::{Context, Web};
use crate::view::Context as _;
use crate::{view::View, Modify};
use web_sys::Element;

/// Html element view.
pub struct Html<'a, A, V> {
    tag: &'a str,
    modify: A,
    view: V,
}

impl<'a, A, V> Html<'a, A, V> {
    pub fn new(tag: &'a str, modify: A, view: V) -> Self {
        Self { tag, modify, view }
    }

    pub fn modify<A2>(self, attributes: A2) -> Html<'a, A2, V> {
        Html::new(self.tag, attributes, self.view)
    }

    pub fn then<V2>(self, view: V2) -> Html<'a, A, (V, V2)> {
        Html::new(self.tag, self.modify, (self.view, view))
    }
}

macro_rules! html_tags {
    ($($tag:ident),+) => {
        $(
            pub fn $tag(attrs: A, view: V) -> Self {
                Html::new(stringify!($tag), attrs, view)
            }
        )+
    };
}

impl<'a, A, V> Html<'static, A, V> {
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

impl<'a, A, V, E> View<Web<E>> for Html<'a, A, V>
where
    A: Modify<Web<E>, Element>,
    V: View<Web<E>>,
    E: 'static,
{
    type State = (A::State, Element, V::State);

    fn build(self, cx: &mut Context<E>) -> Self::State {
        let mut elem = cx.document.create_element(self.tag).unwrap();
        cx.insert(&elem);

        let attrs = self.modify.build(cx, &mut elem);
        let (elem, _, state) = cx.with_nested(elem, |cx| self.view.build(cx));
        (attrs, elem, state)
    }

    fn rebuild(self, cx: &mut Context<E>, state: &mut Self::State) {
        self.modify.rebuild(cx, &mut state.1, &mut state.0);

        cx.skip();
        cx.with_nested(state.1.clone(), |cx| {
            self.view.rebuild(cx, &mut state.2);
        });
    }

    fn remove(_cx: &mut Context<E>, state: &mut Self::State) {
        state.1.remove();
    }
}
