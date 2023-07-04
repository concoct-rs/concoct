use std::{
    iter,
    mem::{self, MaybeUninit},
};

use super::slot::{Slot, SlotKind};

// TODO this exists to split borrows
pub struct SlotTable {
    slots: Box<[MaybeUninit<Slot>]>,
    gap_start: usize,
    gap_end: usize,
    capacity: usize,
    pub pos: usize,
}

impl SlotTable {
    pub fn new() -> Self {
        Self::with_capacity(32)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            slots: new_slots(capacity),
            gap_start: 0,
            gap_end: capacity,
            capacity,
            pos: 0,
        }
    }

    /// Get the slot at `index`.
    pub fn get(&self, index: usize) -> Option<&Slot> {
        self.get_address(index)
            .map(|addr| unsafe { self.slots[addr].assume_init_ref() })
    }

    /// Get the slot at `index`.
    pub fn get_mut(&mut self, index: usize) -> Option<&mut Slot> {
        self.get_address(index)
            .map(|addr| unsafe { self.slots[addr].assume_init_mut() })
    }

    pub fn get_address(&self, index: usize) -> Option<usize> {
        let addr = if index >= self.gap_start && index < self.gap_end {
            self.gap_end
        } else {
            index
        };

        if addr < self.slots.len() {
            Some(addr)
        } else {
            None
        }
    }

    pub fn peek_mut(&mut self) -> Option<&mut Slot> {
        self.get_mut(self.pos)
    }

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
    }

    /// Start a new iterator over the slots inside this composer.
    pub fn slots(&self) -> impl Iterator<Item = &Slot> {
        let mut pos = 0;
        iter::from_fn(move || {
            if let Some(slot) = self.get(pos) {
                pos += 1;
                Some(slot)
            } else {
                None
            }
        })
    }

    pub fn slot_kinds(&self) -> impl Iterator<Item = SlotKind> + '_ {
        self.slots().map(|slot| slot.kind())
    }
}

fn new_slots(capacity: usize) -> Box<[MaybeUninit<Slot>]> {
    Vec::from_iter(iter::repeat_with(|| MaybeUninit::uninit()).take(capacity)).into_boxed_slice()
}
