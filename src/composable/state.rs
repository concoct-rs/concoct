use tracing::trace;

use crate::{
    composer::{Id, StateKey},
    Composer, Semantics, Widget,
};
use std::{
    any::Any,
    cell::{Ref, RefCell, RefMut},
    marker::PhantomData,
    panic::Location,
    rc::Rc,
};

/// Create or retrieve application state.
#[track_caller]
pub fn state<T: 'static>(f: impl FnOnce() -> T) -> State<T> {
    let location = Location::caller();
    let key = Composer::with(|composer| {
        let mut cx = composer.borrow_mut();
        let id = cx.id(location);

        if let Some(widget) = cx.get_mut::<StateWidget<T>>(&id) {
            let key = widget.key;
            cx.children.push(id);
            key
        } else {
            let key = cx.states.insert(id.clone());
            drop(cx);

            let value = f();
            let mut cx = composer.borrow_mut();
            let widget = StateWidget {
                key,
                value: Rc::new(RefCell::new(value)),
                group_id: cx.current_group_id.clone(),
            };
            cx.insert(id, widget, None);

            key
        }
    });

    State {
        key,
        _marker: PhantomData,
    }
}

pub struct State<T> {
    pub key: StateKey,
    _marker: PhantomData<Rc<T>>,
}

impl<T> Clone for State<T> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T> Copy for State<T> {}

impl<T: 'static> State<T> {
    /// Get a reference to this state
    pub fn get(self) -> StateRef<T> {
        Composer::with(|composer| {
            let cx = composer.borrow();
            let id = &cx.states[self.key];

            let widget = cx.get::<StateWidget<T>>(id).unwrap();

            StateRef {
                key: self.key,
                group_id: widget.group_id.clone(),
                rc: widget.value.clone(),
            }
        })
    }

    /// Get the cloned current value of this state
    pub fn cloned(self) -> T
    where
        T: Clone,
    {
        self.get().cloned()
    }
}

pub struct StateRef<T> {
    key: StateKey,
    group_id: Id,
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
        trace!("State changed in {}", &self.group_id);

        Composer::with(|composer| {
            composer
                .borrow_mut()
                .changed
                .insert((self.key, self.group_id.clone()))
        });

        self.rc.as_ref().borrow_mut()
    }

    /// Return the cloned current value
    pub fn cloned(&self) -> T
    where
        T: Clone,
    {
        self.as_ref().clone()
    }
}

pub struct StateWidget<T> {
    key: StateKey,
    value: Rc<RefCell<T>>,
    group_id: Id,
}

impl<T: 'static> Widget for StateWidget<T> {
    fn layout(&mut self, _semantics: &mut Semantics) {}

    fn semantics(&mut self, _semantics: &mut Semantics) {}

    fn paint(&mut self, _semantics: &Semantics, _canvas: &mut skia_safe::Canvas) {}

    fn remove(&mut self, _semantics: &mut Semantics) {}

    fn any(&self) -> &dyn Any {
        self
    }

    fn any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
