use super::{Id, View};
use std::{any::Any, marker::PhantomData};

pub struct Adapt<T1, A1, T2, A2, V, F = fn(&mut T1, AdaptThunk<T2, A2, V>) -> Option<A1>> {
    f: F,
    child: V,
    phantom: PhantomData<fn() -> (T1, A1, T2, A2)>,
}

pub struct AdaptThunk<'a, T2, A2, V: View<T2, A2>> {
    child: &'a V,
    id_path: &'a [Id],
    message: Box<dyn std::any::Any>,
    _marker: PhantomData<(T2, A2)>,
}

impl<T1, A1, T2, A2, V, F> Adapt<T1, A1, T2, A2, V, F>
where
    V: View<T2, A2>,
    F: Fn(&mut T1, AdaptThunk<T2, A2, V>) -> Option<A1>,
{
    pub fn new(f: F, child: V) -> Self {
        Adapt {
            f,
            child,
            phantom: Default::default(),
        }
    }
}

impl<'a, T2, A2, V: View<T2, A2>> AdaptThunk<'a, T2, A2, V> {
    pub fn call(self, _app_state: &mut T2) -> Option<A2> {
        todo!()
    }
}

impl<T1, A1, T2, A2, V, F> View<T1, A1> for Adapt<T1, A1, T2, A2, V, F>
where
    V: View<T2, A2>,
    F: Fn(&mut T1, AdaptThunk<T2, A2, V>) -> Option<A1>,
{
    fn build(&mut self, cx: &mut super::BuildContext) -> Id {
        self.child.build(cx)
    }

    fn rebuild(&mut self, cx: &mut super::BuildContext, old: &mut Self) {
        self.child.rebuild(cx, &mut old.child)
    }

    fn message(&mut self, _state: &mut T1, _id_path: &[Id], _message: &dyn Any) {
        todo!()
    }

    fn layout(&mut self, cx: &mut super::LayoutContext, id: Id) {
        self.child.layout(cx, id)
    }

    fn paint(&mut self, taffy: &taffy::Taffy, canvas: &mut skia_safe::Canvas) {
        self.child.paint(taffy, canvas)
    }
}
