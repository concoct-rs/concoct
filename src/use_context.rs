use crate::{use_ref, LocalContext, Ref, UseRef};
use std::{
    any::{Any, TypeId},
    marker::PhantomData,
    rc::Rc,
};

pub fn use_context<T: 'static>() -> Option<UseContext<T>> {
    let cell =
        use_ref(|| {
            LocalContext::current()
                .scope
                .borrow()
                .contexts
                .get(&TypeId::of::<T>())
                .cloned()
        });

    if cell.get().is_some() {
        Some(UseContext {
            use_ref: cell,
            _marker: PhantomData,
        })
    } else {
        None
    }
}

pub struct UseContext<T> {
    use_ref: UseRef<Option<Rc<dyn Any>>>,
    _marker: PhantomData<T>,
}

impl<T: 'static> UseContext<T> {
    pub fn get(&self) -> Ref<T> {
        let guard = self.use_ref.get();
        let value = std::cell::Ref::map(guard.value, |value| {
            value.as_ref().unwrap().downcast_ref().unwrap()
        });

        Ref {
            value,
            _rc: guard._rc,
        }
    }
}

pub fn use_provider<T: 'static>(make_context: impl FnOnce() -> T) {
    use_ref(|| {
        let value = make_context();
        LocalContext::current()
            .scope
            .borrow_mut()
            .contexts
            .insert(value.type_id(), Rc::new(value));
    });
}
