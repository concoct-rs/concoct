//! Viewable components.

use crate::{build_inner, hook::use_context, rebuild_inner, Scope, TextViewContext};
use std::{cell::Cell, rc::Rc};

mod adapt;
pub use self::adapt::{adapt, Adapt};

mod memo;
pub use self::memo::{memo, Memo};

/// Viewable component.
pub trait View<T, A = ()> {
    /// View this component, returning its body.
    fn body(&mut self, cx: &Scope<T, A>) -> impl View<T, A>;
}

impl<T, A> View<T, A> for () {
    fn body(&mut self, cx: &Scope<T, A>) -> impl View<T, A> {
        cx.is_empty.set(true);
    }
}

impl<T, A, V: View<T, A>> View<T, A> for &mut V {
    fn body(&mut self, cx: &Scope<T, A>) -> impl View<T, A> {
        (&mut **self).body(cx)
    }
}

macro_rules! impl_view_for_tuple {
    ($($t:tt : $idx:tt),*) => {
        impl<T, A, $($t: View<T, A>),*> View<T, A> for ($($t),*) {
            fn body(&mut self, cx: &Scope<T, A>) -> impl View<T, A> {
                if cx.node.inner.borrow().children.is_empty() {
                    $( build_inner(&mut self.$idx, cx); )*
                } else {
                    $( {
                        let key = cx.node.inner.borrow().children[$idx];
                        let node = cx.nodes.borrow()[key].clone();
                        node.inner.borrow_mut().hook_idx = 0;

                        let cx = Scope {
                            key,
                            node,
                            update: cx.update.clone(),
                            is_empty: Cell::new(false),
                            nodes: cx.nodes.clone(),
                            contexts: cx.contexts.clone()
                        };

                        let mut body = self.$idx.body(&cx);
                        if !cx.is_empty.get() {
                            rebuild_inner(&mut body, &cx);
                        }
                    } )*
                }
                cx.is_empty.set(true);
            }
        }
    };
}

impl_view_for_tuple!(V1: 0, V2: 1);
impl_view_for_tuple!(V1: 0, V2: 1, V3: 2);
impl_view_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3);
impl_view_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3, V5: 4);
impl_view_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3, V5: 4, V6: 5);
impl_view_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3, V5: 4, V6: 5, V7: 6);
impl_view_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3, V5: 4, V6: 5, V7: 6, V8: 7);
impl_view_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3, V5: 4, V6: 5, V7: 6, V8: 7, V9: 8);
impl_view_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3, V5: 4, V6: 5, V7: 6, V8: 7, V9: 8, V10: 9);

impl<T: 'static, A: 'static> View<T, A> for &str {
    fn body(&mut self, cx: &Scope<T, A>) -> impl View<T, A> {
        let text_cx: Rc<TextViewContext<T, A>> = use_context(cx);
        let mut view = text_cx.view.borrow_mut();
        view(cx, self)
    }
}

impl<T: 'static, A: 'static> View<T, A> for String {
    fn body(&mut self, cx: &Scope<T, A>) -> impl View<T, A> {
        let text_cx: Rc<TextViewContext<T, A>> = use_context(cx);
        let mut view = text_cx.view.borrow_mut();
        view(cx, self)
    }
}
