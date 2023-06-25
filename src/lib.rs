pub trait Compose {
    fn changed<T>(&mut self, value: T) -> bool;

    fn is_skipping(&self) -> bool;

    fn skip_to_group_end(&mut self);

    fn start_restart_group(&mut self, id: u64);

    fn end_restart_group(&mut self, update: impl FnMut(&mut Self));
}

pub struct Composer {

}

impl Compose for Composer {
    fn changed<T>(&mut self, value: T) -> bool {
        dbg!("changed");
        false
    }

    fn is_skipping(&self) -> bool {
        true
    }

    fn skip_to_group_end(&mut self) {
        todo!()
    }

    fn start_restart_group(&mut self, id: u64) {
        dbg!(id);
    }

    fn end_restart_group(&mut self, update: impl FnMut(&mut Self)) {
        todo!()
    }
}