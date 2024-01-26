use crate::IntoAction;
use std::rc::Rc;

/// Handle to update a scope.
pub struct Handle<T, A = ()> {
    pub(crate) update: Rc<dyn Fn(Rc<dyn Fn(&Handle<T, A>, &mut T) -> Option<A>>)>,
}

impl<T, A> Handle<T, A> {
    /// Send an update to the virtual dom from this handle's scope.
    pub fn update<IA: IntoAction<A>>(&self, f: impl Fn(&Handle<T, A>, &mut T) -> IA + 'static) {
        self.update_raw(Rc::new(move |cx, state| f(cx, state).into_action()))
    }

    /// Send an update to the virtual dom from this handle's scope.
    pub fn update_raw(&self, f: Rc<dyn Fn(&Handle<T, A>, &mut T) -> Option<A>>) {
        (self.update)(f)
    }
}

impl<T, A> Clone for Handle<T, A> {
    fn clone(&self) -> Self {
        Self {
            update: self.update.clone(),
        }
    }
}
