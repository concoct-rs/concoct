use std::{any::Any, mem};

const Group_Fields_Size: usize = 5;

pub trait Slot {
    fn any(&self) -> &dyn Any;

    fn any_eq(&self, other: &dyn Any) -> bool;
}

#[derive(Default)]
pub struct SlotTable {
    slots: Box<[Option<*mut dyn Slot>]>,
    slots_len: usize,
    groups: Vec<usize>,
    groups_len: usize,
    is_writing: bool,
}

impl SlotTable {
    pub fn is_empty(&self) -> bool {
        self.groups_len == 0
    }

    pub fn writer(&mut self) -> SlotWriter {
        assert!(!self.is_writing);
        self.is_writing = true;

        SlotWriter {
            current_slot: 0,
            current_slot_end: 0,
            slot_gap_start: self.slots_len,
            slot_gap_len: self.slots.len() - self.slots_len,
            insert_count: 0,
            parent: -1,
            group_gap_len: self.groups.len() / Group_Fields_Size - self.groups_len,
            current_group_end: self.groups_len,
            end_stack: Vec::new(),
        }
    }

    pub fn write(&mut self, f: impl FnOnce(&mut Self, &mut SlotWriter)) {
        let mut writer = self.writer();
        f(self, &mut writer);
    }
}

pub struct SlotWriter {
    current_slot: usize,
    current_slot_end: usize,
    slot_gap_start: usize,
    slot_gap_len: usize,
    insert_count: usize,
    parent: i32,
    group_gap_len: usize,
    current_group_end: usize,
    end_stack: Vec<usize>,
}

impl SlotWriter {
    /// Begin inserting at the current location. beginInsert() can be nested and must be called with
    /// a balanced number of endInsert()
    pub fn begin_insert(&mut self, table: &mut SlotTable) {
        let count = self.insert_count;
        self.insert_count += 1;
        if count == 0 {
            self.save_current_group_end(table)
        }
    }

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

    /// Skip the current slot without updating. If the slot table is inserting then and
    /// [Composer.Empty] slot is added and [skip] return [Composer.Empty].
    pub fn skip(&mut self, table: &mut SlotTable) -> Option<&mut dyn Slot> {
        let idx = self.skip_inner(table);
        table.slots[idx].map(|ptr| unsafe { ptr.as_mut().unwrap() })
    }

    /// Set the value of the next slot. Returns the previous value of the slot or [Composer.Empty]
    /// is being inserted.
    pub fn update(
        &mut self,
        table: &mut SlotTable,
        value: Option<Box<dyn Slot>>,
    ) -> Option<&mut dyn Slot> {
        let idx = self.skip_inner(table);
        self.set(table, value);
        table.slots[idx].map(|ptr| unsafe { &mut *ptr })
    }

    fn data_index_to_data_address(&self, data_index: usize) -> usize {
        if data_index < self.slot_gap_start {
            data_index
        } else {
            data_index + self.slot_gap_len
        }
    }

    fn capacity(&self, table: &SlotTable) -> usize {
        table.groups.len() / Group_Fields_Size
    }

    /// Insert room into the slot table. This is performed by first moving the gap to [currentSlot]
    /// and then reducing the gap [size] slots. If the gap is smaller than [size] the gap is grown
    /// to at least accommodate [size] slots. The new slots are associated with [group].
    fn insert_slots(&mut self, table: &mut SlotTable, size: usize, group: i32) {
        if size == 0 {
            return;
        }

        self.move_slot_gap_to(table, self.current_slot, group);

        if self.slot_gap_len < size {
            // Create a bigger gap
            let old_capacity = table.slots.len();
            let old_size = old_capacity - self.slot_gap_len;

            // Double the size of the array, but at least MinGrowthSize and >= size
            const MIN_SLOTS_GROWTH_SIZE: usize = 32;
            let new_capacity = MIN_SLOTS_GROWTH_SIZE
                .max(old_capacity * 2)
                .max(old_size + size);

            let mut new_data = vec![None; new_capacity];
            let new_gap_len = new_capacity - old_size;
            let old_gap_end_address = self.slot_gap_start + self.slot_gap_len;
            let new_gap_end_address = self.slot_gap_start + new_gap_len;

            // Copy the old arrays into the new arrays
            new_data[..self.slot_gap_start].copy_from_slice(&table.slots[..self.slot_gap_start]);

            let len = old_capacity - old_gap_end_address;
            new_data[new_gap_end_address..new_gap_end_address + len]
                .copy_from_slice(&table.slots[old_gap_end_address..old_capacity]);

            // Update the gap and slots
            table.slots = new_data.into_boxed_slice();
            self.slot_gap_len = new_gap_len;
        }

        if self.current_slot_end >= self.slot_gap_start {
            self.current_slot_end += size;
        }
        self.slot_gap_start = self.slot_gap_start + size;
        self.slot_gap_len = self.slot_gap_len - size;
    }

    /// Move the gap in [slots] to [index] where [group] is expected to receive any new slots added.
    fn move_slot_gap_to(&mut self, table: &mut SlotTable, index: usize, group: i32) {
        if self.slot_gap_start != index {
            if index < self.slot_gap_start {
                // Move the gap down to index by shifting the data up.
                table
                    .slots
                    .copy_within(index..self.slot_gap_start, index + self.slot_gap_len)
            } else {
                // Shift the data down, leaving the gap at index
                table.slots.copy_within(
                    self.slot_gap_start + self.slot_gap_len..index + self.slot_gap_len,
                    self.slot_gap_start,
                );
            }

            // TODO
            // Update the data anchors affected by the move

            self.slot_gap_start = index;
        }
    }

    /// Save [currentGroupEnd] to [endStack].
    fn save_current_group_end(&mut self, table: &mut SlotTable) {
        // Record the end location as relative to the end of the slot table so when we pop it
        // back off again all inserts and removes that happened while a child group was open
        // are already reflected into its value.
        self.end_stack
            .push(self.capacity(table) - self.group_gap_len - self.current_group_end)
    }

    fn skip_inner(&mut self, table: &mut SlotTable) -> usize {
        if self.insert_count > 0 {
            self.insert_slots(table, 1, self.parent)
        }
        let idx = self.current_slot;
        self.current_slot += 1;

        self.data_index_to_data_address(idx)
    }
}
