use concoct::{composable, Composable};

#[composable]
fn app(count: i32) {
    dbg!(count);
}

fn main() {
    let mut state = None;
    app(0).compose(0, &mut state);
    app(0).compose(0, &mut state);
    app(1).compose(0, &mut state);
}
