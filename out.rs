#[composable]
pub fn a() {
    panic!("Must be called from a concoct runtime.")
}
#[composable]
pub fn b(x: i32) {
    panic!("Must be called from a concoct runtime.")
}
fn aComposable(composer: &mut impl Composer, changed: u64) {
    composer.start_restart_group(0u64);
    if changed == 0 && composer.is_skipping() {
        composer.skip_to_group_end();
    } else {
        {}
    }
    composer.end_restart_group(|composer| aComposable(composer, changed | 1));
}
fn bComposable(composer: &mut impl Composer, changed: u64, x: i32) {
    composer.start_restart_group(1u64);
    if changed == 0 && composer.is_skipping() {
        composer.skip_to_group_end();
    } else {
        {}
    }
    composer.end_restart_group(|composer| bComposable(composer, changed | 1, x));
}
