use concoct::{composable, Composable};

#[composable]
fn app() {
    dbg!("Hello World");
}

fn main() {
    let mut app = app();
    app.compose(0, ());
    app.compose(0, ());
}
