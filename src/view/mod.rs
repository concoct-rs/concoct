use impl_trait_for_tuples::impl_for_tuples;
use web_sys::Text;

pub mod html;
pub use html::Html;

mod lazy;
pub use lazy::{lazy, Lazy};

pub trait Platform {
    type Event;

    type Context: Context;
}

pub trait Context {
    fn skip(&mut self);
}

pub trait View<P: Platform> {
    type State;

    fn build(self, cx: &mut P::Context) -> Self::State;

    fn rebuild(self, cx: &mut P::Context, state: &mut Self::State);

    fn remove(cx: &mut P::Context, state: &mut Self::State);
}

impl<P, V> View<P> for Option<V>
where
    P: Platform,
    V: View<P>,
{
    type State = Option<V::State>;

    fn build(self, cx: &mut P::Context) -> Self::State {
        self.map(|view| view.build(cx))
    }

    fn rebuild(self, cx: &mut P::Context, state: &mut Self::State) {
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

    fn remove(cx: &mut P::Context, state: &mut Self::State) {
        if let Some(state) = state {
            V::remove(cx, state);
            cx.skip()
        }
    }
}

#[cfg(feature = "web")]
impl<E> View<crate::web::Web<E>> for &'_ str {
    type State = (Self, Text);

    fn build(self, cx: &mut crate::web::Context<E>) -> Self::State {
        let elem = cx.document.create_text_node(&self);
        cx.insert(&elem);

        (self, elem)
    }

    fn rebuild(self, cx: &mut crate::web::Context<E>, (prev, text): &mut Self::State) {
        if &self != &*prev {
            text.set_text_content(Some(&self))
        }
        cx.skip()
    }

    fn remove(_cx: &mut crate::web::Context<E>, state: &mut Self::State) {
        state.1.remove();
    }
}

#[cfg(feature = "web")]
impl<E> View<crate::web::Web<E>> for String {
    type State = (String, Text);

    fn build(self, cx: &mut crate::web::Context<E>) -> Self::State {
        let elem = cx.document.create_text_node(&self);
        cx.insert(&elem);
        (self, elem)
    }

    fn rebuild(self, cx: &mut crate::web::Context<E>, (prev, text): &mut Self::State) {
        if &self != &*prev {
            text.set_text_content(Some(&self))
        }
        cx.skip()
    }

    fn remove(_cx: &mut crate::web::Context<E>, state: &mut Self::State) {
        state.1.remove();
    }
}

#[impl_for_tuples(16)]
impl<P: Platform> View<P> for Tuple {
    for_tuples!( type State = ( #( Tuple::State ),* ); );

    fn build(self, cx: &mut P::Context) -> Self::State {
        for_tuples!( (#( self.Tuple.build(cx) ),*) )
    }

    fn rebuild(self, cx: &mut P::Context, state: &mut Self::State) {
        for_tuples!( #( self.Tuple.rebuild(cx, &mut state.Tuple); )* )
    }

    fn remove(cx: &mut P::Context, state: &mut Self::State) {
        for_tuples!( #( Tuple::remove(cx, &mut state.Tuple); )* )
    }
}

impl<P, K, V> View<P> for Vec<(K, V)>
where
    P: Platform,
    K: PartialEq,
    V: View<P>,
{
    type State = Vec<(K, V::State)>;

    fn build(self, cx: &mut P::Context) -> Self::State {
        self.into_iter()
            .map(|(key, view)| {
                let state = view.build(cx);
                (key, state)
            })
            .collect()
    }

    fn rebuild(self, cx: &mut P::Context, state: &mut Self::State) {
        // Build new views and rebuild old views
        let new_state = self
            .into_iter()
            .map(|(key, view)| {
                let view_state = if let Some(pos) = state
                    .iter_mut()
                    .position(|(state_key, _)| &key == state_key)
                {
                    let (_, mut view_state) = state.remove(pos);
                    view.rebuild(cx, &mut view_state);
                    view_state
                } else {
                    view.build(cx)
                };
                (key, view_state)
            })
            .collect();

        // Remove trailing views
        remove_views::<_, _, V>(cx, state);

        *state = new_state;
    }

    fn remove(cx: &mut P::Context, state: &mut Self::State) {
        remove_views::<_, _, V>(cx, state)
    }
}

fn remove_views<K, P, V>(cx: &mut P::Context, state: &mut [(K, V::State)])
where
    P: Platform,
    V: View<P>,
{
    for (_, view_state) in &mut state[..] {
        V::remove(cx, view_state);
    }
}
