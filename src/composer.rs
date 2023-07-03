use crate::{
    snapshot::{Scope, Snapshot},
    Apply, Composable,
};
use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
    fmt, iter,
    mem::MaybeUninit,
};

pub enum Slot {
    RestartGroup {
        id: TypeId,
        f: Option<Box<dyn FnMut(&mut Composer) + Send>>,
    },
    ReplaceableGroup {
        id: TypeId,
    },
    Node {
        data: Box<dyn Any>,
    },
}

impl fmt::Debug for Slot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

pub struct Composer {
    applier: Box<dyn Apply>,
    node_ids: Vec<Box<dyn Any>>,
    tracked_states: HashSet<u64>,
    snapshot: Snapshot,
    slots: Box<[MaybeUninit<Slot>]>,
    gap_start: usize,
    gap_end: usize,
    capacity: usize,
    pos: usize,
    map: HashMap<u64, usize>,
}

impl Composer {
    pub fn new(applier: Box<dyn Apply>) -> Self {
        Self::with_capacity(applier, 32)
    }

    pub fn with_capacity(applier: Box<dyn Apply>, capacity: usize) -> Self {
        Self {
            applier,
            node_ids: Vec::new(),
            tracked_states: HashSet::new(),
            snapshot: Snapshot::enter(),
            map: HashMap::new(),
            slots: Vec::from_iter(iter::repeat_with(|| MaybeUninit::uninit()).take(capacity))
                .into_boxed_slice(),
            gap_start: 0,
            gap_end: capacity,
            capacity: capacity,
            pos: 0,
        }
    }

    pub fn slots(&self) -> impl Iterator<Item = &Slot> {
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
    pub fn get(&self, index: usize) -> Option<&Slot> {
        self.get_address(index)
            .map(|addr| unsafe { self.slots[addr].assume_init_ref() })
    }

    /// Get the slot at `index`.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Slot> {
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

    pub fn compose(&mut self, content: impl Composable) {
        self.node_ids.push(self.applier.root());

        content.compose(self, 0);
    }

    pub async fn recompose(&mut self) {
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

    pub fn node(&mut self, node: Box<dyn Any>) {
        if let Some(slot) = self.get_mut(self.pos) {
            let is_replaceable = match slot {
                Slot::ReplaceableGroup { id: _ } | Slot::Node { data: _ } => true,
                _ => false,
            };

            if is_replaceable {
                let parent_id = self.node_ids.last().unwrap().clone();
                self.applier.update(parent_id, node);
                let slot = Slot::Node { data: Box::new(()) };
                *self.get_mut(self.pos).unwrap() = slot;
                self.pos += 1;
                return;
            }
        }

        let parent_id = self.node_ids.last().unwrap();
        self.applier.insert(parent_id, node);
        let slot = Slot::Node { data: Box::new(()) };
        self.insert(slot);
    }

    fn group(&mut self, slot: Slot) {
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

    pub fn insert(&mut self, slot: Slot) {
        if self.pos != self.gap_start {}

        self.slots[self.pos] = MaybeUninit::new(slot);
        self.pos += 1;
        self.gap_start += 1;
    }
}
