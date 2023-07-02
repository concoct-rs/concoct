/// A policy to control how the result of [mutableStateOf] report and merge changes to the state object.
/// A mutation policy can be passed as an parameter to [mutableStateOf], and [compositionLocalOf].
/// Typically, one of the stock policies should be used such as [referentialEqualityPolicy],
/// [structuralEqualityPolicy], or [neverEqualPolicy]. However, a custom mutation policy can be
/// created by implementing this interface, such as a counter policy,
pub trait MutationPolicy<T> {
    /// Determine if setting a state value's are equivalent and should be treated as equal.
    /// If [equivalent] returns `true` the new value is not considered a change.

    fn is_eq(&mut self, a: T, b: T) -> bool;

    /// Merge conflicting changes in snapshots. This is only called if [current] and [applied] are
    /// not [equivalent]. If a valid merged value can be calculated then it should be returned.
    ///
    /// For example, if the state object holds an immutable data class with multiple fields,
    /// and [applied] has changed fields that are unmodified by [current] it might be valid to return
    /// a new copy of the data class that combines that changes from both [current] and [applied]
    /// allowing a snapshot to apply that would have otherwise failed.
    #[allow(unused_variables)]
    fn merge(&mut self, previous: T, current: T, applied: T) -> Option<T> {
        None
    }
}

pub struct ReferentialEqualityPolicy;

impl<T> MutationPolicy<T> for ReferentialEqualityPolicy
where
    T: PartialEq,
{
    fn is_eq(&mut self, a: T, b: T) -> bool {
        a == b
    }
}
