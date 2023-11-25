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
        let (a_key, b_key) = *use_ref(|| {
            BUILD_CONTEXT
                .try_with(|cx| {
                    let mut g = cx.borrow_mut();
                    let mut cx = g.as_mut().unwrap().borrow_mut();

                    let a = self.0.clone();
                    let a_key = cx.insert(Box::new(move || Box::new(a.clone())));

                    let b = self.1.clone();
                    let b_key = cx.insert(Box::new(move || Box::new(b.clone())));

                    (a_key, b_key)
                })
                .unwrap()
        })
        .get();

        BUILD_CONTEXT
            .try_with(|cx| {
                let mut g = cx.borrow_mut();
                let mut cx = g.as_mut().unwrap().borrow_mut();

                let a = self.0.clone();
                let b = self.1.clone();
                cx.nodes[a_key].make_composable = Box::new(move || Box::new(a.clone()));
                cx.nodes[b_key].make_composable = Box::new(move || Box::new(b.clone()));
            })
            .unwrap();
    }
}
