use crate::snapshot::{Scope, Snapshot};
use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
};

pub struct Composer {
    tracked_states: HashSet<u64>,
    snapshot: Snapshot,
    map: HashMap<u64, TypeId>,
}

impl Composer {
    pub fn new() -> Self {
        Self {
            tracked_states: HashSet::new(),
            snapshot: Snapshot::enter(),
            map: HashMap::new(),
        }
    }

    pub fn compose(&mut self, content: impl FnOnce()) {
        content();
    }

    pub async fn recompose(&mut self, content: impl FnOnce()) {
        for id in self.snapshot.apply().await {
            dbg!(self.map.get(&id).unwrap());
        }

        self.tracked_states = HashSet::new();

        content();
    }

    pub fn group(&mut self, id: TypeId, f: impl FnOnce()) {
        let tracked = self.tracked_states.clone();

        let scope = Scope::default().enter(f);
        for state_id in &scope.reads {
            if !self.tracked_states.insert(*state_id) {
                self.map.insert(*state_id, id);
            }
        }

        self.tracked_states = tracked;
    }
}
