use crate::{
    slot_table::{Slot, SlotReader, SlotTable, SlotWriter},
    Composable, Compose,
};
use std::{
    any::TypeId,
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    mem,
};

struct ReuseKey;

enum GroupKind {
    Group,
    Node,
    ReusableNode,
}

#[doc(hidden)]
pub struct Composer {
    reader: SlotReader,
    writer: SlotWriter,
    is_inserting: bool,
    compound_key_hash: u64,
}

impl Composer {
    pub fn new() -> Self {
        let slot_table = SlotTable::default();
        let insert_table = SlotTable::default();
        Self {
            reader: slot_table.into_reader(),
            writer: insert_table.into_writer(),
            is_inserting: false,
            compound_key_hash: 0,
        }
    }

    pub fn compose(&mut self, content: impl Composable) {
        content.compose(self, 0);
    }

    pub fn apply_changes(&mut self) {
        self.reader = mem::take(&mut self.writer).close().into_reader();
    }

    /// Determine if the current slot table value is equal to the given value, if true, the value
    /// is scheduled to be skipped during [ControlledComposition.applyChanges] and [changes] return
    /// false; otherwise [ControlledComposition.applyChanges] will update the slot table to [value].
    /// In either case the composer's slot table is advanced.
    pub fn changed<T>(&mut self, value: &T) -> bool
    where
        T: Clone + Hash + PartialEq + 'static,
    {
        if self.next_slot().and_then(|slot| slot.any().downcast_ref()) == Some(value) {
            self.update_value(Some(Box::new(value.clone())));
            true
        } else {
            false
        }
    }

    fn update_value(&mut self, value: Option<Box<dyn Slot>>) {
        if self.is_inserting {
            self.writer.update(value);
            // TODO
        } else {
            todo!()
        }
    }

    fn next_slot(&mut self) -> Option<&dyn Slot> {
        if self.is_inserting {
            // validateNodeNotExpected()
            None
        } else {
            self.reader.next_slot()
        }
    }

    fn start(
        &mut self,
        id: TypeId,
        object_key: Option<Box<dyn Slot>>,
        _kind: GroupKind,
        data: Option<Box<dyn Slot>>,
    ) {
        self.update_compound_hash_key_on_enter_group(id, object_key.as_deref(), data.as_deref());
    }

    fn end(&mut self, _is_node: bool) {}

    fn update_compound_hash_key_on_enter_group(
        &mut self,
        id: TypeId,
        data_key: Option<&dyn Slot>,
        data: Option<&dyn Slot>,
    ) {
        if let Some(data_key) = data_key {
            let mut hasher = DefaultHasher::new();
            data_key.dyn_hash(&mut hasher);
            self.update_compound_hash_key_on_enter_group_with_key_hash(hasher.finish());
        } else {
            // TODO && id == ReuseKey.type_id()
            if let Some(data) = data {
                let mut hasher = DefaultHasher::new();
                data.dyn_hash(&mut hasher);
                self.update_compound_hash_key_on_enter_group_with_key_hash(hasher.finish())
            } else {
                let mut hasher = DefaultHasher::new();
                id.hash(&mut hasher);
                self.update_compound_hash_key_on_enter_group_with_key_hash(hasher.finish());
            }
        }
    }

    fn update_compound_hash_key_on_enter_group_with_key_hash(&mut self, key_hash: u64) {
        self.compound_key_hash = self.compound_key_hash.rotate_left(3) ^ key_hash;
    }
}

impl Compose for Composer {
    fn start_restart_group(&mut self, type_id: TypeId) {
        self.start(type_id, None, GroupKind::Group, None);
        // TODO add restart scope
    }

    fn end_restart_group(&mut self, _f: impl FnOnce() -> Box<dyn FnMut(&mut Self)>) {
        // TODO
        self.end(false)
    }

    fn start_replaceable_group(&mut self, type_id: TypeId) {
        self.start(type_id, None, GroupKind::Group, None)
    }

    fn end_replaceable_group(&mut self) {
        self.end(false)
    }

    fn is_skipping(&self) -> bool {
        false
    }

    fn skip_to_group_end(&mut self) {
        todo!()
    }

    fn cache<T>(&mut self, is_invalid: bool, f: impl FnOnce() -> T) -> T
    where
        T: Clone + Hash + PartialEq + 'static,
    {
        if let Some(slot) = self.next_slot() {
            if !is_invalid {
                return slot.any().downcast_ref::<T>().unwrap().clone();
            }
        }

        let value = f();
        self.update_value(Some(Box::new(value.clone())));
        value
    }
}
