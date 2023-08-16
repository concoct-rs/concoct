use std::{any::Any, num::NonZeroU128};
use taffy::Taffy;

mod adapt;
pub use adapt::{Adapt, AdaptThunk};

mod canvas;
pub use canvas::Canvas;

pub mod layout_context;
pub use layout_context::LayoutContext;

mod remember;
pub use remember::{remember, Remember};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Id(NonZeroU128);

pub struct BuildContext {
    pub next_id: NonZeroU128,
    pub unused_ids: Vec<Id>,
}

impl BuildContext {
    pub fn id(&mut self) -> Id {
        self.unused_ids.pop().unwrap_or_else(|| {
            let id = self.next_id;
            self.next_id = self.next_id.checked_add(1).unwrap();
            Id(id)
        })
    }
}

pub trait View<T, A = ()> {
    fn build(&mut self, cx: &mut BuildContext) -> Id;

    fn rebuild(&mut self, cx: &mut BuildContext, old: &mut Self);

    fn layout(&mut self, cx: &mut LayoutContext, id: Id);

    fn paint(&mut self, taffy: &Taffy, canvas: &mut skia_safe::Canvas);

    fn message(&mut self, state: &mut T, id_path: &[Id], message: &dyn Any);
}
