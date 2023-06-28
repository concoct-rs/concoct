#![feature(register_tool)]
#![register_tool(concoct_rt)]

use concoct::composable;

#[composable]
fn a() {
    dbg!("test");
}

fn main() {}
