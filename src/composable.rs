use crate::{use_ref, AnyComposable, BUILD_CONTEXT};

/// Composable object that handles diffing.
pub trait Composable: PartialEq + 'static {
    fn compose(&mut self) -> impl IntoComposable;
}

impl Composable for () {
    fn compose(&mut self) -> impl IntoComposable {}
}

pub trait IntoComposable: 'static {
    fn into_composer(self) -> impl Composable;
}

impl<C: Composable> IntoComposable for C {
    fn into_composer(self) -> impl Composable {
        self
    }
}

macro_rules! impl_composable_for_tuple {
    ($($a:ident: $b:tt),*) => {
        impl<$($a: IntoComposable),*> IntoComposable for ($($a),*) {
            fn into_composer(self) -> impl Composable {
                let mut composables = Some(self);
                let keys = *use_ref(|| {
                    BUILD_CONTEXT
                        .try_with(|cx| {
                            let mut g = cx.borrow_mut();
                            let mut cx = g.as_mut().unwrap().borrow_mut();
                            let composables = composables.take().unwrap();

                            ($({
                                let mut a = Some(composables.$b);
                                cx.insert(Box::new(move || Box::new(a.take().unwrap().into_composer())))
                            }),*)
                        })
                        .unwrap()
                })
                .get();

                if let Some(composables) = composables.take() {
                    BUILD_CONTEXT
                        .try_with(|cx| {
                            let mut g = cx.borrow_mut();
                            let cx = g.as_mut().unwrap().borrow_mut();

                            ($({
                                let mut a = Some(composables.$b);
                                cx.nodes[keys.$b].borrow_mut().make_composable =
                                    Box::new(move || Box::new(a.take().unwrap().into_composer()));
                            }),*)
                        })
                        .unwrap();
                }
            }
        }
    };
}

impl Composable for &'static str {
    fn compose(&mut self) -> impl IntoComposable {
        BUILD_CONTEXT
            .try_with(|cx| {
                let g = cx.borrow();
                let mut cx = g.as_ref().unwrap().borrow_mut();
                cx.platform.from_str(self).any_build()
            })
            .unwrap();
    }
}

macro_rules! impl_composable_for_tuples {
    ($( ( $( $a:tt: $b:tt ),* ) ), * ) => {
        $(impl_composable_for_tuple!($($a: $b),*);)*
    };
}

impl_composable_for_tuples!(
    (A: 0, B: 1),
    (A: 0, B: 1, C: 2),
    (A: 0, B: 1, C: 2, D: 3),
    (A: 0, B: 1, C: 2, D: 3, E: 4),
    (A: 0, B: 1, C: 2, D: 3, E: 4, F: 5),
    (A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6),
    (A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7),
    (A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8),
    (A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9),
    (A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10),
    (A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11),
    (A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12),
    (A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12, N: 13),
    (A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12, N: 13, O: 14),
    (A: 0, B: 1, C: 2, D: 3, E: 4, F: 5, G: 6, H: 7, I: 8, J: 9, K: 10, L: 11, M: 12, N: 13, O: 14, P: 15)
);
