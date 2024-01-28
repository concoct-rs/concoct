pub trait Model<A = ()>: 'static {
    type Message: 'static;

    fn update(&mut self, msg: Self::Message) -> Option<A>;
}

impl<A> Model<A> for () {
    type Message = ();

    fn update(&mut self, _msg: Self::Message) -> Option<A> {
        None
    }
}
