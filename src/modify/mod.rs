use impl_trait_for_tuples::impl_for_tuples;
use wasm_bindgen::JsCast;
use web_sys::{Element, Event, KeyboardEvent};

mod on;
use crate::web::Context;

pub use self::on::{on, On};

mod value;
pub use self::value::{value, Value};

mod attr;
pub use attr::{attr, class};

pub trait Modify<E> {
    type State;

    fn build(self, cx: &mut Context<E>, elem: &mut Element) -> Self::State;

    fn rebuild(self, cx: &mut Context<E>, elem: &mut Element, state: &mut Self::State);
}

#[impl_for_tuples(16)]
impl<E> Modify<E> for Tuple {
    for_tuples!( type State = ( #( Tuple::State ),* ); );

    fn build(self, cx: &mut Context<E>, elem: &mut Element) -> Self::State {
        for_tuples!( (#( self.Tuple.build(cx, elem) ),*) )
    }

    fn rebuild(self, cx: &mut Context<E>, elem: &mut Element, state: &mut Self::State) {
        for_tuples!( #( self.Tuple.rebuild(cx, elem, &mut state.Tuple); )* )
    }
}

pub fn event_target_value(event: &Event) -> String {
    event
        .target()
        .unwrap()
        .unchecked_into::<web_sys::HtmlInputElement>()
        .value()
}

pub fn event_key_code(event: &Event) -> u32 {
    event.unchecked_ref::<KeyboardEvent>().key_code()
}
