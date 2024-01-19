use concoct::{use_ref, Body, Context, Tree, View};
use std::cell::RefCell;

struct A {
    n: u8,
}

impl View for A {
    fn body(&self) -> impl Body {
        concoct::request_update();

        let count = use_ref(|| RefCell::new(0));
        dbg!(self.n, &count);

        *count.borrow_mut() += 1;
    }
}

struct App;

impl View for App {
    fn body(&self) -> impl Body {
        (A { n: 0 }, A { n: 1 })
    }
}

fn main() {
    let cx = Context::default();
    cx.enter();

    let v = App;
    let mut tree = v.tree();
    tree.build();

    cx.rebuild();
    cx.rebuild();
}
