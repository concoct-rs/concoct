use concoct::{Context, Object, Signal};

#[derive(Default)]
struct Counter {
    value: i32,
}

impl Object for Counter {}

impl Signal<i32> for Counter {}

impl Counter {
    fn set_value(cx: &mut Context<Self>, value: i32) {
        if cx.value != value {
            cx.value = value;
            cx.emit(value);
        }
    }
}

fn main() {
    let a = Counter::default().start();
    let b = Counter::default().start();

    a.bind(&b, Counter::set_value);

    Counter::set_value(&mut a.cx(), 2);

    assert_eq!(a.borrow().value, 2);
    assert_eq!(b.borrow().value, 2);
}
