use std::rc::Rc;

pub enum ActionResult<A> {
    Action(A),
    Rebuild,
}

pub struct Scope<T, A = ()> {
    update: Rc<dyn Fn(Rc<dyn Fn(T) -> Option<ActionResult<A>>>)>,
}

pub trait View<T, A = ()> {
    fn body(&mut self, cx: &mut Scope<T, A>) -> impl View<T, A>;
}
