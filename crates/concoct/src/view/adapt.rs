use crate::{build_inner, ActionResult, Scope, View};
use std::{cell::RefCell, marker::PhantomData, rc::Rc};

pub fn adapt<T1, A1, F, V, T2, A2>(f: F, view: V) -> Adapt<T1, A1, F, V, T2, A2>
where
    T1: 'static,
    A1: 'static,
    T2: 'static,
    A2: 'static,
    F: FnMut(&mut T1, Rc<dyn Fn(&mut T2) -> Option<ActionResult<A2>>>) -> Option<ActionResult<A1>>
        + 'static,
    V: View<T2, A2>,
{
    Adapt {
        f: Rc::new(RefCell::new(f)),
        view,
        _marker: PhantomData,
    }
}

pub struct Adapt<T1, A1, F, V, T2, A2> {
    f: Rc<RefCell<F>>,
    view: V,
    _marker: PhantomData<(T1, A1, T2, A2)>,
}

impl<T1, A1, F, V, T2, A2> View<T1, A1> for Adapt<T1, A1, F, V, T2, A2>
where
    T1: 'static,
    A1: 'static,
    T2: 'static,
    A2: 'static,
    F: FnMut(&mut T1, Rc<dyn Fn(&mut T2) -> Option<ActionResult<A2>>>) -> Option<ActionResult<A1>>
        + 'static,
    V: View<T2, A2>,
{
    fn body(&mut self, cx: &crate::Scope<T1, A1>) -> impl View<T1, A1> {
        let parent_update = cx.update.clone();
        let mapper = self.f.clone();
        let update = Rc::new(move |f: Rc<dyn Fn(&mut T2) -> Option<ActionResult<A2>>>| {
            let mapper = mapper.clone();
            parent_update(Rc::new(move |state| mapper.borrow_mut()(state, f.clone())))
        });
        let child_cx = Scope {
            key: cx.key,
            node: cx.node.clone(),
            update,
            is_empty: cx.is_empty.clone(),
            nodes: cx.nodes.clone(),
            contexts: cx.contexts.clone(),
        };
        build_inner(&mut self.view, &child_cx);
    }
}
