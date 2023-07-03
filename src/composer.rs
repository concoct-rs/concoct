use crate::{
    snapshot::{Scope, Snapshot},
    Operation,
};
use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
    fmt, iter,
    marker::PhantomData,
    mem::MaybeUninit,
};

pub enum Slot<T, U> {
    RestartGroup {
        id: TypeId,
        f: Option<Box<dyn FnMut(&mut Composer<T, U>) + Send>>,
    },
    ReplaceableGroup {
        id: TypeId,
    },
    Node {
        data: Box<dyn Any>,
    },
}

impl<T, U> fmt::Debug for Slot<T, U> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RestartGroup { id, f: _ } => {
                f.debug_struct("RestartGroup").field("id", id).finish()
            }
            Self::ReplaceableGroup { id } => {
                f.debug_struct("ReplaceableGroup").field("id", id).finish()
            }
            Self::Node { data } => f.debug_struct("Node").finish(),
        }
    }
}

pub struct Composer<T, U> {
    tracked_states: HashSet<u64>,
    snapshot: Snapshot,
    slots: Box<[MaybeUninit<Slot<T, U>>]>,
    gap_start: usize,
    gap_end: usize,
    gap_size: usize,
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
            slots: Vec::from_iter(iter::repeat_with(|| MaybeUninit::uninit()).take(10))
                .into_boxed_slice(),
            gap_start: 0,
            gap_end: 10,
            gap_size: 10,
            pos: 0,
            _marker: PhantomData,
        }
    }

    pub fn slots(&self) -> impl Iterator<Item = &Slot<T, U>> {
        let mut pos = 0;
        iter::from_fn(move || loop {
            if pos < self.gap_start {
                let slot = unsafe { self.slots[pos].assume_init_ref() };
                pos += 1;
                break Some(slot);
            } else if pos == self.gap_start {
                pos = self.gap_end;
            } else if pos < self.slots.len() - 1 {
                let slot = unsafe { self.slots[pos].assume_init_ref() };
                pos += 1;
                break Some(slot);
            } else {
                break None;
            }
        })
    }

    /// Get the slot at `index`.
    pub fn get(&self, index: usize) -> Option<&Slot<T, U>> {
        self.get_address(index)
            .map(|addr| unsafe { self.slots[addr].assume_init_ref() })
    }

    /// Get the slot at `index`.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Slot<T, U>> {
        self.get_address(index)
            .map(|addr| unsafe { self.slots[addr].assume_init_mut() })
    }

    fn get_address(&self, index: usize) -> Option<usize> {
        let addr = if index >= self.gap_start && index < self.gap_end {
            self.gap_end
        } else {
            index
        };
     

        if addr < self.slots.len() {
            Some(addr)
        } else {
            None
        }
    }

    pub fn cache<R: Clone + 'static>(&mut self, is_invalid: bool, f: impl FnOnce() -> R) -> R {
        if let Some(slot) = self.get_mut(self.pos) {
            if !is_invalid {
                let data = match slot {
                    Slot::Node { data } => data.downcast_ref::<R>().unwrap().clone(),
                    _ => todo!(),
                };
                self.pos += 1;
                data
            } else {
                let value = f();
                let data = Box::new(value.clone());
                *slot = Slot::Node { data };
                value
            }
        } else {
            let value = f();
            let data = Box::new(value.clone());
            let slot = Slot::Node { data };
            self.insert(slot);
            value
        }
    }

    pub fn compose(&mut self, content: impl FnOnce(&mut Self)) -> Vec<Operation<T, U>> {
        content(self);
        Vec::new()
    }

    /*

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
      */

    pub fn insert(&mut self, slot: Slot<T, U>) {
        if self.pos != self.gap_start {}

        self.slots[self.pos] = MaybeUninit::new(slot);
        self.pos += 1;
        self.gap_start += 1;
    }
}

#[cfg(test)]
mod tests {
    use crate::{composer::Slot, Composer, State};
    use std::any::Any;

    #[test]
    fn it_works() {
        let mut composer = Composer::<(), ()>::new();

        composer.compose(|composer| {
            composer.cache(false, || 0);
        });

        let slots: Vec<_> = composer.slots().collect();
        dbg!(&slots);
    }
}
