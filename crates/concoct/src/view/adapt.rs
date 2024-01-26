use crate::{Handle, Scope, View};
use std::{cell::RefCell, marker::PhantomData, rc::Rc};

/// Adapt a view's state to a different one.
pub fn adapt<F, V, T1, A1, T2, A2>(f: F, view: V) -> Adapt<F, V, T1, A1, T2, A2>
where
    F: FnMut(&mut T1, Rc<dyn Fn(&mut T2) -> Option<A2>>) -> Option<A1> + 'static,
    V: View<T2, A2>,
    T1: 'static,
    A1: 'static,
    T2: 'static,
    A2: 'static,
{
    Adapt {
        f: Rc::new(RefCell::new(f)),
        view,
        _marker: PhantomData,
    }
}

/// View for the [`adapt`] function.
pub struct Adapt<F, V, T1, A1, T2, A2> {
    f: Rc<RefCell<F>>,
    view: V,
    _marker: PhantomData<(T1, A1, T2, A2)>,
}

impl<T1, A1, F, V, T2, A2> View<T1, A1> for Adapt<F, V, T1, A1, T2, A2>
where
    F: FnMut(
            &Handle<T1, A1>,
            &mut T1,
            Rc<dyn Fn(&Handle<T2, A2>, &mut T2) -> Option<A2>>,
        ) -> Option<A1>
        + 'static,
    V: View<T2, A2>,
    T1: 'static,
    A1: 'static,
    T2: 'static,
    A2: 'static,
{
    fn body(&mut self, cx: &Scope<T1, A1>) -> impl View<T1, A1> {
        let parent_update = cx.update.clone();
        let mapper = self.f.clone();
        let update = Rc::new(
            move |f: Rc<dyn Fn(&Handle<T2, A2>, &mut T2) -> Option<A2>>| {
                let mapper = mapper.clone();
                parent_update(Rc::new(move |cx, state| {
                    mapper.borrow_mut()(cx, state, f.clone())
                }))
            },
        );
        let child_cx = Scope {
            key: cx.key,
            node: cx.node.clone(),
            parent: cx.parent,
            update,
            is_empty: cx.is_empty.clone(),
            nodes: cx.nodes.clone(),
            contexts: cx.contexts.clone(),
        };

        if cx.node.inner.borrow().children.is_empty() {
            child_cx.build(&mut self.view);
        } else {
            child_cx.rebuild(&mut self.view);
        }
    }
}
