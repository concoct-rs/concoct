use crate::macros::trace;

use super::use_ref;
use rustc_hash::FxHasher;
use std::hash::{Hash, Hasher};

/// Hook to cache a value and run an effect when it's changed.
pub fn use_effect(input: impl Hash, effect: impl FnOnce()) {
    let mut hasher = FxHasher::default();
    input.hash(&mut hasher);
    let hash = hasher.finish();

    let mut is_initial = false;
    let last_hash = use_ref(|| {
        is_initial = true;
        hash
    });

    if is_initial || hash != *last_hash {
        trace!("running effect");

        effect()
    }
}
