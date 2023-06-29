use super::{Slot, SlotTable};

pub struct SlotReader {
    pub(super) empty_count: usize,
    pub(super) current_slot: usize,
    pub(super) current_slot_end: usize,
}

impl SlotReader {
    pub fn next(&mut self, table: &mut SlotTable) -> Option<&dyn Slot> {
        if self.empty_count > 0 || self.current_slot >= self.current_slot_end {
            None
        } else {
            let idx = self.current_slot;
            self.current_slot += 1;
            table.slots[idx].map(|ptr| unsafe { ptr.as_ref().unwrap() })
        }
    }
}
