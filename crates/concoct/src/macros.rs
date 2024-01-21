//! Internal macros

macro_rules! trace {
    ($($tt:tt)*) => {
        #[cfg(feature = "tracing")]
        tracing::trace!($($tt)*)
    }
}

pub(crate) use trace;
