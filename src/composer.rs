use crate::snapshot::{Scope, Snapshot};
use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
};

pub enum Slot {
    RestartGroup {
        id: TypeId,
        f: Option<Box<dyn FnMut(&mut Composer)>>,
    },
}

pub struct Composer {
    tracked_states: HashSet<u64>,
    snapshot: Snapshot,
    slots: Vec<Slot>,
    pos: usize,
    map: HashMap<u64, usize>,
}

impl Composer {
    pub fn new() -> Self {
        Self {
            tracked_states: HashSet::new(),
            snapshot: Snapshot::enter(),
            map: HashMap::new(),
            slots: Vec::new(),
            pos: 0,
        }
    }

    pub fn compose(&mut self, content: impl FnOnce(&mut Self)) {
        content(self);

    }

    pub async fn recompose(&mut self) {
        let ids: Vec<_> = self.snapshot.apply().await.collect();
        for id in ids {
            let idx = *self.map.get(&id).unwrap();
            let mut f = match &mut self.slots[idx] {
                Slot::RestartGroup { id: _, f } => f.take().unwrap(),
            };
            self.pos = idx;

            Scope::default().enter(|| {
                f(self);
            });
        }

        self.tracked_states = HashSet::new();
    }

    pub fn group(&mut self, id: TypeId, mut f: impl FnMut(&mut Self) + 'static) {
        let tracked = self.tracked_states.clone();

        let scope = Scope::default().enter(|| f(self));
        for state_id in &scope.reads {
            if self.tracked_states.insert(*state_id) {
                self.map.insert(*state_id, self.pos);
            }
        }

        let f: Option<Box<dyn FnMut(&mut Composer)>> = if self.tracked_states.is_empty() {
            None
        } else {
            Some(Box::new(f))
        };
        self.slots.push(Slot::RestartGroup { id, f });
        self.pos += 1;

        self.tracked_states = tracked;
    }
}

#[cfg(test)]
mod tests {
    use std::any::Any;

    use crate::{Composer, State};

    #[tokio::test]
    async fn it_works() {
        let mut composer = Composer::new();

        composer.compose(|composer| {
            composer.group(().type_id(), |composer| {
                let state = State::new(0);
                dbg!(*state.get());

                state.update(|n| *n += 1);
            })
        });

        composer.recompose().await;
    }
}
