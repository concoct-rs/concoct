use concoct::{Compose, Composer};

fn bComposable(composer: &mut impl Compose, changed: u64, x: i32) {
    composer.start_restart_group(1u64);
    let mut dirty = changed;
    if changed & 14u64 == 0 {
        dirty = changed | if composer.changed(x) { 4 } else { 2 };
    }
    if dirty & 11u64 == 2 && composer.is_skipping() {
        composer.skip_to_group_end();
    } else {
        {}
    }
    composer.end_restart_group(|composer| bComposable(composer, changed | 1, x));
}

fn main() {
    let mut cx = Composer {};

    bComposable(&mut cx, 0, 0);
}
