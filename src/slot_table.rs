use std::{
    any::Any,
    cell::{Ref, RefCell},
    rc::Rc,
};

const GROUP_FIELDS_SIZE: usize = 5;

pub type Slot = Option<Box<dyn Any>>;

pub struct Anchor {}

struct Inner {
    // An array to store group information that is stored as groups of [Group_Fields_Size]
    // elements of the array. The [groups] array can be thought of as an array of an inline struct.
    groups: Box<[i32]>,

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

impl Default for SlotTable {
    fn default() -> Self {
        Self {
            inner: Rc::new(RefCell::new(Inner {
                groups: Box::new([]),
                groups_len: 0,
                slots: Box::new([]),
                slots_len: 0,
                version: 0,
                anchors: Vec::new(),
            })),
        }
    }
}

impl SlotTable {
    pub fn is_empty(&self) -> bool {
        self.inner.borrow().groups_len == 0
    }

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
        let mut table = self.inner.borrow_mut();
        table.version += 1;

        SlotWriter {
            table: self.clone(),
            capacity: table.groups.len() / GROUP_FIELDS_SIZE,
            current_group_end: table.groups_len,
            group_gap_len: table.groups.len() / GROUP_FIELDS_SIZE - table.groups_len,
            insert_count: 0,
            start_stack: Vec::new(),
            end_stack: Vec::new(),
            node_count: 0,
            node_count_stack: Vec::new(),
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
    capacity: usize,
    current_group_end: usize,
    group_gap_len: usize,
    insert_count: usize,
    start_stack: Vec<usize>,
    end_stack: Vec<usize>,
    node_count: usize,
    node_count_stack: Vec<usize>,
}

impl SlotWriter {
    /// Begin inserting at the current location. beginInsert() can be nested and must be called with
    /// a balanced number of endInsert()
    pub fn begin_insert(&mut self) {
        let insert_count = self.insert_count;
        self.insert_count += 1;

        if insert_count == 0 {
            self.save_current_group_end()
        }
    }

    /// Ends inserting.
    pub fn end_insert(&mut self) {
        if self.insert_count == 0 {
            panic!("Unbalanced begin/end insert")
        }

        self.insert_count -= 1;
        if self.insert_count == 0 {
            if self.node_count_stack.len() != self.start_stack.len() {
                panic!("start_group/end_group mismatch while inserting");
            }
            self.restore_current_group_end();
        }
    }

    pub fn start_group(
        &mut self,
        key: usize,
        object_key: Option<Box<dyn Any>>,
        aux: Option<Box<dyn Any>>,
    ) {
        self.start_group_inner(key, object_key, false, aux)
    }

    /// End the current group. Must be called after the corresponding [startGroup].
    pub fn end_group(&mut self) {}

    // Record the end location as relative to the end of the slot table so when we pop it
    // back off again all inserts and removes that happened while a child group was open
    // are already reflected into its value.
    fn save_current_group_end(&mut self) {
        self.end_stack
            .push(self.capacity - self.group_gap_len - self.current_group_end)
    }

    fn restore_current_group_end(&mut self) -> usize {
        self.current_group_end =
            (self.capacity - self.group_gap_len) - self.end_stack.pop().unwrap();
        self.current_group_end
    }

    fn start_group_inner(
        &mut self,
        _key: usize,
        _object_key: Option<Box<dyn Any>>,
        _is_node: bool,
        _aux: Option<Box<dyn Any>>,
    ) {
        self.node_count_stack.push(self.node_count);
    }

    // Insert `size` number of groups in front of `currentGroup`.
    // These groups are implicitly a child of `parent`.
    fn insert_groups(&mut self, count: usize) {
        if count == 0 {
            return;
        }
    }

    // Move the gap in [groups] to [index].
    fn move_group_gap_to(&mut self, _index: usize) {}
}
