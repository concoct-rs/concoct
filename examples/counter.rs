use concoct::{Handle, Object, Runtime, Signal, Slot};

#[derive(Default)]
pub struct Counter {
    value: i32,
}

impl Object for Counter {}

impl Signal<i32> for Counter {}

impl Slot<i32> for Counter {
    fn update(&mut self, cx: Handle<Self>, msg: i32) {
        if self.value != msg {
            self.value = msg;
            cx.emit(msg);
        }
    }
}

#[tokio::main]
async fn main() {
    let rt = Runtime::default();
    let _guard = rt.enter();

    let a = Counter::default().start();
    let b = Counter::default().start();

    a.bind(&b);

    a.send(1);
    a.send(2);

    rt.run().await;

    assert_eq!(a.borrow().value, 2);
    assert_eq!(b.borrow().value, 2);
}
