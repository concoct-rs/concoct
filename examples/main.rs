use concoct::{composable, Composable};

#[composable]
fn app() {
    dbg!("Hello World");
}

fn main() {
    let mut state = None;
    app().compose(0, &mut state);
    app().compose(0, &mut state);
}
