use concoct::{Context, Handle};

#[derive(Default)]
struct Counter {
    value: i32,
}

impl Counter {
    fn set_value(cx: &mut Context<Self>, value: i32) {
        dbg!(value);
        if cx.value != value {
            cx.value = value;
            cx.emit(value);
        }
    }
}

fn main() {
    let a = Handle::new(Counter::default());
    let b = Handle::new(Counter::default());

    a.bind(&b, Counter::set_value);

    Counter::set_value(&mut a.cx(), 2);

    assert_eq!(a.borrow().value, 2);
    assert_eq!(b.borrow().value, 2);
}
