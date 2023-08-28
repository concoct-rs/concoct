use crate::Platform;
use impl_trait_for_tuples::impl_for_tuples;

pub trait Modify<P: Platform, E> {
    type State;

    fn build(self, cx: &mut P, elem: &mut E) -> Self::State;

    fn rebuild(self, cx: &mut P, elem: &mut E, state: &mut Self::State);
}

#[impl_for_tuples(16)]
impl<P: Platform, E> Modify<P, E> for Tuple {
    for_tuples!( type State = ( #( Tuple::State ),* ); );

    fn build(self, cx: &mut P, elem: &mut E) -> Self::State {
        for_tuples!( (#( self.Tuple.build(cx, elem) ),*) )
    }

    fn rebuild(self, cx: &mut P, elem: &mut E, state: &mut Self::State) {
        for_tuples!( #( self.Tuple.rebuild(cx, elem, &mut state.Tuple); )* )
    }
}
