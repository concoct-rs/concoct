use super::Web;
use crate::{view::View, Modify, Platform};
use web_sys::Element;

/// State for the [`Html`] view.
pub struct State<M, V> {
    element: Element,
    modify: M,
    view: V,
}

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
    type State = State<A::State, V::State>;

    fn build(self, cx: &mut Web<E>) -> Self::State {
        let mut element = cx.document.create_element(self.tag).unwrap();
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
