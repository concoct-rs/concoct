include!(concat!(env!("OUT_DIR"), "/app.rs"));

use concoct::Composer;

fn main() {
    let mut cx = Composer::default();
    counterComposable(&mut cx, 0, 0);
    counterComposable(&mut cx, 0, 0);
}
