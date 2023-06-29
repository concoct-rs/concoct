use crate::{
    slot_table::{Slot, SlotReader, SlotTable, SlotWriter},
    Compose,
};

pub struct Composer {
    slot_table: SlotTable,
    insert_table: SlotTable,
    reader: SlotReader,
    writer: SlotWriter,
    is_inserting: bool,
}

impl Composer {
    pub fn new() -> Self {
        let mut slot_table = SlotTable::default();
        let mut insert_table = SlotTable::default();
        Self {
            reader: slot_table.reader(),
            writer: insert_table.writer(),
            slot_table,
            insert_table,
            is_inserting: false,
        }
    }

    /// Determine if the current slot table value is equal to the given value, if true, the value
    /// is scheduled to be skipped during [ControlledComposition.applyChanges] and [changes] return
    /// false; otherwise [ControlledComposition.applyChanges] will update the slot table to [value].
    /// In either case the composer's slot table is advanced.
    pub fn changed<T>(&mut self, value: &T) -> bool
    where
        T: Clone + PartialEq + 'static,
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
            self.writer.update(&mut self.insert_table, value);
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
            self.reader.next(&mut self.slot_table)
        }
    }
}

impl Compose for Composer {
    fn start_restart_group(&mut self, type_id: std::any::TypeId) {
        todo!()
    }

    fn end_restart_group(&mut self, f: impl FnOnce() -> Box<dyn FnMut(&mut Self)>) {
        todo!()
    }

    fn start_replaceable_group(&mut self, type_id: std::any::TypeId) {
        todo!()
    }

    fn end_replaceable_group(&mut self) {
        todo!()
    }

    fn is_skipping(&self) -> bool {
        todo!()
    }

    fn skip_to_group_end(&mut self) {
        todo!()
    }

    fn cache<T>(&mut self, is_invalid: bool, f: impl FnOnce() -> T) -> T {
        todo!()
    }
}
