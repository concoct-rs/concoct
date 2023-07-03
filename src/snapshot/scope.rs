use std::{cell::RefCell, collections::HashSet, mem};

#[derive(Debug, Default)]
pub struct Scope {
    pub reads: HashSet<u64>,
    pub writes: HashSet<u64>,
}

impl Scope {
    pub fn enter(self, f: impl FnOnce()) -> Self {
        let parent = LOCAL_SCOPE
            .try_with(|scope| mem::replace(&mut *scope.borrow_mut(), Some(self)))
            .unwrap();

        f();

        LOCAL_SCOPE
            .try_with(|scope| mem::replace(&mut *scope.borrow_mut(), parent))
            .unwrap()
            .unwrap()
    }
}

thread_local! {
    pub(super) static LOCAL_SCOPE: RefCell<Option<Scope>> = RefCell::new(None);
}
