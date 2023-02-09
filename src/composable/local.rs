use super::{container::Container, widget};
use crate::{Composer, Widget};
use std::{
    any::{Any, TypeId},
    rc::Rc,
};

pub fn local<T: 'static>() -> Option<Rc<T>> {
    Composer::with(|composer| {
        let cx = composer.borrow();
        cx.contexts.get(&TypeId::of::<T>()).map(|id| {
            let widget: &ContextWidget<T> = cx.get(id).unwrap();
            widget.value.clone()
        })
    })
}

#[track_caller]
pub fn provider<T: 'static>(value: T, mut composable: impl FnMut() + 'static) {
    let value = Rc::new(value);

    Container::row(move || {
        let id = widget(
            (),
            |_| ContextWidget {
                value: value.clone(),
            },
            |_, _| {},
        );

        Composer::with(|composer| composer.borrow_mut().contexts.insert(TypeId::of::<T>(), id));

        composable();
    });
}

pub struct ContextWidget<T> {
    value: Rc<T>,
}

impl<T: 'static> Widget for ContextWidget<T> {
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
