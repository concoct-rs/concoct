use std::{
    any::{Any, TypeId},
    hash::{Hash, Hasher},
};

mod reader;
pub use reader::SlotReader;

mod writer;
pub use writer::SlotWriter;

const GROUP_FIELDS_SIZE: usize = 1;

pub trait Slot {
    fn any(&self) -> &dyn Any;

    fn any_eq(&self, other: &dyn Any) -> bool;

    fn dyn_hash(&self, state: &mut dyn Hasher);
}

impl<T: Hash + PartialEq + 'static> Slot for T {
    fn any(&self) -> &dyn Any {
        todo!()
    }

    fn any_eq(&self, other: &dyn Any) -> bool {
        Some(self) == other.downcast_ref::<T>()
    }

    fn dyn_hash(&self, mut state: &mut dyn Hasher) {
        self.hash(&mut state)
    }
}

const NODE_COUNT_MASK: u32 = 0b0000_0011_1111_1111__1111_1111_1111_1111;
const NODE_BIT_MASK: u32 = 0b0100_0000_0000_0000__0000_0000_0000_0000;

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

    pub fn is_node(&self) -> bool {
        self.mask & NODE_BIT_MASK != 0
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
}

impl SlotTable {
    pub fn is_empty(&self) -> bool {
        self.groups_len == 0
    }

    pub fn into_reader(self) -> SlotReader {
        SlotReader::new(self)
    }

    pub fn into_writer(self) -> SlotWriter {
        SlotWriter::new(self)
    }

    pub fn write(self, f: impl FnOnce(&mut SlotWriter)) -> Self {
        let mut writer = self.into_writer();
        f(&mut writer);
        writer.close()
    }
}

impl Drop for SlotTable {
    fn drop(&mut self) {
        for slot in self.slots.into_iter().copied() {
            if let Some(ptr) = slot {
                drop(unsafe { Box::from_raw(ptr) });
            }
        }
    }
}
