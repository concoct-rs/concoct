use impl_trait_for_tuples::impl_for_tuples;

use crate::BuildContext;

/// Composable object that handles diffing.
pub trait Composable {
    type State: 'static;

    fn build(&mut self, cx: &mut BuildContext) -> Self::State;

    fn rebuild(&mut self, state: &mut Self::State);
}

impl<F, C> Composable for F
where
    F: FnMut() -> C + Clone + 'static,
    C: Composable + 'static,
{
    type State = ();

    fn build(&mut self, cx: &mut BuildContext) -> Self::State {
        let mut f = self.clone();
        cx.insert(Box::new(move || Box::new(f())));
    }

    fn rebuild(&mut self, _state: &mut Self::State) {}
}

#[impl_for_tuples(16)]
impl Composable for Tuple {
    for_tuples!( type State = ( #( Tuple::State ),* ); );

    fn build(&mut self, cx: &mut BuildContext) -> Self::State {
        for_tuples!( ( #( self.Tuple.build(cx) ),* ) )
    }

    fn rebuild(&mut self, state: &mut Self::State) {
        {
            for_tuples!(#( self.Tuple.rebuild(&mut state.Tuple); )* )
        };
    }
}
