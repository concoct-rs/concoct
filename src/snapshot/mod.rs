//! ```
//! use concoct::snapshot::{Snapshot, State};
//!
//! let mut state = State::new(0);
//! state.set(1);
//!
//! // State is not updated until the next snapshot is entered
//! assert_eq!(*state.get(), 0);
//!
//! let snapshot = Snapshot::take();
//! snapshot.enter(|| {
//!     assert_eq!(*state.get(), 1);
//! });
//! ```

use std::{
    any::Any,
    borrow::BorrowMut,
    cell::RefCell,
    collections::HashSet,
    mem,
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex,
    },
};

pub mod mutation_policy;

mod state;
pub use state::State;

pub struct Snapshot {
    id: u64,
    invalid: HashSet<u64>,
    kind: SnapKind,
}

impl Snapshot {
    pub fn take() -> Self {
        Self::take_with_observer(None)
    }

    pub fn take_with_observer(read_observer: Option<Box<dyn FnMut(Box<dyn Any>)>>) -> Self {
        with_current_snapshot(|snapshot| snapshot.take_nested_snapshot(read_observer))
    }

    pub fn enter<R>(self, f: impl FnOnce() -> R) -> R {
        let prev = swap_current(Some(self));
        let output = f();
        swap_current(prev);
        output
    }

    fn take_nested_snapshot(
        &mut self,
        _read_observer: Option<Box<dyn FnMut(Box<dyn Any>)>>,
    ) -> Self {
        match self.kind {
            SnapKind::Global => {
                let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
                Self {
                    id,
                    invalid: self.invalid.clone(),
                    kind: SnapKind::ReadOnly,
                }
            }
            SnapKind::ReadOnly => todo!(),
        }
    }
}

pub enum SnapKind {
    Global,
    ReadOnly,
}

lazy_static::lazy_static! {
    static ref GLOBAL_SNAPSHOT: Mutex<Snapshot> = Mutex::new(Snapshot { id: 1, invalid: HashSet::new(), kind: SnapKind::Global });
}

static NEXT_ID: AtomicU64 = AtomicU64::new(2);

thread_local! {
    static THREAD_SNAPSHOT: RefCell<Option<Snapshot>> = RefCell::new(None);
}

pub fn swap_current(snapshot: Option<Snapshot>) -> Option<Snapshot> {
    THREAD_SNAPSHOT
        .try_with(|current| mem::replace(&mut *current.borrow_mut(), snapshot))
        .unwrap()
}

pub fn with_current_snapshot<R>(f: impl FnOnce(&mut Snapshot) -> R) -> R {
    THREAD_SNAPSHOT
        .try_with(|thread_snapshot| {
            if let Some(snapshot) = thread_snapshot.borrow_mut().as_mut() {
                f(snapshot.borrow_mut())
            } else {
                let mut snapshot = GLOBAL_SNAPSHOT.lock().unwrap();
                f(&mut *snapshot)
            }
        })
        .unwrap()
}

const INVALID_SNAPSHOT: u64 = 0;
