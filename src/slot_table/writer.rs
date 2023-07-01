use super::{Group, Slot, SlotTable, GROUP_FIELDS_SIZE};
use std::{any::TypeId, mem};

pub struct SlotWriter {
    table: SlotTable,

    /// The location of the `slots` array that contains the data for the [parent] group.
    current_slot: usize,
    current_slot_end: usize,
    slot_gap_start: usize,
    slot_gap_len: usize,
    insert_count: usize,
    parent: i32,
    group_gap_start: usize,
    group_gap_len: usize,
    current_group: usize,
    current_group_end: usize,
    end_stack: Vec<usize>,
    node_count: usize,
    node_count_stack: Vec<usize>,
}

impl Default for SlotWriter {
    fn default() -> Self {
        Self::new(SlotTable::default())
    }
}

impl SlotWriter {
    pub fn new(table: SlotTable) -> Self {
        Self {
            current_slot: 0,
            current_slot_end: 0,
            slot_gap_start: table.slots_len,
            slot_gap_len: table.slots.len() - table.slots_len,
            insert_count: 0,
            parent: -1,
            group_gap_start: table.groups_len,
            group_gap_len: table.groups.len() / GROUP_FIELDS_SIZE - table.groups_len,
            current_group: 0,
            current_group_end: table.groups_len,
            end_stack: Vec::new(),
            node_count: 0,
            node_count_stack: Vec::new(),
            table,
        }
    }

    pub fn close(mut self) -> SlotTable {
        self.table.groups_len = self.group_gap_start;
        self.table.slots_len = self.slot_gap_start;
        self.table
    }

    /// Begin inserting at the current location. beginInsert() can be nested and must be called with
    /// a balanced number of endInsert()
    pub fn begin_insert(&mut self) {
        let count = self.insert_count;
        self.insert_count += 1;
        if count == 0 {
            self.save_current_group_end()
        }
    }

    /// Ends inserting.
    pub fn end_insert(&mut self) {
        assert!(self.insert_count > 0);

        self.insert_count -= 1;
        if self.insert_count == 0 {
            //assert!(nodeCountStack.size == startStack.size);

            self.restore_current_group_end();
        }
    }

    /// Set the value at the groups current data slot
    pub fn set(&mut self, value: Option<Box<dyn Slot>>) -> Option<Box<dyn Slot>> {
        assert!(self.current_slot <= self.current_slot_end);

        let slot = value.map(|slot| Box::into_raw(slot));
        let ptr = mem::replace(
            &mut self.table.slots[self.data_index_to_data_address(self.current_slot - 1)],
            slot,
        );

        ptr.map(|ptr| unsafe { Box::from_raw(ptr) })
    }

    /// Skip the current slot without updating. If the slot table is inserting then and
    /// [Composer.Empty] slot is added and [skip] return [Composer.Empty].
    pub fn skip(&mut self, _table: &mut SlotTable) -> Option<&mut dyn Slot> {
        let idx = self.skip_inner();
        self.table.slots[idx].map(|ptr| unsafe { ptr.as_mut().unwrap() })
    }

    /// Set the value of the next slot. Returns the previous value of the slot or [Composer.Empty]
    /// is being inserted.
    pub fn update(&mut self, value: Option<Box<dyn Slot>>) -> Option<&mut dyn Slot> {
        let idx = self.skip_inner();
        self.set(value);
        self.table.slots[idx].map(|ptr| unsafe { &mut *ptr })
    }

    fn data_index_to_data_address(&self, data_index: usize) -> usize {
        if data_index < self.slot_gap_start {
            data_index
        } else {
            data_index + self.slot_gap_len
        }
    }

    fn capacity(&self) -> usize {
        self.table.groups.len() / GROUP_FIELDS_SIZE
    }

