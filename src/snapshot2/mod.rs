pub struct SnapshotIdSet {
    // Bit set from (lowerBound + 64)-(lowerBound+127) of the set
    upper_set: i64,

    // Bit set from (lowerBound)-(lowerBound+63) of the set
    lower_set: i64,

    // Lower bound of the bit set. All values above lowerBound+127 are clear.
    // Values between lowerBound and lowerBound+127 are recorded in lowerSet and upperSet
    lower_bound: i32,

    // A sorted array of the index of bits set below lowerBound
    below_bound: Option<Vec<i32>>,
}

impl SnapshotIdSet {
    /// Check if the bit at `index` is set.
    pub fn is_set(&self, index: i32) -> bool {
        let offset = index - self.lower_bound;
        if offset >= 0 && offset < i64::BITS as _ {
            (1 << offset) & self.lower_set != 0
        } else if offset >= i64::BITS as _ && offset < (i64::BITS as i32) * 2 {
            (1 << (offset - i64::BITS as i32)) & self.upper_set != 0
        } else if offset > 0 {
            false
        } else if let Some(ref below_bound) = self.below_bound {
            below_bound.binary_search(&index).is_ok()
        } else {
            false
        }
    }
}
