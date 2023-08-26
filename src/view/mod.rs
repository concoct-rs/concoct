use crate::Context;
use web_sys::Text;

pub mod html;
pub use html::{Attribute, Html};

pub trait View<M> {
    type State;

    fn build(self, cx: &mut Context<M>) -> Self::State;

    fn rebuild(self, cx: &mut Context<M>, state: &mut Self::State);
}

impl<M> View<M> for () {
    type State = ();

    fn build(self, cx: &mut Context<M>) -> Self::State {}

    fn rebuild(self, cx: &mut Context<M>, state: &mut Self::State) {}
}

impl<'a, M> View<M> for &'a str {
    type State = (Self, Text);

    fn build(self, cx: &mut Context<M>) -> Self::State {
        let elem = cx.document.create_text_node(&self);
        cx.stack.last_mut().unwrap().append_child(&elem).unwrap();
        (self, elem)
    }

    fn rebuild(self, _cx: &mut Context<M>, (prev, text): &mut Self::State) {
        if &self != &*prev {
            text.set_text_content(Some(&self))
        }
    }
}

impl<M> View<M> for String {
    type State = (String, Text);

    fn build(self, cx: &mut Context<M>) -> Self::State {
        let elem = cx.document.create_text_node(&self);
        cx.stack.last_mut().unwrap().append_child(&elem).unwrap();
        (self, elem)
    }

    fn rebuild(self, _cx: &mut Context<M>, (prev, text): &mut Self::State) {
        if &self != &*prev {
            text.set_text_content(Some(&self))
        }
    }
}

impl<M, A: View<M>, B: View<M>, C: View<M>> View<M> for (A, B, C) {
    type State = (A::State, B::State, C::State);

    fn build(self, cx: &mut Context<M>) -> Self::State {
        (self.0.build(cx), self.1.build(cx),self.2.build(cx))
    }

    fn rebuild(self, cx: &mut Context<M>, state: &mut Self::State) {
        self.0.rebuild(cx, &mut state.0);
        self.1.rebuild(cx, &mut state.1);
        self.2.rebuild(cx, &mut state.2)
    }
}
