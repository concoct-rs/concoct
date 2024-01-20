use super::use_ref;
use rustc_hash::FxHasher;

use std::hash::{Hash, Hasher};

pub fn use_effect(dependencies: impl Hash, effect: impl FnOnce()) {
    let mut hasher = FxHasher::default();
    dependencies.hash(&mut hasher);
    let hash = hasher.finish();

    let mut is_initial = false;
    let last_hash = use_ref(|| {
        is_initial = true;
        hash
    });

    if is_initial || hash != *last_hash {
        effect()
    }
}
