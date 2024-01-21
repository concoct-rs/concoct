//! Viewable components of a user-interface.

use crate::{Node, Tree, ViewBuilder};
use std::hash::Hash;

mod cell;
pub use cell::ViewCell;

mod empty;
pub use empty::Empty;

mod memo;
pub use memo::{memo, Memo};

mod one_of;
pub use one_of::*;

/// Viewable component of a user-interface.
///
/// This trait creates a statically-typed tree of views
/// for efficient state updates.
///
/// Most implementations should come from [`ViewBuilder`], which this trait
/// is implemented for.
pub trait View: 'static {
    fn into_tree(self) -> impl Tree;
}

impl<B: View> View for Option<B> {
    fn into_tree(self) -> impl Tree {
        self.map(|me| me.into_tree())
    }
}

impl<K: Hash + Eq + 'static, B: View> View for Vec<(K, B)> {
    fn into_tree(self) -> impl Tree {
        self.into_iter()
            .map(|(key, body)| (key, body.into_tree()))
            .collect::<Vec<_>>()
    }
}

impl<V: ViewBuilder> View for V {
    fn into_tree(self) -> impl Tree {
        Node {
            view: self,
            body: None,
            builder: |me: &'static V| me.build().into_tree(),
            scope: None,
            key: None,
        }
    }
}

macro_rules! impl_view_for_tuple {
    ($($t:tt : $idx:tt),*) => {
        impl<$($t: View),*> View for ($($t),*) {
            fn into_tree(self) -> impl Tree {
                ($(  self.$idx.into_tree() ),*)

            }
        }
    };
}

impl_view_for_tuple!(V1: 0, V2: 1);
impl_view_for_tuple!(V1: 0, V2: 1, V3: 2);
impl_view_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3);
impl_view_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3, V5: 4);
impl_view_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3, V5: 4, V6: 5);
impl_view_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3, V5: 4, V6: 5, V7: 6);
impl_view_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3, V5: 4, V6: 5, V7: 6, V8: 7);
impl_view_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3, V5: 4, V6: 5, V7: 6, V8: 7, V9: 8);
impl_view_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3, V5: 4, V6: 5, V7: 6, V8: 7, V9: 8, V10: 9);
