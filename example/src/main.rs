include!(concat!(env!("OUT_DIR"), "/app.rs"));

use concoct::Composer;

fn main() {
    let mut cx = Composer::default();
    appComposable(&mut cx, 0, 0);
    appComposable(&mut cx, 0, 0);
}
