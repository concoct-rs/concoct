use concoct::{Scope, View, VirtualDom};

struct Child;

impl View<Counter> for Child {
    fn body(&mut self, cx: &Scope<Counter>) -> impl View<Counter> {
        dbg!(cx.key);
    }
}

struct Counter;

impl View<Self> for Counter {
    fn body(&mut self, _cx: &Scope<Self>) -> impl View<Self> {
        dbg!("view");
        (Child, Child)
    }
}

fn main() {
    let mut app = VirtualDom::new(Counter);
    app.build();
    app.rebuild();
}
