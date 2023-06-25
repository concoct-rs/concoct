mod applier;
pub use applier::Applier;

mod compiler;
pub use compiler::run;

pub trait Compose {
    fn changed<T: Clone>(&mut self, value: &T) -> bool;

    fn is_skipping(&self) -> bool;

    fn skip_to_group_end(&mut self);

    fn start_restart_group(&mut self, id: u64);

    fn end_restart_group(&mut self, update: impl FnMut(&mut Self));
}

#[derive(Default)]
pub struct Composer {
    is_changed: bool,
}

impl Compose for Composer {
    fn changed<T: Clone>(&mut self, _value: &T) -> bool {
        self.is_changed = !self.is_changed;
        self.is_changed
    }

    fn is_skipping(&self) -> bool {
        true
    }

    fn skip_to_group_end(&mut self) {}

    fn start_restart_group(&mut self, _id: u64) {}

    fn end_restart_group(&mut self, _update: impl FnMut(&mut Self)) {}
}
