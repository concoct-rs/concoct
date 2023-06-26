use std::any::Any;

pub struct Anchor {}

pub struct SlotTable {
    // An array to store group information that is stored as groups of [Group_Fields_Size]
    // elements of the array. The [groups] array can be thought of as an array of an inline struct.
    groups: Box<i32>,

    // The number of groups contained in [groups].
    groups_len: usize,

    // An array that stores the slots for a group. The slot elements for a group start at the
    // offset returned by [dataAnchor] of [groups] and continue to the next group's slots or to
    // [slotsSize] for the last group. When in a writer the [dataAnchor] is an anchor instead of
    // an index as [slots] might contain a gap.
    slots: Box<[Option<Box<dyn Any>>]>,

    // The number of slots used in [slots].
    slots_len: usize,

    // Tracks the number of active readers. A SlotTable can have multiple readers but only one writer.
    readers: usize,

    // Tracks whether there is an active writer.
    is_writing: bool,

    // An internal version that is incremented whenever a writer is created. This is used to
    // detect when an iterator created by [CompositionData] is invalid.
    version: usize,

    // A list of currently active anchors.
    anchors: Vec<Anchor>,
}
