use crate::Context;
use web_sys::Text;

pub mod html;
pub use html::{Attribute, Html};

mod lazy;
pub use lazy::{lazy, Lazy};

pub trait View<M> {
    type State;

    fn build(self, cx: &mut Context<M>) -> Self::State;

    fn rebuild(self, cx: &mut Context<M>, state: &mut Self::State);

    fn remove(cx: &mut Context<M>, state: &mut Self::State);
}

impl<M> View<M> for () {
    type State = ();

    fn build(self, _cx: &mut Context<M>) -> Self::State {}

    fn rebuild(self, _cx: &mut Context<M>, _state: &mut Self::State) {}

    fn remove(_cx: &mut Context<M>, _state: &mut Self::State) {}
}

impl<M, V: View<M>> View<M> for Option<V> {
    type State = Option<V::State>;

    fn build(self, cx: &mut Context<M>) -> Self::State {
        self.map(|view| view.build(cx))
    }

    fn rebuild(self, cx: &mut Context<M>, state: &mut Self::State) {
        if let Some(view) = self {
            if let Some(state) = state {
                view.rebuild(cx, state)
            } else {
                *state = Some(view.build(cx))
            }
        } else if let Some(s) = state {
            V::remove(cx, s);
            *state = None;
            cx.skip();
        }
    }

    fn remove(cx: &mut Context<M>, state: &mut Self::State) {
        if let Some(state) = state {
            V::remove(cx, state);
            cx.skip()
        }
    }
}

impl<'a, M> View<M> for &'a str {
    type State = (Self, Text);

    fn build(self, cx: &mut Context<M>) -> Self::State {
        let elem = cx.document.create_text_node(&self);
        cx.insert(&elem);

        (self, elem)
    }

    fn rebuild(self, cx: &mut Context<M>, (prev, text): &mut Self::State) {
        if &self != &*prev {
            text.set_text_content(Some(&self))
        }
        cx.skip()
    }

    fn remove(_cx: &mut Context<M>, state: &mut Self::State) {
        state.1.remove();
    }
}

impl<M> View<M> for String {
    type State = (String, Text);

    fn build(self, cx: &mut Context<M>) -> Self::State {
        let elem = cx.document.create_text_node(&self);
        cx.insert(&elem);
        (self, elem)
    }

    fn rebuild(self, cx: &mut Context<M>, (prev, text): &mut Self::State) {
        if &self != &*prev {
            text.set_text_content(Some(&self))
        }
        cx.skip()
    }

    fn remove(_cx: &mut Context<M>, state: &mut Self::State) {
        state.1.remove();
    }
}

impl<M, A: View<M>, B: View<M>, C: View<M>> View<M> for (A, B, C) {
    type State = (A::State, B::State, C::State);

    fn build(self, cx: &mut Context<M>) -> Self::State {
        (self.0.build(cx), self.1.build(cx), self.2.build(cx))
    }

    fn rebuild(self, cx: &mut Context<M>, state: &mut Self::State) {
        self.0.rebuild(cx, &mut state.0);
        self.1.rebuild(cx, &mut state.1);
        self.2.rebuild(cx, &mut state.2)
    }

    fn remove(cx: &mut Context<M>, state: &mut Self::State) {
        A::remove(cx, &mut state.0);
        B::remove(cx, &mut state.1);
        C::remove(cx, &mut state.2);
    }
}
