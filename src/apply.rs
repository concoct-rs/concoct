/// Applier for changes to a tree of nodes of type `N`.
pub trait Apply<N> {
    type NodeId;

    /// The node that operations will be applied on at any given time. It is expected that the
    /// value of this property will change as [down] and [up] are called.
    fn current(&self) -> Self::NodeId;

    /// Called when the [Composer] is about to begin applying changes using this applier.
    /// [onEndChanges] will be called when changes are complete.
    fn on_begin_changes(&mut self) {}

    /// Called when the [Composer] is finished applying changes using this applier.
    /// A call to [onBeginChanges] will always precede a call to [onEndChanges].
    fn on_end_changes(&mut self) {}

    /// Indicates that the applier is getting traversed "down" the tree. When this gets called,
    /// [node] is expected to be a child of [current], and after this operation, [node] is
    /// expected to be the new [current].
    fn down(&mut self, node: Self::NodeId);

    /// Indicates that the applier is getting traversed "up" the tree. After this operation
    /// completes, the [current] should return the "parent" of the [current] node at the beginning
    /// of this operation.
    fn up(&mut self);

    /// Indicates that [instance] should be inserted as a child to [current] at [index]. An applier
    /// should insert the node into the tree either in [insertTopDown] or [insertBottomUp], not both.
    ///
    /// The [insertTopDown] method is called before the children of [instance] have been created and
    /// inserted into it. [insertBottomUp] is called after all children have been created and inserted.
    fn insert_top_down(&mut self, index: usize, instance: N);

    /// Indicates that [instance] should be inserted as a child of [current] at [index]. An applier
    /// should insert the node into the tree either in [insertTopDown] or [insertBottomUp], not
    /// both. See the description of [insertTopDown] to which describes when to implement
    /// [insertTopDown] and when to use [insertBottomUp].
    fn insert_bottom_up(index: usize, instance: N);

    /// Indicates that the children of [current] from [index] to [index] + [count] should be removed.
    fn remove(&mut self, index: usize, count: usize);

    /// Indicates that [count] children of [current] should be moved from index [from] to index [to].
    /// The [to] index is relative to the position before the change, so, for example, to move an
    /// element at position 1 to after the element at position 2, [from] should be `1` and [to]
    /// should be `3`. If the elements were A B C D E, calling `move(1, 3, 1)` would result in the
    /// elements being reordered to A C B D E.
    fn shift(&mut self, from: usize, to: usize, count: usize);

    /// Move to the root and remove all nodes from the root, preparing both this [Applier]
    /// and its root to be used as the target of a new composition in the future.
    fn clear(&mut self);
}
