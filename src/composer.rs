use crate::{
    snapshot::{Scope, Snapshot},
    Operation,
};
use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
    marker::PhantomData,
};

pub enum Slot<T, U> {
    RestartGroup {
        id: TypeId,
        f: Option<Box<dyn FnMut(&mut Composer<T, U>) + Send>>,
    },
    ReplaceableGroup {
        id: TypeId,
    },
}

pub struct Composer<T, U> {
    tracked_states: HashSet<u64>,
    snapshot: Snapshot,
    slots: Vec<Slot<T, U>>,
    pos: usize,
    map: HashMap<u64, usize>,
    _marker: PhantomData<(T, U)>,
}

impl<T, U> Composer<T, U> {
    pub fn new() -> Self {
        Self {
            tracked_states: HashSet::new(),
            snapshot: Snapshot::enter(),
            map: HashMap::new(),
            slots: Vec::new(),
            pos: 0,
            _marker: PhantomData,
        }
    }

    pub fn compose(&mut self, content: impl FnOnce(&mut Self)) -> Vec<Operation<T, U>> {
        content(self);
        Vec::new()
    }

    pub async fn recompose(&mut self) -> Vec<Operation<T, U>> {
        let ids: Vec<_> = self.snapshot.apply().await.collect();
        for id in ids {
            let idx = *self.map.get(&id).unwrap();
            let mut f = match &mut self.slots[idx] {
                Slot::RestartGroup { id: _, f } => f.take().unwrap(),
                _ => todo!(),
            };
            self.pos = idx;

            Scope::default().enter(|| {
                f(self);
            });
        }

        self.tracked_states = HashSet::new();
        Vec::new()
    }

    pub fn restart_group(&mut self, id: TypeId, mut f: impl FnMut(&mut Self) + Send + 'static) {
        let tracked = self.tracked_states.clone();

        let scope = Scope::default().enter(|| f(self));
        for state_id in &scope.reads {
            if self.tracked_states.insert(*state_id) {
                self.map.insert(*state_id, self.pos);
            }
        }

        let f: Option<Box<dyn FnMut(&mut Self) + Send>> = if self.tracked_states.is_empty() {
            None
        } else {
            Some(Box::new(f))
        };
        self.slots.push(Slot::RestartGroup { id, f });
        self.pos += 1;

        self.tracked_states = tracked;
    }

    pub fn replaceable_group(&mut self, id: TypeId, mut f: impl FnMut(&mut Self)) {
        self.slots.push(Slot::ReplaceableGroup { id });
        f(self);
    }
}

#[cfg(test)]
mod tests {
    use crate::{Composer, State};
    use std::any::Any;

    #[tokio::test]
    async fn it_works() {
        let mut composer = Composer::<(), ()>::new();

        composer.compose(|composer| {
            composer.restart_group(().type_id(), |_composer| {
                let state = State::new(0);
                dbg!(*state.get());

                state.update(|n| *n += 1);
            })
        });

        composer.recompose().await;
    }
}
