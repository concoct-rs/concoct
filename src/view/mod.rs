use crate::Id;
use std::any::Any;

mod adapt;
pub use adapt::{Adapt, AdaptThunk};

mod text;
pub use text::Text;

pub trait View<T, A> {
    type State;

    fn view(&mut self, state: &mut T, id_path: &[Id], message: Box<dyn Any>);
}
