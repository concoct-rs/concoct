use crate::{use_ref, ViewContext};
use crate::{AnyView, View};

pub trait IntoView: 'static {
    fn into_view(self) -> impl View;
}

impl<C: View> IntoView for C {
    fn into_view(self) -> impl View {
        self
    }
}

impl IntoView for Box<dyn AnyView> {
    fn into_view(self) -> impl View {
        let mut view = Some(self);

        let key = *use_ref(|| {
            let mut cx = ViewContext::current();
            let view = view.take().unwrap();

            let mut a = Some(view);
            cx.insert(Box::new(move || Box::new(a.take().unwrap().into_view())))
        })
        .get();

        if let Some(view) = view.take() {
            let cx = ViewContext::current();

            let mut a = Some(view);
            cx.inner.borrow_mut().nodes[key].borrow_mut().make_view =
                Box::new(move || Box::new(a.take().unwrap().into_view()));
        }
    }
}

macro_rules! impl_view_for_tuple {
    ($($a:ident: $b:tt),*) => {
        impl<$($a: IntoView),*> IntoView for ($($a),*) {
            fn into_view(self) -> impl View {
                let mut views = Some(self);
                let keys = *use_ref(|| {
                    let mut cx = ViewContext::current();
                    let views = views.take().unwrap();

                    ($({
                        let mut a = Some(views.$b);
                        cx.insert(Box::new(move || Box::new(a.take().unwrap().into_view())))
                    }),*)
                })
                .get();

                if let Some(views) = views.take() {
                    let cx = ViewContext::current();

                    ($({
                        let mut a = Some(views.$b);
                        cx.inner.borrow_mut().nodes[keys.$b].borrow_mut().make_view =
                            Box::new(move || Box::new(a.take().unwrap().into_view()));
                    }),*);
                }
            }
        }
    };
}

macro_rules! impl_view_for_tuples {
    ($( ( $( $a:tt: $b:tt ),* ) ), * ) => {
        $(impl_view_for_tuple!($($a: $b),*);)*
    };
}

impl_view_for_tuples!(
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
