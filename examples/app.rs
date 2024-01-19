use concoct::{Body, Context, Tree, View};

struct A;

impl View for A {
    fn body(&self) -> impl Body {
        concoct::request_update();

        dbg!("A");
    }
}

struct App;

impl View for App {
    fn body(&self) -> impl Body {
        dbg!("App");
        A
    }
}

fn main() {
    let cx = Context::default();
    cx.enter();

    let v = App;
    let mut tree = v.tree();
    tree.build();

    cx.rebuild();
}
