fn a() -> aComposable {
    aComposable { is_done: false }
}
struct aComposable {
    is_done: bool,
}
impl aComposable {
    fn compose(&mut self, count: i32) {
        if !self.is_done {
            _eprint();
            self.is_done = true;
        }
    }
}
fn main() {}
