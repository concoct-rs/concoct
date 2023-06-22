mod button;
pub use button::Button;

use crate::Context;

pub trait View {
    fn view(&mut self, cx: &mut Context);
}

impl View for String {
    fn view(&mut self, _cx: &mut Context) {
        dbg!(self);
    }
}
