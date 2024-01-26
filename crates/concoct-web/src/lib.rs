use concoct::{
    hook::{use_context, use_on_drop, use_provider, use_ref},
    view::TextContext,
    Scope, View,
};
use rustc_hash::FxHasher;
use std::{
    cell::Cell,
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
    rc::Rc,
};
use web_sys::{Document, HtmlElement, Window};

pub mod html;

struct WebContext {
    window: Window,
    document: Document,
    body: HtmlElement,
    parent: HtmlElement,
}

pub struct HtmlRoot<C> {
    content: C,
}

impl<T: 'static, A: 'static, C: View<T, A>> View<T, A> for HtmlRoot<C> {
    fn body(&mut self, cx: &Scope<T, A>) -> impl View<T, A> {
        use_provider(cx, || {
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            let body = document.body().unwrap();
            WebContext {
                window,
                document,
                body: body.clone(),
                parent: body,
            }
        });

        use_provider(cx, || {
            TextContext::new(|cx: &Scope<T, A>, s| {
                let web_cx: Rc<WebContext> = use_context(cx);

                let mut is_init = false;
                let (hash_cell, node) = use_ref(cx, || {
                    let elem = web_cx.document.create_text_node(s);
                    web_cx.parent.append_child(&elem).unwrap();

                    let mut hasher = FxHasher::default();
                    s.hash(&mut hasher);
                    let hash = hasher.finish();

                    is_init = true;
                    (Cell::new(hash), elem)
                });

                let node_clone = node.clone();
                use_on_drop(cx, move || {
                    node_clone.remove();
                });

                if !is_init {
                    let mut hasher = FxHasher::default();
                    s.hash(&mut hasher);
                    let hash = hasher.finish();

                    let last_hash = hash_cell.get();
                    hash_cell.set(hash);

                    if hash != last_hash {
                        node.set_text_content(Some(s));
                    }
                }
            })
        });

        &mut self.content
    }
}

impl<C> Deref for HtmlRoot<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

impl<C> DerefMut for HtmlRoot<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.content
    }
}

pub async fn run<V: View<V> + 'static>(content: V) {
    concoct::run(HtmlRoot { content }).await
}
