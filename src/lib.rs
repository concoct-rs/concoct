use std::{
    any::{Any, TypeId},
    mem, ptr,
};

extern crate self as concoct;

pub use concoct_macros::composable;

pub trait Compose {
    fn start_restart_group(&mut self, type_id: TypeId);

    fn end_restart_group(&mut self, f: impl FnOnce() -> Box<dyn FnMut(&mut Self)>);

    fn start_replaceable_group(&mut self, type_id: TypeId);

    fn end_replaceable_group(&mut self);

    fn is_skipping(&self) -> bool;

    fn skip_to_group_end(&mut self);

    fn cache<T>(&mut self, is_invalid: bool, f: impl FnOnce() -> T) -> T;
}

pub trait Composable {
    type Output;

    fn compose(self, compose: &mut impl Compose, changed: u32) -> Self::Output;
}

#[macro_export]
macro_rules! compose {
    ($composable:expr) => {
        $composable
    };
}

// TODO
#[macro_export]
macro_rules! current_composer {
    () => {};
}

#[composable]
pub fn remember<T: 'static, F: FnOnce() -> T + 'static>(f: F) -> T {
    composer.cache(false, f)
}

pub trait Slot {
    fn any(&self) -> &dyn Any;

    fn any_eq(&self, other: &dyn Any) -> bool;
}

pub struct SlotTable {
    slots: Box<[Option<*mut dyn Slot>]>,
    slots_len: usize,
    is_writing: bool,
}

impl SlotTable {
    pub fn writer(&mut self) -> SlotWriter {
        assert!(!self.is_writing);
        self.is_writing = true;

        SlotWriter {
            current_slot: 0,
            current_slot_end: 0,
            slot_gap_start: self.slots_len,
            slot_gap_len: self.slots.len() - self.slots_len,
        }
    }
}

pub struct SlotWriter {
    current_slot: usize,
    current_slot_end: usize,
    slot_gap_start: usize,
    slot_gap_len: usize,
}

impl SlotWriter {
    /// Set the value at the groups current data slot
    pub fn set(
        &mut self,
        table: &mut SlotTable,
        value: Option<Box<dyn Slot>>,
    ) -> Option<Box<dyn Slot>> {
        assert!(self.current_slot <= self.current_slot_end);

        let slot = value.map(|slot| Box::into_raw(slot));

        let ptr = mem::replace(
            &mut table.slots[self.data_index_to_data_address(self.current_slot - 1)],
            slot,
        );

        ptr.map(|ptr| unsafe { Box::from_raw(ptr) })
    }

    fn data_index_to_data_address(&self, dataIndex: usize) -> usize {
        if dataIndex < self.slot_gap_start {
            dataIndex
        } else {
            dataIndex + self.slot_gap_len
        }
    }
}
