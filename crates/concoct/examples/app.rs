use concoct::{Model, View, VirtualDom};

struct AppModel;

impl Model for AppModel {
    type Message = ();

    fn update(&mut self, msg: Self::Message) -> Option<()> {
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

    fn body(&self, model: &Self::Model) -> impl View<Self::Message> {}
}

fn main() {
    let mut vdom = VirtualDom::new(App);
    vdom.build();
    dbg!(vdom);
}
