use crate::{use_ref, BUILD_CONTEXT};

/// Composable object that handles diffing.
pub trait Composable: PartialEq + 'static {
    fn compose(&mut self) -> impl Composable;
}

impl Composable for () {
    fn compose(&mut self) -> impl Composable {}
}

impl<A: Composable + Clone, B: Composable + Clone> Composable for (A, B) {
    fn compose(&mut self) -> impl Composable {
        use_ref(|| {
            BUILD_CONTEXT
                .try_with(|cx| {
                    let mut g = cx.borrow_mut();
                    let mut cx = g.as_mut().unwrap().borrow_mut();

                    let a = self.0.clone();
                    cx.insert(Box::new(move || Box::new(a.clone())));

                    let b = self.1.clone();
                    cx.insert(Box::new(move || Box::new(b.clone())));
                })
                .unwrap();
        });
    }
}
