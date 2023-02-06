use std::panic::Location;

use crate::{composer::WidgetNode, Composer, Widget};

#[track_caller]
pub fn widget<T, W>(state: T, f: impl FnOnce(T) -> W, g: impl FnOnce(T, &mut WidgetNode))
where
    W: Widget,
{
    let location = Location::caller();
    Composer::with(|composer| {
        let mut cx = composer.borrow_mut();
        let id = cx.id(location);

        if let Some(node) = cx.get_node_mut(&id) {
            g(state, node);
            cx.children.push(id);
        } else {
            let widget = f(state);
            cx.insert(id, widget, None);
        }
    })
}
