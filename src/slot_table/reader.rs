use super::{Slot, SlotTable};

/// Reader for a [`SlotTable`].
pub struct SlotReader {
    /// The current slot table being read
    table: SlotTable,

    /// The number of times `begin_empty` has been called.
    empty_count: usize,

    /// The current slot of `parent`.
    /// This slot will be the next slot returned by `next_slot` unless it is equal ot [currentSlotEnd].
    current_slot: usize,

    /// The current end slot of `parent`.
    current_slot_end: usize,
}

impl SlotReader {
    /// Create a new slot reader from the given [`SlotTable`].
    pub fn new(table: SlotTable) -> Self {
        Self {
            empty_count: 0,
            current_slot: 0,
            current_slot_end: 0,
            table,
        }
    }

    /// Get the value of the slot at `current_group` or `None` if at then end of a group.
    /// During empty mode this value is always `None` which is the value a newly inserted slot.
    pub fn next_slot(&mut self) -> Option<&dyn Slot> {
        if self.empty_count > 0 || self.current_slot >= self.current_slot_end {
            None
        } else {
            let idx = self.current_slot;
            self.current_slot += 1;
            self.table.slots[idx].map(|ptr| unsafe { ptr.as_ref().unwrap() })
        }
    }
}
