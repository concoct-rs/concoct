///! Viewable components
use crate::Platform;
use impl_trait_for_tuples::impl_for_tuples;

mod lazy;
pub use lazy::{lazy, Lazy};

/// Viewable user interface component.
///
/// Composing views creates a statically-typed UI tree that can be traversed to display elements.
/// Building a view produces its state, which can then be updated when the view is rebuilt.
///
/// Views are generic over [`Platform`] and can be implemented for a single or multiple backends.
pub trait View<P: Platform> {
    /// The state of the view tree.
    /// This type represents both the state of this view and its descendants.
    type State;

    /// Build the initial state of the view.
    fn build(self, cx: &mut P) -> Self::State;

    /// Rebuild the view with its current state.
    fn rebuild(self, cx: &mut P, state: &mut Self::State);

    /// Remove the view from the UI.
    fn remove(cx: &mut P, state: &mut Self::State);
}

impl<P, V> View<P> for Option<V>
where
    P: Platform,
    V: View<P>,
{
    type State = Option<V::State>;

    fn build(self, cx: &mut P) -> Self::State {
        self.map(|view| view.build(cx))
    }

    fn rebuild(self, cx: &mut P, state: &mut Self::State) {
        if let Some(view) = self {
            if let Some(state) = state {
                view.rebuild(cx, state)
            } else {
                *state = Some(view.build(cx))
            }
        } else if let Some(s) = state {
            V::remove(cx, s);
            *state = None;
            cx.advance();
        }
    }

    fn remove(cx: &mut P, state: &mut Self::State) {
        if let Some(state) = state {
            V::remove(cx, state);
            cx.advance()
        }
    }
}

#[impl_for_tuples(16)]
impl<P: Platform> View<P> for Tuple {
    for_tuples!( type State = ( #( Tuple::State ),* ); );

    fn build(self, cx: &mut P) -> Self::State {
        for_tuples!( (#( self.Tuple.build(cx) ),*) )
    }

    fn rebuild(self, cx: &mut P, state: &mut Self::State) {
        for_tuples!( #( self.Tuple.rebuild(cx, &mut state.Tuple); )* )
    }

    fn remove(cx: &mut P, state: &mut Self::State) {
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

    fn build(self, cx: &mut P) -> Self::State {
        self.into_iter()
            .map(|(key, view)| {
                let state = view.build(cx);
                (key, state)
            })
            .collect()
    }

    fn rebuild(self, cx: &mut P, state: &mut Self::State) {
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

    fn remove(cx: &mut P, state: &mut Self::State) {
        remove_views::<_, _, V>(cx, state)
    }
}

fn remove_views<K, P, V>(cx: &mut P, state: &mut [(K, V::State)])
where
    P: Platform,
    V: View<P>,
{
    for (_, view_state) in &mut state[..] {
        V::remove(cx, view_state);
    }
}

#[cfg(feature = "web")]
impl<E> View<crate::web::Web<E>> for &'_ str {
    type State = (Self, web_sys::Text);

    fn build(self, cx: &mut crate::web::Web<E>) -> Self::State {
        let elem = cx.document.create_text_node(&self);
        cx.insert(&elem);

        (self, elem)
    }

    fn rebuild(self, cx: &mut crate::web::Web<E>, (prev, text): &mut Self::State) {
        if &self != &*prev {
            text.set_text_content(Some(&self))
        }
        cx.advance()
    }

    fn remove(_cx: &mut crate::web::Web<E>, state: &mut Self::State) {
        state.1.remove();
    }
}

#[cfg(feature = "web")]
impl<E> View<crate::web::Web<E>> for String {
    type State = (String, web_sys::Text);

    fn build(self, cx: &mut crate::web::Web<E>) -> Self::State {
        let elem = cx.document.create_text_node(&self);
        cx.insert(&elem);
        (self, elem)
    }

    fn rebuild(self, cx: &mut crate::web::Web<E>, (prev, text): &mut Self::State) {
        if &self != &*prev {
            text.set_text_content(Some(&self))
        }
        cx.advance()
    }

    fn remove(_cx: &mut crate::web::Web<E>, state: &mut Self::State) {
        state.1.remove();
    }
}

#[cfg(feature = "web")]
impl<E> View<crate::web::Web<E>> for crate::State<String> {
    type State = (State<String>, web_sys::Text);

    fn build(self, cx: &mut crate::web::Web<E>) -> Self::State {
        let elem = cx.document.create_text_node(&self.get());
        cx.insert(&elem);
        (self, elem)
    }

    fn rebuild(self, cx: &mut crate::web::Web<E>, (prev, text): &mut Self::State) {
        if &self != &*prev {
            text.set_text_content(Some(&self.get()))
        }
        cx.advance()
    }

    fn remove(_cx: &mut crate::web::Web<E>, state: &mut Self::State) {
        state.1.remove();
    }
}
