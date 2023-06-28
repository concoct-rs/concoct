fn a() -> aComposable {
    aComposable {
        is_done: false,
        composable0: b(),
        composable1: b(),
    }
}
struct aComposable {
    is_done: bool,
    composable0: bComposable,
    composable1: bComposable,
}
impl aComposable {
    fn compose(&mut self) {
        if !self.is_done {
            self.composable0.compose();
            self.composable1.compose();
            self.is_done = true;
        }
    }
}
fn b() -> bComposable {
    bComposable { is_done: false }
}
struct bComposable {
    is_done: bool,
}
impl bComposable {
    fn compose(&mut self) {
        if !self.is_done {
            _eprint();
            self.is_done = true;
        }
    }
}
fn main() {}
