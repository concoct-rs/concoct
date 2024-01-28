use crate::Model;

pub trait View<A = ()> {
    type Message: 'static;

    type Model: Model<A, Message = Self::Message>;

    fn build(&mut self) -> Self::Model;

    fn body(&self, model: &Self::Model) -> impl View<Self::Message>;
}

impl<A> View<A> for () {
    type Message = ();

    type Model = ();

    fn build(&mut self) -> Self::Model {}

    fn body(&self, _model: &Self::Model) -> impl View<Self::Message> {}
}
