use impl_trait_for_tuples::impl_for_tuples;

use crate::{use_ref, BuildContext};

/// Composable object that handles diffing.
pub trait Composable {
    fn compose(&mut self, cx: &mut BuildContext);
}

impl<F, C> Composable for F
where
    F: FnMut() -> C + Clone + 'static,
    C: Composable + 'static,
{
    fn compose(&mut self, cx: &mut BuildContext) {
        let mut f = self.clone();
        use_ref(|| {
            cx.insert(Box::new(move || Box::new(f())));
        });
    }
}

#[impl_for_tuples(16)]
impl Composable for Tuple {
    fn compose(&mut self, cx: &mut BuildContext) {
        for_tuples!(#( self.Tuple.compose(cx); )*)
    }
}
