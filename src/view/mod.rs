use crate::Context;

mod text;
pub use text::Text;

pub trait View {
    fn view(&mut self, cx: &mut Context);
}

impl View for String {
    fn view(&mut self, _cx: &mut Context) {
        dbg!(self);
    }
}
