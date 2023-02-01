use crate::{Composer, Widget};
use slotmap::DefaultKey;
use std::{
    any::Any,
    cell::{Ref, RefCell, RefMut},
    marker::PhantomData,
    panic::Location,
    rc::Rc,
};

#[track_caller]
pub fn state<T: 'static>(f: impl FnOnce() -> T) -> State<T> {
    let location = Location::caller();
    let key = Composer::with(|composer| {
        let mut cx = composer.borrow_mut();
        let id = cx.id(location);

        if let Some(widget) = cx.get_mut::<StateWidget<T>>(&id) {
            widget.key
        } else {
            let key = cx.states.insert(id.clone());

            let value = f();
            let widget = StateWidget {
                key,
                value: Rc::new(RefCell::new(Box::new(value))),
            };

            cx.insert(id.clone(), widget, None);

            key
        }
    });

    State {
        key,
        _marker: PhantomData,
    }
}

pub struct State<T> {
    key: DefaultKey,
    _marker: PhantomData<T>,
}

impl<T: 'static> State<T> {
    pub fn get(self) -> StateRef<T> {
        Composer::with(|composer| {
            let cx = composer.borrow();
            let id = &cx.states[self.key];
            let widget = cx.get::<StateWidget<T>>(id).unwrap();

            StateRef {
                rc: widget.value.clone(),
            }
        })
    }
}

pub struct StateRef<T> {
    rc: Rc<RefCell<T>>,
}

impl<T> StateRef<T> {
    /// Return a reference to this state's value.
    pub fn as_ref(&self) -> Ref<'_, T> {
        self.rc.as_ref().borrow()
    }

    /// Return a mutable reference to this state's value.
    /// This will trigger a recompose for this state's parent.
    pub fn as_mut(&self) -> RefMut<'_, T> {
        self.rc.as_ref().borrow_mut()
    }

    pub fn cloned(&self) -> T
    where
        T: Clone,
    {
        self.as_ref().clone()
    }
}

pub struct StateWidget<T> {
    key: DefaultKey,
    value: Rc<RefCell<T>>,
}

impl<T: 'static> Widget for StateWidget<T> {
    fn semantics(&mut self, semantics: &mut crate::Semantics) {}

    fn any(&self) -> &dyn Any {
        self
    }

    fn any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
