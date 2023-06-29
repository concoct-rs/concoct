use crate::{slot_table::{SlotTable, SlotWriter}, Compose};

pub struct Composer {
    slot_table: SlotTable,
    insert_table: SlotTable,
    writer: SlotWriter,
    is_inserting: bool
}

impl Composer {
    pub fn new() -> Self {
        let mut insert_table = SlotTable::default();
        Self {
            slot_table: SlotTable::default(),
            writer: insert_table.writer(),
            insert_table,
            is_inserting: false
        }
    }

    fn next_slot(&mut self) {
        todo!()
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