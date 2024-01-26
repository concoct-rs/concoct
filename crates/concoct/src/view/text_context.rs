use crate::Scope;
use std::cell::RefCell;

/// Provider for a platform-specific text view.
///
/// If you're writing a custom backend, you can use this to override
/// the default implementation of `View` for string types (like `&str` and `String`).
///
/// To expose it to child views, use [`use_provider`](`crate::hook::use_provider`).
pub struct TextContext<T, A> {
    pub(crate) view: RefCell<Box<dyn FnMut(&Scope<T, A>, &str)>>,
}

impl<T, A> TextContext<T, A> {
    /// Create a text view context from a view function.
    ///
    /// Text-based views, such as `&str` or `String` will call
    /// this view function on when rendered.
    pub fn new(view: impl FnMut(&Scope<T, A>, &str) + 'static) -> Self {
        Self {
            view: RefCell::new(Box::new(view)),
        }
    }
}
