/// Marker trait for an action.
pub trait Action {}

/// Conversion into an [`Action`].
pub trait IntoAction<A>: sealed::Sealed {
    /// Convert an output to an optional action.
    fn into_action(self) -> Option<A>;
}

mod sealed {
    pub trait Sealed {}
}

impl sealed::Sealed for () {}

impl<A> IntoAction<A> for () {
    fn into_action(self) -> Option<A> {
        None
    }
}

impl<A: Action> sealed::Sealed for A {}

impl<A: Action> IntoAction<A> for A {
    fn into_action(self) -> Option<A> {
        Some(self)
    }
}

impl<A: Action> sealed::Sealed for Option<A> {}

impl<A: Action> IntoAction<A> for Option<A> {
    fn into_action(self) -> Option<A> {
        self
    }
}
