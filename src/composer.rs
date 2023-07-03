use crate::snapshot::{Scope, Snapshot};
use std::collections::HashSet;

pub struct Composer {
    tracked_states: HashSet<u64>,
    snapshot: Snapshot,
}

impl Composer {
    pub fn compose(&mut self, content: impl FnOnce()) {
        content();
    }

    pub async fn recompose(&mut self, content: impl FnOnce()) {
        self.snapshot.apply().await;

        content();
    }

    pub fn group(&mut self, f: impl FnOnce()) {
        let scope = Scope::default().enter(f);
        self.tracked_states.extend(scope.reads);
    }
}
