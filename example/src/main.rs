#![feature(register_tool)]
#![register_tool(concoct_rt)]

use concoct::composable;

#[composable]
fn a() {
    b();
    b();
}

#[composable]
fn b() {
    dbg!("test");
}

fn main() {}