    /// Insert room into the slot self.table. This is performed by first moving the gap to [currentSlot]
    /// and then reducing the gap [size] slots. If the gap is smaller than [size] the gap is grown
    /// to at least accommodate [size] slots. The new slots are associated with [group].
    fn insert_slots(&mut self, size: usize, group: i32) {
        if size == 0 {
            return;
        }

        self.move_slot_gap_to(self.current_slot, group);

        if self.slot_gap_len < size {
            // Create a bigger gap
            let old_capacity = self.table.slots.len();
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
            new_data[..self.slot_gap_start]
                .copy_from_slice(&self.table.slots[..self.slot_gap_start]);

            let len = old_capacity - old_gap_end_address;
            new_data[new_gap_end_address..new_gap_end_address + len]
                .copy_from_slice(&self.table.slots[old_gap_end_address..old_capacity]);

            // Update the gap and slots
            self.table.slots = new_data.into_boxed_slice();
            self.slot_gap_len = new_gap_len;
        }

        if self.current_slot_end >= self.slot_gap_start {
            self.current_slot_end += size;
        }
        self.slot_gap_start = self.slot_gap_start + size;
        self.slot_gap_len = self.slot_gap_len - size;
    }

    /// Move the gap in [slots] to [index] where [group] is expected to receive any new slots added.
    fn move_slot_gap_to(&mut self, index: usize, _group: i32) {
        if self.slot_gap_start != index {
            if index < self.slot_gap_start {
                // Move the gap down to index by shifting the data up.
                self.table
                    .slots
                    .copy_within(index..self.slot_gap_start, index + self.slot_gap_len)
            } else {
                // Shift the data down, leaving the gap at index
                self.table.slots.copy_within(
                    self.slot_gap_start + self.slot_gap_len..index + self.slot_gap_len,
                    self.slot_gap_start,
                );
            }

            // TODO
            // Update the data anchors affected by the move

            self.slot_gap_start = index;
        }
    }

    /// Restore [currentGroupEnd] from [endStack].
    fn restore_current_group_end(&mut self) -> usize {
        self.current_group_end =
            (self.capacity() - self.group_gap_len) - self.end_stack.pop().unwrap();
        self.current_group_end
    }

    /// Save [currentGroupEnd] to [endStack].
    fn save_current_group_end(&mut self) {
        // Record the end location as relative to the end of the slot table so when we pop it
        // back off again all inserts and removes that happened while a child group was open
        // are already reflected into its value.
        self.end_stack
            .push(self.capacity() - self.group_gap_len - self.current_group_end)
    }

    fn skip_inner(&mut self) -> usize {
        if self.insert_count > 0 {
            self.insert_slots(1, self.parent)
        }
        let idx = self.current_slot;
        self.current_slot += 1;

        self.data_index_to_data_address(idx)
    }

    pub fn start_group(&mut self, id: TypeId, data_key: Option<Box<dyn Slot>>) {
        self.start_group_inner(id, data_key, false, None)
    }

    fn start_group_inner(
        &mut self,
        id: TypeId,
        object_key: Option<Box<dyn Slot>>,
        is_node: bool,
        aux: Option<Box<dyn Slot>>,
    ) {
        self.node_count_stack.push(self.node_count);

        self.current_group_end = if self.insert_count > 0 {
            self.insert_groups(1);

            let has_aux = !is_node && aux.is_some();

            self.table.groups[self.group_index_to_address(self.current_group)] = Group::new(
                id,
                is_node,
                object_key.is_some(),
                !is_node && aux.is_some(),
                self.parent as usize,
                self.current_slot,
            );
            self.current_slot_end = self.current_slot;

            let data_slots_needed = (if is_node { 1 } else { 0 })
                + (if object_key.is_some() { 1 } else { 0 })
                + (if has_aux { 1 } else { 0 });

            if data_slots_needed > 0 {
                self.insert_slots(data_slots_needed, self.current_group as _);
                let aux = aux.map(Box::into_raw);
                if is_node {
                    let idx = self.current_slot;
                    self.current_slot += 1;
                    self.table.slots[idx] = aux;
                }
                if let Some(key) = object_key {
                    let idx = self.current_slot;
                    self.current_slot += 1;
                    self.table.slots[idx] = Some(Box::into_raw(key));
                }
                if has_aux {
                    let idx = self.current_slot;
                    self.current_slot += 1;
                    self.table.slots[idx] = aux;
                }
            }

            self.node_count = 0;
            self.parent = self.current_group as _;
            self.current_group += 1;

            self.current_group_end + 1
        } else {
            todo!()
        }
    }

