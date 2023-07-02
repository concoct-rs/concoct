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


pub trait Snapshot {
    fn take_nested_snapshot(
        &mut self,
        read_observer: Option<Box<dyn FnMut(Box<dyn Any>)>>,
    ) -> Box<dyn Snapshot>;
}

static GLOBAL_SNAPSHOT: Mutex<GlobalSnapshot> = Mutex::new(GlobalSnapshot {});

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

thread_local! {
    static THREAD_SNAPSHOT: RefCell<Option<Box<dyn Snapshot>>> = RefCell::new(None);
}

pub fn snapshot(read_observer: Option<Box<dyn FnMut(Box<dyn Any>)>>) -> Box<dyn Snapshot> {
    with_current_snapshot(|snapshot| snapshot.take_nested_snapshot(read_observer))
}

pub fn swap_current(snapshot: Option<Box<dyn Snapshot>>) -> Option<Box<dyn Snapshot>> {
    THREAD_SNAPSHOT
        .try_with(|current| mem::replace(&mut *current.borrow_mut(), snapshot))
        .unwrap()
}

pub fn enter<R>(snapshot: Box<dyn Snapshot>, f: impl FnOnce() -> R) -> R {
    let prev = swap_current(Some(snapshot));
    let output = f();
    swap_current(prev);
    output
}

pub struct GlobalSnapshot {}

impl Snapshot for GlobalSnapshot {
    fn take_nested_snapshot(
        &mut self,
        read_observer: Option<Box<dyn FnMut(Box<dyn Any>)>>,
    ) -> Box<dyn Snapshot> {
        let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
        Box::new(ReadOnlySnapshot { id, read_observer })
    }
}

pub fn with_current_snapshot<R>(f: impl FnOnce(&mut dyn Snapshot) -> R) -> R {
    THREAD_SNAPSHOT
        .try_with(|thread_snapshot| {
            if let Some(snapshot) = thread_snapshot.borrow_mut().as_deref_mut() {
                f(snapshot.borrow_mut())
            } else {
                let mut snapshot = GLOBAL_SNAPSHOT.lock().unwrap();
                f(&mut *snapshot)
            }
        })
        .unwrap()
}

pub struct ReadOnlySnapshot {
    id: u64,
    // invalid = invalid,
    read_observer: Option<Box<dyn FnMut(Box<dyn Any>)>>,
}

impl Snapshot for ReadOnlySnapshot {
    fn take_nested_snapshot(
        &mut self,
        read_observer: Option<Box<dyn FnMut(Box<dyn Any>)>>,
    ) -> Box<dyn Snapshot> {
        todo!()
    }
}
