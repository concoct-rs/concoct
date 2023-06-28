#![feature(register_tool)]
#![register_tool(concoct_rt)]

use concoct::{composable, Composable};

#[composable]
fn a(count: i32) {
    dbg!(count);
}

fn main() {
    let mut app = a();
    app.compose(());
}
