use crate::{composer::Id, Composer, Semantics, Widget};
use skia_safe::Canvas;
use slotmap::DefaultKey;
use std::{
    any::Any,
    cell::{RefCell, RefMut},
    mem,
    panic::Location,
};

pub fn remember(keys: &[DefaultKey], composable: impl FnOnce()) {
    let location = Location::caller();
    Composer::with(|composer| {
        let mut cx = composer.borrow_mut();
        let id = cx.id(location);

        if let Some(widget) = cx.get::<RememberWidget>(&id) {
            let is_changed = keys.iter().any(|key| {
                cx.changed
                    .iter()
                    .find(|(changed, _id)| changed == key)
                    .is_some()
            });

            if is_changed {
                let children = remember_inner(composer, cx, composable);

                let mut cx = composer.borrow_mut();
                let widget = cx.get_mut::<RememberWidget>(&id).unwrap();
                widget.children = children;
            } else {
                // Keep children in semantics tree
                let children = widget.children.clone();
                for child in children {
                    cx.children.push(child);
                }

                cx.children.push(id);
            }
        } else {
            let children = remember_inner(composer, cx, composable);

            let mut cx = composer.borrow_mut();
            let widget = RememberWidget { children };
            cx.insert(id, widget, None);
        }
    });
}

fn remember_inner(
    composer: &RefCell<Composer>,
    mut cx: RefMut<Composer>,
    composable: impl FnOnce(),
) -> Vec<Id> {
    let parent_children = mem::take(&mut cx.children);
    drop(cx);

    composable();

    let mut cx = composer.borrow_mut();
    let children = mem::replace(&mut cx.children, parent_children);
    cx.children.extend_from_slice(&children);
    children
}

pub struct RememberWidget {
    children: Vec<Id>,
}

impl Widget for RememberWidget {
    fn layout(&mut self, _semantics: &mut Semantics) {}

    fn semantics(&mut self, _semantics: &mut Semantics) {}

    fn paint(&mut self, _semantics: &Semantics, _canvas: &mut Canvas) {}

    fn remove(&mut self, _semantics: &mut Semantics) {}

    fn any(&self) -> &dyn Any {
        self
    }

    fn any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
