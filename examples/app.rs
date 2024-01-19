use std::cell::RefCell;

use concoct::{use_ref, Body, Context, Tree, View};

struct A;

impl View for A {
    fn body(&self) -> impl Body {
        concoct::request_update();

        let count = use_ref(|| RefCell::new(0));
        dbg!(&count);

        *count.borrow_mut() += 1;

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
