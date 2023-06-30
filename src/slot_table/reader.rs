use super::{Slot, SlotTable};

pub struct SlotReader {
    table: SlotTable,
   empty_count: usize,
   current_slot: usize,
   current_slot_end: usize,
}

impl SlotReader{
    pub fn new(table: SlotTable) -> Self {
        Self {
            empty_count: 0,
            current_slot: 0,
            current_slot_end: 0,
            table
        }
    }

    pub fn next(&mut self) -> Option<&dyn Slot> {
        if self.empty_count > 0 || self.current_slot >= self.current_slot_end {
            None
        } else {
            let idx = self.current_slot; 
            self.current_slot += 1;
            self.table.slots[idx].map(|ptr| unsafe { ptr.as_ref().unwrap() })
        }
    }
}
