//! # Composer
//! The composer stores the data from the composition tree.
//!
//! ```ignore
//! #[composable]
//! fn app() {
//!     compose!(node(0));
//! }
//!
//! // Will be stored as:
//!
//! Group {
//!     len: 2,
//!     kind: Restart,
//! },
//! Group {
//!     len: 1,
//!     kind: Restart,
//! },
//! Node {
//!     data: None,
//! },
//! ```

use crate::{
    snapshot::{Scope, Snapshot},
    Apply, Composable, State,
};
use std::{
    any::{Any, TypeId},
    collections::{HashMap, HashSet},
    fmt, iter,
    mem::MaybeUninit,
    rc::Rc,
};

use self::slot_table::SlotTable;

pub mod slot_table;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotKind {
    RestartGroup,
    ReplaceGroup,
    Node,
    Data,
}

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
    Data {
        value: Option<Box<dyn Any>>,
    },
    Node {
        id: Rc<dyn Any>,
    },
}

impl Slot {
    pub fn kind(&self) -> SlotKind {
        match self {
            Self::Group {
                id: _,
                len: _,
                kind,
            } => match kind {
                GroupKind::Replace => SlotKind::ReplaceGroup,
                GroupKind::Restart { f: _ } => SlotKind::RestartGroup,
            },
            Self::Data { value: _ } => SlotKind::Data,
            Self::Node { id: _ } => SlotKind::Node,
        }
    }
}

impl fmt::Debug for Slot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Group { id: _, len, kind } => f
                .debug_struct("Group")
                .field("len", len)
                .field("kind", kind)
                .finish(),
            Self::Data { value: data } => f.debug_struct("Data").field("data", data).finish(),
            Self::Node { id: _ } => f.debug_struct("Node").finish(),
        }
    }
}

/// Composer for a UI tree that builds and rebuilds a depth-first traversal of the tree's nodes.
///
/// See the [`module`](concoct::composer) docs for more.
pub struct Composer {
    applier: Box<dyn Apply>,
    node_ids: Vec<Rc<dyn Any>>,
    tracked_states: HashSet<u64>,
    snapshot: Snapshot,
    pub slot_table: SlotTable,
    map: HashMap<u64, usize>,
    contexts: HashMap<TypeId, Vec<State<Box<dyn Any + Send>>>>,
    child_count: usize,
}

impl Default for Composer {
    fn default() -> Self {
        Self::new(Box::new(()))
    }
}

fn new_slots(capacity: usize) -> Box<[MaybeUninit<Slot>]> {
    Vec::from_iter(iter::repeat_with(|| MaybeUninit::uninit()).take(capacity)).into_boxed_slice()
}

impl Composer {
    /// Create a new composer with the given `applier`.
    pub fn new(applier: Box<dyn Apply>) -> Self {
        Self::with_capacity(applier, 32)
    }

    /// Create a new composer with the given `applier` and capacity.
    pub fn with_capacity(applier: Box<dyn Apply>, capacity: usize) -> Self {
        Self {
            applier,
            node_ids: Vec::new(),
            tracked_states: HashSet::new(),
            snapshot: Snapshot::enter(),
            map: HashMap::new(),
            slot_table: SlotTable::with_capacity(capacity),
            contexts: HashMap::new(),
            child_count: 0,
        }
    }

    /// Compose the initial content.
    pub fn compose(&mut self, content: impl Composable) {
        self.node_ids.push(self.applier.root().into());
        content.compose(self, 0);
    }

