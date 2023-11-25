use crate::{use_ref, BUILD_CONTEXT};

/// Composable object that handles diffing.
pub trait Composable: PartialEq + 'static {
    fn compose(&mut self) -> impl Composable;
}

impl Composable for () {
    fn compose(&mut self) -> impl Composable {}
}

pub fn group<T>(composables: T) -> Group<T> {
    Group::new(composables)
}

#[derive(PartialEq)]
pub struct Group<T> {
    composables: Option<T>,
}
impl<T> Group<T> {
    pub fn new(composables: T) -> Self {
        Self {
            composables: Some(composables),
        }
    }
}

impl<A: Composable, B: Composable> Composable for Group<(A, B)> {
    fn compose(&mut self) -> impl Composable {
        let (a_key, b_key) = *use_ref(|| {
            BUILD_CONTEXT
                .try_with(|cx| {
                    let mut g = cx.borrow_mut();
                    let mut cx = g.as_mut().unwrap().borrow_mut();

                    let composables = self.composables.take().unwrap();
                    let mut a = Some(composables.0);
                    let a_key = cx.insert(Box::new(move || Box::new(a.take().unwrap())));

                    let mut b = Some(composables.1);
                    let b_key = cx.insert(Box::new(move || Box::new(b.take().unwrap())));

                    (a_key, b_key)
                })
                .unwrap()
        })
        .get();

        if let Some(composables) = self.composables.take() {
            BUILD_CONTEXT
                .try_with(|cx| {
                    let mut g = cx.borrow_mut();
                    let mut cx = g.as_mut().unwrap().borrow_mut();

                    let mut a = Some(composables.0);
                    cx.nodes[a_key].make_composable = Box::new(move || Box::new(a.take().unwrap()));

                    let mut b = Some(composables.1);
                    cx.nodes[b_key].make_composable = Box::new(move || Box::new(b.take().unwrap()));
                })
                .unwrap();
        }
    }
}
