use std::{
    any::{Any, TypeId},
    mem,
};

const GROUP_FIELDS_SIZE: usize = 1;

pub trait Slot {
    fn any(&self) -> &dyn Any;

    fn any_eq(&self, other: &dyn Any) -> bool;
}

const NODE_COUNT_MASK: u32 = 0b0000_0011_1111_1111__1111_1111_1111_1111;

#[derive(Clone, Copy)]
pub struct Group {
    id: TypeId,
    mask: u32,
    parent_anchor: usize,
    size_offset: usize,
    data_anchor: usize,
}

impl Group {
    pub fn empty() -> Self {
        Self {
            id: TypeId::of::<()>(),
            mask: 0,
            parent_anchor: 0,
            size_offset: 0,
            data_anchor: 0,
        }
    }

    pub fn new(
        id: TypeId,
        is_node: bool,
        has_data_key: bool,
        has_data: bool,
        parent_anchor: usize,
        data_anchor: usize,
    ) -> Self {
        const NODE_BIT_MASK: u32 = 0b0100_0000_0000_0000__0000_0000_0000_0000;
        const OBJECT_KEY_MASK: u32 = 0b0010_0000_0000_0000__0000_0000_0000_0000;
        const AUX_MASK: u32 = 0b0001_0000_0000_0000__0000_0000_0000_0000;

        let node_bit = if is_node { NODE_BIT_MASK } else { 0 };
        let data_key_bit = if has_data_key { OBJECT_KEY_MASK } else { 0 };
        let data_bit = if has_data { AUX_MASK } else { 0 };

        Self {
            id,
            mask: node_bit | data_key_bit | data_bit,
            parent_anchor,
            size_offset: 0,
            data_anchor,
        }
    }

    pub fn node_count(&self) -> u32 {
        self.mask & NODE_COUNT_MASK
    }

    pub fn set_node_count(&mut self, value: u32) {
        assert!(value < NODE_COUNT_MASK);
        self.mask &= !NODE_COUNT_MASK | value
    }
}

#[derive(Default)]
pub struct SlotTable {
    slots: Box<[Option<*mut dyn Slot>]>,
    slots_len: usize,
    groups: Box<[Group]>,
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
            group_gap_start: self.groups_len,
            group_gap_len: self.groups.len() / GROUP_FIELDS_SIZE - self.groups_len,
            current_group: 0,
            current_group_end: self.groups_len,
            end_stack: Vec::new(),
            node_count: 0,
            node_count_stack: Vec::new(),
        }
    }

    pub fn write(&mut self, f: impl FnOnce(&mut Self, &mut SlotWriter)) {
        let mut writer = self.writer();
        f(self, &mut writer);
        writer.close(self);
    }
}

pub struct SlotWriter {
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

impl SlotWriter {
    pub fn close( self, table: &mut SlotTable) {
       table.groups_len = self.group_gap_start;
       table.slots_len = self.slot_gap_start;
    }


    /// Begin inserting at the current location. beginInsert() can be nested and must be called with
    /// a balanced number of endInsert()
    pub fn begin_insert(&mut self, table: &mut SlotTable) {
        let count = self.insert_count;
        self.insert_count += 1;
        if count == 0 {
            self.save_current_group_end(table)
        }
    }

    /// Ends inserting.
    pub fn end_insert(&mut self, table: &SlotTable) {
        assert!(self.insert_count > 0);

        self.insert_count -= 1;
        if self.insert_count == 0 {
            //assert!(nodeCountStack.size == startStack.size);

            self.restore_current_group_end(table);
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
        table.groups.len() / GROUP_FIELDS_SIZE
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

    /// Restore [currentGroupEnd] from [endStack].
    fn restore_current_group_end(&mut self, table: &SlotTable) -> usize {
        self.current_group_end =
            (self.capacity(table) - self.group_gap_len) - self.end_stack.pop().unwrap();
        self.current_group_end
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

    pub fn start_group(
        &mut self,
        table: &mut SlotTable,
        id: TypeId,
        data_key: Option<Box<dyn Slot>>,
    ) {
        self.start_group_inner(table, id, data_key, false, None)
    }

    fn start_group_inner(
        &mut self,
        table: &mut SlotTable,
        id: TypeId,
        object_key: Option<Box<dyn Slot>>,
        is_node: bool,
        aux: Option<Box<dyn Slot>>,
    ) {
        self.node_count_stack.push(self.node_count);

        self.current_group_end = if self.insert_count > 0 {
            self.insert_groups(table, 1);

            let has_aux = !is_node && aux.is_some();

            table.groups[self.group_index_to_address(self.current_group)] = Group::new(
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
                self.insert_slots(table, data_slots_needed, self.current_group as _);
                let aux = aux.map(Box::into_raw);
                if is_node {
                    let idx = self.current_slot;
                    self.current_slot += 1;
                    table.slots[idx] = aux;
                }
                if let Some(key) = object_key {
                    let idx = self.current_slot;
                    self.current_slot += 1;
                    table.slots[idx] = Some(Box::into_raw(key));
                }
                if has_aux {
                    let idx = self.current_slot;
                    self.current_slot += 1;
                    table.slots[idx] = aux;
                }
            }

            self.current_group_end + 1
        } else {
            todo!()
        }
    }

    /// Insert [size] number of groups in front of [currentGroup]. These groups are implicitly a
    /// child of [parent].
    fn insert_groups(&mut self, table: &mut SlotTable, size: usize) {
        if size == 0 {
            return;
        }

        self.move_group_gap_to(table, self.current_group);

        let old_capacity = table.groups.len() / GROUP_FIELDS_SIZE;
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
            new_groups[..len].copy_from_slice(&table.groups[0..len]);

            let offset = new_gap_end_address * GROUP_FIELDS_SIZE;
            let start = old_gap_end_address * GROUP_FIELDS_SIZE;
            let end = old_capacity * GROUP_FIELDS_SIZE;
            let len = end - start;
            new_groups[offset..offset + len].copy_from_slice(&table.groups[start..end]);

            // Update the gap and slots
            table.groups = new_groups.into_boxed_slice();
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
    fn move_group_gap_to(&mut self, table: &mut SlotTable, index: usize) {
        if self.group_gap_start == index {
            return;
        }

        if self.group_gap_len > 0 {
            // Here physical is used to mean an index of the actual first int of the group in the
            // array as opposed ot the logical address which is in groups of Group_Field_Size
            // integers. IntArray.copyInto expects physical indexes.
            let groupPhysicalAddress = index * GROUP_FIELDS_SIZE;
            let groupPhysicalGapLen = self.group_gap_len * GROUP_FIELDS_SIZE;
            let groupPhysicalGapStart = self.slot_gap_start * GROUP_FIELDS_SIZE;

            if index < self.slot_gap_start {
                table.groups.copy_within(
                    groupPhysicalAddress..groupPhysicalGapStart,
                    groupPhysicalAddress + groupPhysicalGapLen,
                );
            } else {
                table.groups.copy_within(
                    groupPhysicalGapStart + groupPhysicalGapLen
                        ..groupPhysicalAddress + groupPhysicalGapLen,
                    groupPhysicalGapStart,
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
}