    /// Insert [size] number of groups in front of [currentGroup]. These groups are implicitly a
    /// child of [parent].
    fn insert_groups(&mut self, size: usize) {
        if size == 0 {
            return;
        }

        self.move_group_gap_to(self.current_group);

        let old_capacity = self.table.groups.len() / GROUP_FIELDS_SIZE;
        let old_size = old_capacity - self.group_gap_len;
        if self.group_gap_len < size {
            // Create a bigger gap
            // Double the size of the array, but at least MinGrowthSize and >= size
            const MIN_GROUP_GROWTH_SIZE: usize = 32;
            let new_capacity = MIN_GROUP_GROWTH_SIZE
                .max(old_capacity * 2)
                .max(old_size * size);

            let mut new_groups = vec![Group::empty(); new_capacity * GROUP_FIELDS_SIZE];
            let new_gap_len = new_capacity - old_size;
            let old_gap_end_address = self.group_gap_start + self.group_gap_len;
            let new_gap_end_address = self.group_gap_start + new_gap_len;

            // Copy the old arrays into the new arrays
            let len = self.slot_gap_start * GROUP_FIELDS_SIZE;
            new_groups[..len].copy_from_slice(&self.table.groups[0..len]);

            let offset = new_gap_end_address * GROUP_FIELDS_SIZE;
            let start = old_gap_end_address * GROUP_FIELDS_SIZE;
            let end = old_capacity * GROUP_FIELDS_SIZE;
            let len = end - start;
            new_groups[offset..offset + len].copy_from_slice(&self.table.groups[start..end]);

            // Update the gap and slots
            self.table.groups = new_groups.into_boxed_slice();
            self.group_gap_len = new_gap_len;
        }

        // Move the currentGroupEnd to account for inserted groups.

        if self.current_group_end >= self.group_gap_start {
            self.current_group_end += size
        }

        // Update the gap start and length
        self.group_gap_start += size;
        self.group_gap_len -= size;

        // TODO
    }

    /**
     * Move the gap in [groups] to [index].
     */
    fn move_group_gap_to(&mut self, index: usize) {
        if self.group_gap_start == index {
            return;
        }

        if self.group_gap_len > 0 {
            // Here physical is used to mean an index of the actual first int of the group in the
            // array as opposed ot the logical address which is in groups of Group_Field_Size
            // integers. IntArray.copyInto expects physical indexes.
            let group_physical_address = index * GROUP_FIELDS_SIZE;
            let group_physical_gap_len = self.group_gap_len * GROUP_FIELDS_SIZE;
            let group_physical_gap_start = self.slot_gap_start * GROUP_FIELDS_SIZE;

            if index < self.slot_gap_start {
                self.table.groups.copy_within(
                    group_physical_address..group_physical_gap_start,
                    group_physical_address + group_physical_gap_len,
                );
            } else {
                self.table.groups.copy_within(
                    group_physical_gap_start + group_physical_gap_len
                        ..group_physical_address + group_physical_gap_len,
                    group_physical_gap_start,
                );
            }
        }

        // TODO
        // Gap has moved so the anchor for the groups that moved have changed so the parent
        // anchors that refer to these groups must be updated.

        self.group_gap_start = index;
    }

    fn group_index_to_address(&self, index: usize) -> usize {
        if index < self.group_gap_start {
            index
        } else {
            index + self.group_gap_len
        }
    }

    /// End the current group. Must be called after the corresponding startGroup().
    pub fn end_group(&mut self) -> usize {
        let group_address = self.group_index_to_address(self.parent as usize);

        let new_nodes = self.node_count;
        let new_group_size = self.current_group - self.parent as usize;
        let is_node = self.table.groups[group_address].is_node();

        if self.insert_count > 0 {
            let group = &mut self.table.groups[group_address];
            group.size_offset = new_group_size;
            group.set_node_count(self.node_count as _);
            self.node_count =
                self.node_count_stack.pop().unwrap() + if is_node { 1 } else { self.node_count };
            // TODO parent
        }

        new_nodes
    }
}