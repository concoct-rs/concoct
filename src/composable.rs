use crate::use_ref;
use impl_trait_for_tuples::impl_for_tuples;

/// Composable object that handles diffing.
pub trait Composable: PartialEq {
    fn compose(&mut self);
}

impl<F, C> Composable for F
where
    F: FnMut() -> C + PartialEq + Clone + 'static,
    C: Composable + 'static,
{
    fn compose(&mut self) {
        let _f = self.clone();
        use_ref(|| {
            // cx.insert(Box::new(move || Box::new(f())));
        });
    }
}

#[impl_for_tuples(16)]
impl Composable for Tuple
where
    Self: PartialEq,
{
    fn compose(&mut self) {
        for_tuples!(#( self.Tuple.compose(); )*)
    }
}
