use crate::{Context, Model};

pub trait View<A = ()> {
    type Message: 'static;

    type Model: Model<A, Message = Self::Message>;

    fn build(&mut self) -> Self::Model;

    fn body(&mut self, cx: &Context, model: &Self::Model) -> impl View<Self::Message>;
}

impl<A> View<A> for () {
    type Message = ();

    type Model = ();

    fn build(&mut self) -> Self::Model {}

    fn body(&mut self, cx: &Context, _model: &Self::Model) -> impl View<Self::Message> {
        cx.is_empty.set(true);
    }
}

impl<A: 'static, V1: View<A>, V2: View<A>> View<A> for (V1, V2) {
    type Message = ();

    type Model = ();

    fn build(&mut self) -> Self::Model {}

    fn body(&mut self, cx: &Context, _model: &Self::Model) -> impl View<Self::Message> {
        cx.build(&mut self.0);
        cx.build(&mut self.1);
        cx.is_empty.set(true);
    }
}
