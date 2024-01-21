use std::{any::Any, collections::HashSet, hash::Hash};

mod node;
pub use node::Node;

/// Statically-typed view tree.
///
/// This trait is unsafe and intended as the lower-level backend of [`View`](crate::View).
pub trait Tree: 'static {
    unsafe fn build(&mut self);

    unsafe fn rebuild(&mut self, last: &mut dyn Any, is_changed: bool);

    unsafe fn remove(&mut self);
}

impl<T: Tree> Tree for Option<T> {
    unsafe fn build(&mut self) {
        if let Some(tree) = self {
            tree.build()
        }
    }

    unsafe fn rebuild(&mut self, last: &mut dyn Any, _is_changed: bool) {
        if let Some(tree) = self {
            if let Some(last_tree) = last.downcast_mut::<Self>().unwrap() {
                tree.rebuild(last_tree, true)
            } else {
                tree.build();
            }
        } else if let Some(last_tree) = last.downcast_mut::<Self>().unwrap() {
            last_tree.remove();
        }
    }

    unsafe fn remove(&mut self) {
        if let Some(tree) = self {
            tree.remove()
        }
    }
}

impl<K, T> Tree for Vec<(K, T)>
where
    K: Hash + Eq + 'static,
    T: Tree,
{
    unsafe fn build(&mut self) {
        for (_, body) in self.iter_mut() {
            body.build()
        }
    }

    unsafe fn rebuild(&mut self, last: &mut dyn Any, _is_changed: bool) {
        let mut visited = HashSet::new();
        let last = last.downcast_mut::<Self>().unwrap();

        for (key, body) in self.iter_mut() {
            if let Some((_, last_body)) = last.iter_mut().find(|(last_key, _)| last_key == key) {
                body.rebuild(last_body, true);
                visited.insert(key);
            } else {
                body.build();
            }
        }

        for (key, body) in last.iter_mut() {
            if !visited.contains(key) {
                body.remove();
            }
        }
    }

    unsafe fn remove(&mut self) {
        for (_, body) in self.iter_mut() {
            body.remove()
        }
    }
}

macro_rules! impl_tree_for_tuple {
    ($($t:tt : $idx:tt),*) => {
        impl<$($t: Tree),*> Tree for ($($t),*) {
            unsafe fn build(&mut self) {
               $(
                    self.$idx.build();
               )*
            }

            unsafe fn rebuild(&mut self, last: &mut dyn Any, is_changed: bool) {
                if let Some(last) = last.downcast_mut::<Self>() {
                    $(
                        self.$idx.rebuild(&mut last.$idx, is_changed);
                    )*
                }
            }

            unsafe fn remove(&mut self) {
                $(
                     self.$idx.remove();
                )*
             }
        }
    };
}

impl_tree_for_tuple!(V1: 0, V2: 1);
impl_tree_for_tuple!(V1: 0, V2: 1, V3: 2);
impl_tree_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3);
impl_tree_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3, V5: 4);
impl_tree_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3, V5: 4, V6: 5);
impl_tree_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3, V5: 4, V6: 5, V7: 6);
impl_tree_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3, V5: 4, V6: 5, V7: 6, V8: 7);
impl_tree_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3, V5: 4, V6: 5, V7: 6, V8: 7, V9: 8);
impl_tree_for_tuple!(V1: 0, V2: 1, V3: 2, V4: 3, V5: 4, V6: 5, V7: 6, V8: 7, V9: 8, V10: 9);
