use slotmap::DefaultKey;

use crate::Composer;

pub fn remember(keys: &[DefaultKey], f: impl FnOnce()) {
    Composer::with(|composer| {
        let cx = composer.borrow();

        let is_changed = keys.iter().any(|key| {
            cx.changed
                .iter()
                .find(|(changed, _id)| changed == key)
                .is_some()
        });
        if is_changed {
            drop(cx);
            f()
        }
    })
}
