pub trait Composer {
    fn is_skipping(&self) -> bool;

    fn skip_to_group_end(&mut self);

    fn start_restart_group(&mut self, id: u64);

    fn end_restart_group(&mut self, update: impl FnMut(&mut Self));
}
