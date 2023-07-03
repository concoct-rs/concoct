use crate::{
    snapshot::{Scope, Snapshot},
    Apply, Composable, State,
};
use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
    fmt, iter,
    mem::MaybeUninit,
};

pub enum GroupKind {
    Restart {
        f: Option<Box<dyn FnMut(&mut Composer) + Send>>,
    },
    Replace,
}

impl fmt::Debug for GroupKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Restart { f: _ } => f.debug_struct("Restart").finish(),
            Self::Replace {} => f.debug_struct("Replace").finish(),
        }
    }
}

pub enum Slot {
    Group {
        id: TypeId,
        len: usize,
        kind: GroupKind,
    },
    Node {
        data: Option<Box<dyn Any>>,
    },
}

impl fmt::Debug for Slot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Group { id: _, len, kind } => f
                .debug_struct("Group")
                .field("len", len)
                .field("kind", kind)
                .finish(),
            Self::Node { data } => f.debug_struct("Node").field("data", data).finish(),
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
    contexts: HashMap<TypeId, Vec<State<Box<dyn Any + Send>>>>,
    child_count: usize,
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
            contexts: HashMap::new(),
            child_count: 0,
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
            self.pos = idx + 1;

            let mut restart = match self.get_mut(idx).unwrap() {
                Slot::Group {
                    id: _,
                    len: _,
                    kind: GroupKind::Restart { f },
                } => f.take().unwrap(),
                _ => todo!(),
            };
            Scope::default().enter(|| {
                restart(self);
            });

            match self.get_mut(idx).unwrap() {
                Slot::Group {
                    id: _,
                    len: _,
                    kind: GroupKind::Restart { f },
                } => *f = Some(restart),
                _ => todo!(),
            };
        }

        self.tracked_states = HashSet::new();
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

    pub fn cache<R>(&mut self, is_invalid: bool, f: impl FnOnce() -> R) -> R
    where
        R: Clone + 'static,
    {
        if let Some(slot) = self.peek_mut() {
            let value = if !is_invalid {
                match slot {
                    Slot::Node { data } => {
                        data.as_ref().unwrap().downcast_ref::<R>().unwrap().clone()
                    }
                    _ => todo!(),
                }
            } else {
                let value = f();
                let data = Box::new(value.clone());
                *slot = Slot::Node { data: Some(data) };
                value
            };

            self.pos += 1;
            self.child_count += 1;

            value
        } else {
            let value = f();
            let data = Box::new(value.clone());
            let slot = Slot::Node { data: Some(data) };
            self.insert(slot);
            value
        }
    }

    pub fn restart_group(
        &mut self,
        id: TypeId,
        mut f: impl FnMut(&mut Self) + Clone + Send + 'static,
    ) {
        let idx = self.pos;
        self.group(Slot::Group {
            id,
            len: 0,
            kind: GroupKind::Restart { f: None },
        });

        let tracked = self.tracked_states.clone();

        let parent_child_count = self.child_count;
        self.child_count = 0;

        let scope = Scope::default().enter(|| f(self));

        let child_count = self.child_count;
        self.child_count = parent_child_count;

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
        if let Slot::Group {
            id: _,
            len,
            kind: GroupKind::Restart { f },
        } = self.get_mut(idx).unwrap()
        {
            *len = child_count;
            *f = restart;
        } else {
            todo!()
        }

        self.tracked_states = tracked;
    }

    pub fn replaceable_group<R>(&mut self, id: TypeId, f: impl FnOnce(&mut Self) -> R) -> R {
        let idx = self.pos;
        self.group(Slot::Group {
            id,
            len: 0,
            kind: GroupKind::Replace {},
        });

        let parent_child_count = self.child_count;
        self.child_count = 0;

        let output = f(self);

        let child_count = self.child_count;
        self.child_count = parent_child_count;

        if let Slot::Group {
            id: _,
            len,
            kind: _,
        } = self.get_mut(idx).unwrap()
        {
            *len = child_count;
        }

        output
    }

    fn group(&mut self, slot: Slot) {
        if let Some(current_slot) = self.peek_mut() {
            match current_slot {
                Slot::Group {
                    id: _,
                    len: _,
                    kind: GroupKind::Replace,
                } => {
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

    pub fn node(&mut self, node: Box<dyn Any>) {
        if let Some(slot) = self.peek_mut() {
            let is_replaceable = match slot {
                Slot::Group {
                    id: _,
                    len: _,
                    kind: GroupKind::Replace,
                }
                | Slot::Node { data: _ } => true,
                _ => false,
            };

            if is_replaceable {
                let parent_id = self.node_ids.last().unwrap().clone();
                self.applier.update(parent_id, node);
                let slot = Slot::Node { data: None };
                *self.peek_mut().unwrap() = slot;

                self.pos += 1;
                self.child_count += 1;

                return;
            }
        }

        let parent_id = self.node_ids.last().unwrap();
        self.applier.insert(parent_id, node);
        let slot = Slot::Node { data: None };
        self.insert(slot);
    }

    pub fn provide(&mut self, value: Box<dyn Send + Any>) {
        let id = value.as_ref().type_id();
        let state = State::new(value);

        if let Some(values) = self.contexts.get_mut(&id) {
            values.push(state);
        } else {
            self.contexts.insert(id, vec![state]);
        }
    }

    pub fn context<T: Clone + 'static>(&self) -> T {
        self.contexts
            .get(&TypeId::of::<T>())
            .unwrap()
            .last()
            .unwrap()
            .get()
            .downcast_ref::<T>()
            .unwrap()
            .clone()
    }

    /// Get the slot at `index`.
    fn get(&self, index: usize) -> Option<&Slot> {
        self.get_address(index)
            .map(|addr| unsafe { self.slots[addr].assume_init_ref() })
    }

    /// Get the slot at `index`.
    fn get_mut(&mut self, index: usize) -> Option<&mut Slot> {
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

    fn peek_mut(&mut self) -> Option<&mut Slot> {
        self.get_mut(self.pos)
    }

    /// Insert a slot into the current position.
    fn insert(&mut self, slot: Slot) {
        if self.pos != self.gap_start {}

        self.slots[self.pos] = MaybeUninit::new(slot);
        self.pos += 1;
        self.gap_start += 1;
        self.child_count += 1;
    }
}

impl fmt::Debug for Composer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let slots: Vec<_> = self.slots().collect();
        f.debug_struct("Composer").field("slots", &slots).finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::{composable, compose, node, remember, Composer, State};

    #[composable]
    fn app() {
        let count = compose!(remember(|| State::new(0)));

        count.update(|n| *n = 1);

        if *count.get() == 0 {
            compose!(node(()));
        }
    }

    #[tokio::test]
    async fn it_works() {
        let mut composer = Composer::new(Box::new(()));
        composer.compose(app());

        composer.recompose().await;

        dbg!(composer);
    }
}
