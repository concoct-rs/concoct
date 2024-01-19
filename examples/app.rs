use concoct::{hook::use_state, Body, Context, Tree, View};

struct A {
    n: u8,
}

impl View for A {
    fn body(&self) -> impl Body {
        let (count, set_count) = use_state(|| 0);
        dbg!((self.n, &count));

        set_count(*count + 1)
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
