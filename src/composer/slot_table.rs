use std::{mem::{self, MaybeUninit}, iter};

use super::Slot;

// TODO this exists to split borrows
pub struct SlotTable {
    slots: Box<[MaybeUninit<Slot>]>,
    gap_start: usize,
    gap_end: usize,
    capacity: usize,
    pos: usize,
    child_count: usize,
}

impl SlotTable {
    /// Insert a slot into the current position.
    pub fn insert(&mut self, slot: Slot) {
        if self.pos != self.gap_start {}

        // Check if we're out of space
        if self.gap_start == self.gap_end {
            // Double the capacity, to a minimum of 32
            self.capacity = (self.capacity * 2).max(32);
            let mut slots = new_slots(self.capacity);

            // Move slots from the old table
            for idx in 0..self.gap_start {
                slots[idx] = mem::replace(&mut self.slots[idx], MaybeUninit::uninit());
            }
            for idx in self.gap_end..self.slots.len() {
                slots[idx + self.gap_start] =
                    mem::replace(&mut self.slots[idx], MaybeUninit::uninit());
            }

            // Update the state
            self.gap_start = self.slots.len();
            self.gap_end = slots.len();
            self.slots = slots;
        }

        // Insert the new slot
        self.slots[self.pos] = MaybeUninit::new(slot);
        self.pos += 1;
        self.gap_start += 1;
        self.child_count += 1;
    }
}

fn new_slots(capacity: usize) -> Box<[MaybeUninit<Slot>]> {
    Vec::from_iter(iter::repeat_with(|| MaybeUninit::uninit()).take(capacity)).into_boxed_slice()
}
