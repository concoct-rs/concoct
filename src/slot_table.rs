use std::{
    any::Any,
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

pub type Slot = Option<Box<dyn Any>>;

pub struct Anchor {}

struct Inner {
    // An array to store group information that is stored as groups of [Group_Fields_Size]
    // elements of the array. The [groups] array can be thought of as an array of an inline struct.
    groups: Box<i32>,

    // The number of groups contained in [groups].
    groups_len: usize,

    // An array that stores the slots for a group. The slot elements for a group start at the
    // offset returned by [dataAnchor] of [groups] and continue to the next group's slots or to
    // [slotsSize] for the last group. When in a writer the [dataAnchor] is an anchor instead of
    // an index as [slots] might contain a gap.
    slots: Box<[Option<Box<dyn Any>>]>,

    // The number of slots used in [slots].
    slots_len: usize,

    // An internal version that is incremented whenever a writer is created. This is used to
    // detect when an iterator created by [CompositionData] is invalid.
    version: usize,

    // A list of currently active anchors.
    anchors: Vec<Anchor>,
}

#[derive(Clone)]
pub struct SlotTable {
    inner: Rc<RefCell<Inner>>,
}

impl SlotTable {
    /// Open a reader.
    /// Any number of readers can be created but a slot table cannot be read while it is being written to.
    pub fn reader(&self) -> SlotReader {
        SlotReader {
            table: self.clone(),
            current_slot: 0,
            current_slot_end: 0,
            empty_count: 0,
        }
    }

    /// Open a writer. Only one writer can be created for a slot table at a time and all readers
    /// must be closed an do readers can be created while the slot table is being written to.
    pub fn writer(&self) -> SlotWriter {
        self.inner.borrow_mut().version += 1;

        SlotWriter {
            table: self.clone(),
        }
    }
}

pub struct SlotReader {
    table: SlotTable,
    current_slot: usize,
    current_slot_end: usize,
    empty_count: usize,
}

impl SlotReader {
    pub fn next_slot(&mut self) -> Option<Ref<Option<Box<dyn Any>>>> {
        if self.empty_count > 0 || self.current_slot >= self.current_slot_end {
            return None;
        }

        let idx = self.current_slot;
        self.current_slot += 1;
        Some(Ref::map(self.table.inner.borrow(), |table| {
            &table.slots[idx]
        }))
    }
}

pub struct SlotWriter {
    table: SlotTable,
}

impl SlotWriter {}