    /// Recompose the current content from `compose` when a state change is requested.
    /// Updating [`State`] will trigger this method.
    pub async fn recompose(&mut self) {
        let ids: Vec<_> = self.snapshot.apply().await.collect();
        for id in ids {
            let idx = *self.map.get(&id).unwrap();
            self.slot_table.pos = idx + 1;

            let mut restart = match self.slot_table.get_mut(idx).unwrap() {
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

            match self.slot_table.get_mut(idx).unwrap() {
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

    /// Cache a value in the composition.
    /// During initial composition `f` is called to produce the value that is then stored in the slot table.
    /// During recomposition, if `is_invalid` is false the value is obtained from the slot table and `f` is not invoked.
    /// If `is_invalid` is false a new value is produced by calling [block] and the slot table is updated to
    /// contain the new value.
    pub fn cache<R>(&mut self, is_invalid: bool, f: impl FnOnce() -> R) -> R
    where
        R: Clone + 'static,
    {
        if let Some(slot) = self.slot_table.peek_mut() {
            let value = if !is_invalid {
                match slot {
                    Slot::Data { value: data } => {
                        data.as_ref().unwrap().downcast_ref::<R>().unwrap().clone()
                    }
                    _ => todo!(),
                }
            } else {
                let value = f();
                let data = Box::new(value.clone());
                *slot = Slot::Data { value: Some(data) };
                value
            };

            self.slot_table.pos += 1;
            self.child_count += 1;

            value
        } else {
            let value = f();
            let data = Box::new(value.clone());
            let slot = Slot::Data { value: Some(data) };
            self.slot_table.insert(slot);
            value
        }
    }

    pub fn restart_group(
        &mut self,
        id: TypeId,
        mut f: impl FnMut(&mut Self) + Clone + Send + 'static,
    ) {
        self.group(
            id,
            GroupKind::Restart {
                f: Some(Box::new(f.clone())),
            },
            |me, idx| {
                let tracked = me.tracked_states.clone();

                let scope = Scope::default().enter(|| {
                    f(me);
                });

                for state_id in &scope.state_ids {
                    if me.tracked_states.insert(*state_id) {
                        me.map.insert(*state_id, idx);
                    }
                }

                me.tracked_states = tracked;
            },
        );
    }

    /// Create or update a replacable group.
    /// A replacable group is a group that cannot be moved and can only either inserted, removed, or replaced.
    /// For example, this is the group created by most control flow constructs (such as an `if`).
    pub fn replaceable_group<R>(&mut self, id: TypeId, f: impl FnOnce(&mut Self) -> R) -> R {
        self.group(id, GroupKind::Replace, |me, _idx| f(me))
    }

    fn group<R>(
        &mut self,
        id: TypeId,
        kind: GroupKind,
        f: impl FnOnce(&mut Self, usize) -> R,
    ) -> R {
        let idx = self.slot_table.pos;
        let last_len = self.start_group(id, GroupKind::Replace);

        let parent_child_count = self.child_count;
        self.child_count = 0;

        let out = f(self, idx);

        let child_count = self.child_count;
        self.child_count += parent_child_count;

        if let Slot::Group {
            id: _,
            len,
            kind: this_kind,
        } = self.slot_table.get_mut(idx).unwrap()
        {
            *this_kind = kind;
            *len = child_count;

            if let Some(last_len) = last_len {
                if child_count < last_len {
                    // self.remove(idx, idx + last_len - child_count);
                }
            }
        }

        out
    }

    fn start_group(&mut self, id: TypeId, kind: GroupKind) -> Option<usize> {
        if let Some(slot) = self.slot_table.peek_mut() {
            if let Slot::Group {
                id: last_id,
                len: last_len,
                kind: last_kind,
            } = slot
            {
                match last_kind {
                    GroupKind::Replace => {
                        *slot = Slot::Group { id, len: 0, kind };
                        self.slot_table.pos += 1;
                        return None;
                    }
                    GroupKind::Restart { f: _ } => {
                        if id == *last_id {
                            *last_kind = kind;
                            let len = *last_len;
                            self.slot_table.pos += 1;
                            return Some(len);
                        }
                    }
                }
            }
        }

        self.slot_table.insert(Slot::Group { id, len: 0, kind });
        None
    }

    /// Advance the cursor to the next group.
    pub fn skip_group(&mut self) {
        if let Slot::Group { len, .. } = self.slot_table.peek_mut().unwrap() {
            self.slot_table.pos += *len
        } else {
            todo!()
        };
    }

    /// Create or update a node on the tree.
    pub fn node(&mut self, node: Box<dyn Any>) {
        if let Some(slot) = self.slot_table.peek_mut() {
            let is_replaceable = match slot {
                Slot::Group {
                    id: _,
                    len: _,
                    kind: GroupKind::Replace,
                }
                | Slot::Data { value: _ } => true,
                _ => false,
            };

            if is_replaceable {
                let parent_id = self.node_ids.last().unwrap().clone();
                self.applier.update(&parent_id, node);
                let slot = Slot::Data { value: None };
                *self.slot_table.peek_mut().unwrap() = slot;

                self.slot_table.pos += 1;
                self.child_count += 1;

                return;
            }
        }

        let parent_id = self.node_ids.last().unwrap();
        self.applier.insert(parent_id, node);
        let slot = Slot::Data { value: None };
        self.slot_table.insert(slot);
    }

    /// Provide a context with the given `value`.
    pub fn provide(&mut self, value: Box<dyn Send + Any>) {
        let id = value.as_ref().type_id();
        let state = State::new(value);

        if let Some(values) = self.contexts.get_mut(&id) {
            values.push(state);
        } else {
            self.contexts.insert(id, vec![state]);
        }
    }

    /*

    fn remove(&mut self, start: usize, end: usize) {
        let start_addr = self.get_address(start).unwrap();
        let end_addr = self.get_address(end).unwrap();

        for slot in &mut self.slots[start_addr..end_addr] {
            let slot = unsafe { slot.assume_init_mut() };
            if let Slot::Node { id } = slot {
                self.applier.remove(&*id);
            }
        }

        self.gap_start = start_addr;
        self.gap_end = end_addr;
    }
     */
}

impl fmt::Debug for Composer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let slots: Vec<_> = self.slot_table.slots().collect();
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
