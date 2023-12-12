use concoct::{Handler, Runtime, Task};

#[derive(Default)]
pub struct Counter {
    value: i32,
}

impl Task for Counter {}

impl Handler<i32> for Counter {
    fn handle(&mut self, msg: i32) {
        self.value = msg;
    }
}

#[tokio::main]
async fn main() {
    let rt = Runtime::default();
    let _guard = rt.enter();

    let a = Counter::default().spawn();
    let b = Counter::default().spawn();

    a.bind(&b);

    a.send(1);
    a.send(2);

    rt.run().await;

    assert_eq!(a.borrow().value, 2);
    assert_eq!(b.borrow().value, 2);
}
