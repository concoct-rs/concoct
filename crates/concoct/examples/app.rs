use concoct::{Context, Model, View, VirtualDom};

struct Child;

impl View for Child {
    type Message = ();

    type Model = ();

    fn build(&mut self) -> Self::Model {}

    fn body(&mut self, _cx: &Context, _model: &Self::Model) -> impl View<Self::Message> {}
}

struct AppModel;

impl Model for AppModel {
    type Message = ();

    fn update(&mut self, _msg: Self::Message) -> Option<()> {
        todo!()
    }
}

struct App;

impl View for App {
    type Message = ();

    type Model = AppModel;

    fn build(&mut self) -> Self::Model {
        AppModel
    }

    fn body(&mut self, _cx: &Context, _model: &Self::Model) -> impl View<Self::Message> {
        (Child, Child)
    }
}

fn main() {
    let mut vdom = VirtualDom::new(App);
    vdom.build();
    dbg!(vdom);
}
