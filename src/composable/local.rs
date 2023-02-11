use super::{container::Container, widget};
use crate::{Composer, Widget};
use std::{
    any::{Any, TypeId},
    rc::Rc,
};

/// A composition local provided by a [`provider`].
pub fn local<T: 'static>() -> Option<Rc<T>> {
    Composer::with(|composer| {
        let cx = composer.borrow();
        cx.contexts.get(&TypeId::of::<T>()).and_then(|id| {
            let widget: &LocalWidget<T> = cx.get(id)?;
            Some(widget.value.clone())
        })
    })
}

/// Provide a composition local to the given composable
/// ```no_run
/// use concoct::composable::{local, provider};
///
/// provider(false, || {
///     let local_bool = local::<bool>().unwrap();
/// })
/// ```
#[track_caller]
pub fn provider<T: 'static>(value: T, mut composable: impl FnMut() + 'static) {
    let value = Rc::new(value);

    Container::build_row(move || {
        let id = widget(
            (),
            |_| LocalWidget {
                value: value.clone(),
            },
            |_, _| {},
        );

        Composer::with(|composer| composer.borrow_mut().contexts.insert(TypeId::of::<T>(), id));

        composable();
    })
    .flex_grow(1.)
    .view()
}

pub struct LocalWidget<T> {
    pub value: Rc<T>,
}

impl<T: 'static> Widget for LocalWidget<T> {
    fn layout(&mut self, _semantics: &mut crate::Semantics) {}

    fn semantics(&mut self, _semantics: &mut crate::Semantics) {}

    fn paint(&mut self, _semantics: &crate::Semantics, _canvas: &mut skia_safe::Canvas) {}

    fn remove(&mut self, _semantics: &mut crate::Semantics) {}

    fn any(&self) -> &dyn Any {
        self
    }

    fn any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
