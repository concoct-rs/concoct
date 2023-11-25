use crate::{use_ref, BUILD_CONTEXT};

/// Composable object that handles diffing.
pub trait Composable: PartialEq + 'static {
    fn compose(&mut self) -> impl IntoComposable;
}

impl Composable for () {
    fn compose(&mut self) -> impl IntoComposable {}
}

pub trait IntoComposable {
    fn into_composer(self) -> impl Composable;
}

impl<C: Composable> IntoComposable for C {
    fn into_composer(self) -> impl Composable {
        self
    }
}

impl<A: Composable, B: Composable> IntoComposable for (A, B) {
    fn into_composer(self) -> impl Composable {
        let mut composables = Some(self);
        let (a_key, b_key) = *use_ref(|| {
            BUILD_CONTEXT
                .try_with(|cx| {
                    let mut g = cx.borrow_mut();
                    let mut cx = g.as_mut().unwrap().borrow_mut();

                    let composables = composables.take().unwrap();
                    let mut a = Some(composables.0);
                    let a_key = cx.insert(Box::new(move || Box::new(a.take().unwrap())));

                    let mut b = Some(composables.1);
                    let b_key = cx.insert(Box::new(move || Box::new(b.take().unwrap())));

                    (a_key, b_key)
                })
                .unwrap()
        })
        .get();

        if let Some(composables) = composables.take() {
            BUILD_CONTEXT
                .try_with(|cx| {
                    let mut g = cx.borrow_mut();
                    let cx = g.as_mut().unwrap().borrow_mut();

                    let mut a = Some(composables.0);
                    cx.nodes[a_key].borrow_mut().make_composable =
                        Box::new(move || Box::new(a.take().unwrap()));

                    let mut b = Some(composables.1);
                    cx.nodes[b_key].borrow_mut().make_composable =
                        Box::new(move || Box::new(b.take().unwrap()));
                })
                .unwrap();
        }
    }
}
