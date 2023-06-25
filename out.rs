#[composable]
pub fn a() {
    panic!("Must be called from a concoct runtime.")
}
#[composable]
pub fn b() {
    panic!("Must be called from a concoct runtime.")
}
fn aComposable(composer: &mut Composer) {
    {
        bComposable(composer);
    }
}
fn bComposable(composer: &mut Composer) {
    {}
}
