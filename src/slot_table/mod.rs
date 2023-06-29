use std::any::{Any, TypeId};

mod reader;
pub use reader::SlotReader;

mod writer;
pub use writer::SlotWriter;

const GROUP_FIELDS_SIZE: usize = 1;

pub trait Slot {
    fn any(&self) -> &dyn Any;

    fn any_eq(&self, other: &dyn Any) -> bool;
}

impl<T: PartialEq + 'static> Slot for T {
    fn any(&self) -> &dyn Any {
        todo!()
    }

    fn any_eq(&self, other: &dyn Any) -> bool {
        Some(self) == other.downcast_ref::<T>()
    }
}

const NODE_COUNT_MASK: u32 = 0b0000_0011_1111_1111__1111_1111_1111_1111;

#[derive(Clone, Copy)]
struct Group {
    id: TypeId,
    mask: u32,
    parent_anchor: usize,
    size_offset: usize,
    data_anchor: usize,
}

impl Group {
    pub fn empty() -> Self {
        Self {
            id: TypeId::of::<()>(),
            mask: 0,
            parent_anchor: 0,
            size_offset: 0,
            data_anchor: 0,
        }
    }

    pub fn new(
        id: TypeId,
        is_node: bool,
        has_data_key: bool,
        has_data: bool,
        parent_anchor: usize,
        data_anchor: usize,
    ) -> Self {
        const NODE_BIT_MASK: u32 = 0b0100_0000_0000_0000__0000_0000_0000_0000;
        const OBJECT_KEY_MASK: u32 = 0b0010_0000_0000_0000__0000_0000_0000_0000;
        const AUX_MASK: u32 = 0b0001_0000_0000_0000__0000_0000_0000_0000;

        let node_bit = if is_node { NODE_BIT_MASK } else { 0 };
        let data_key_bit = if has_data_key { OBJECT_KEY_MASK } else { 0 };
        let data_bit = if has_data { AUX_MASK } else { 0 };

        Self {
            id,
            mask: node_bit | data_key_bit | data_bit,
            parent_anchor,
            size_offset: 0,
            data_anchor,
        }
    }

    pub fn node_count(&self) -> u32 {
        self.mask & NODE_COUNT_MASK
    }

    pub fn set_node_count(&mut self, value: u32) {
        assert!(value < NODE_COUNT_MASK);
        self.mask &= !NODE_COUNT_MASK | value
    }
}

#[derive(Default)]
pub struct SlotTable {
    slots: Box<[Option<*mut dyn Slot>]>,
    slots_len: usize,
    groups: Box<[Group]>,
    groups_len: usize,
    reader_count: usize,
    is_writing: bool,
}

impl SlotTable {
    pub fn is_empty(&self) -> bool {
        self.groups_len == 0
    }

    pub fn reader(&mut self) -> SlotReader {
        assert!(!self.is_writing);
        self.reader_count += 1;

        SlotReader {
            empty_count: 0,
            current_slot: 0,
            current_slot_end: 0,
        }
    }

    pub fn writer(&mut self) -> SlotWriter {
        assert!(!self.is_writing && self.reader_count == 0);
        self.is_writing = true;

        SlotWriter {
            current_slot: 0,
            current_slot_end: 0,
            slot_gap_start: self.slots_len,
            slot_gap_len: self.slots.len() - self.slots_len,
            insert_count: 0,
            parent: -1,
            group_gap_start: self.groups_len,
            group_gap_len: self.groups.len() / GROUP_FIELDS_SIZE - self.groups_len,
            current_group: 0,
            current_group_end: self.groups_len,
            end_stack: Vec::new(),
            node_count: 0,
            node_count_stack: Vec::new(),
        }
    }

    pub fn write(&mut self, f: impl FnOnce(&mut Self, &mut SlotWriter)) {
        let mut writer = self.writer();
        f(self, &mut writer);
        writer.close(self);
    }
}
