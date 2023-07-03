use crate::{
    snapshot::{Scope, Snapshot},
    Composable, Operation,
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
            Self::Node { data: _ } => f.debug_struct("Node").finish(),
        }
    }
}

pub struct Composer<T, U> {
    tracked_states: HashSet<u64>,
    snapshot: Snapshot,
    slots: Box<[MaybeUninit<Slot<T, U>>]>,
    gap_start: usize,
    gap_end: usize,
    capacity: usize,
    pos: usize,
    map: HashMap<u64, usize>,
    _marker: PhantomData<(T, U)>,
}

impl<T, U> Composer<T, U> {
    pub fn new() -> Self {
        Self::with_capacity(32)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            tracked_states: HashSet::new(),
            snapshot: Snapshot::enter(),
            map: HashMap::new(),
            slots: Vec::from_iter(iter::repeat_with(|| MaybeUninit::uninit()).take(capacity))
                .into_boxed_slice(),
            gap_start: 0,
            gap_end: capacity,
            capacity: capacity,
            pos: 0,
            _marker: PhantomData,
        }
    }

    pub fn slots(&self) -> impl Iterator<Item = &Slot<T, U>> {
        let mut pos = 0;
        iter::from_fn(move || {
            if let Some(slot) = self.get(pos) {
                pos += 1;
                Some(slot)
            } else {
                None
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

    pub fn compose(&mut self, content: impl Composable<T, U>) -> Vec<Operation<T, U>> {
        content.compose(self, 0);
        Vec::new()
    }

    pub async fn recompose(&mut self) -> Vec<Operation<T, U>> {
        let ids: Vec<_> = self.snapshot.apply().await.collect();
        for id in ids {
            let idx = *self.map.get(&id).unwrap();
            let mut restart = match self.get_mut(idx).unwrap() {
                Slot::RestartGroup { id: _, f } => f.take().unwrap(),
                _ => todo!(),
            };
            self.pos = idx + 1;

            Scope::default().enter(|| {
                restart(self);
            });

            match self.get_mut(idx).unwrap() {
                Slot::RestartGroup { id: _, f } => *f = Some(restart),
                _ => todo!(),
            };
        }

        self.tracked_states = HashSet::new();
        Vec::new()
    }

    pub fn restart_group(
        &mut self,
        id: TypeId,
        mut f: impl FnMut(&mut Self) + Clone + Send + 'static,
    ) {
        let idx = self.pos;
        self.group(Slot::RestartGroup { id, f: None });

        let tracked = self.tracked_states.clone();
        let scope = Scope::default().enter(|| f(self));

        for state_id in &scope.state_ids {
            if self.tracked_states.insert(*state_id) {
                self.map.insert(*state_id, idx);
            }
        }

        let restart: Option<Box<dyn FnMut(&mut Self) + Send>> = if self.tracked_states.is_empty() {
            None
        } else {
            Some(Box::new(f.clone()))
        };
        if let Slot::RestartGroup { id: _, f } = self.get_mut(idx).unwrap() {
            *f = restart;
        } else {
            todo!()
        }

        self.tracked_states = tracked;
    }

    pub fn replaceable_group<R>(&mut self, id: TypeId, f: impl FnOnce(&mut Self) -> R) -> R {
        self.group(Slot::ReplaceableGroup { id });

        f(self)
    }

    fn group(&mut self, slot: Slot<T, U>) {
        if let Some(current_slot) = self.get_mut(self.pos) {
            match current_slot {
                Slot::ReplaceableGroup { id: _ } => {
                    *current_slot = slot;
                    self.pos += 1;
                }
                _ => {
                    self.insert(slot);
                }
            }
        } else {
            self.insert(slot);
        }
    }

    pub fn insert(&mut self, slot: Slot<T, U>) {
        if self.pos != self.gap_start {}

        self.slots[self.pos] = MaybeUninit::new(slot);
        self.pos += 1;
        self.gap_start += 1;
    }
}
