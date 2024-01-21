use crate::{Tree, View};
use std::any::Any;

macro_rules! one_of {
    ($name:tt, $($t:tt),*) => {
        /// Container view for children of multiple types.
        pub enum $name<$($t),*> {
            $($t($t)),*
        }

        impl<$($t: View),*> View for $name<$($t),*> {
            fn into_tree(self) -> impl Tree {
                match self {
                    $(
                        $name::$t(body) => $name::$t(body.into_tree()),
                    )*
                }
            }
        }

        impl<$($t: Tree),*> Tree for $name<$($t),*> {
            unsafe fn build(&mut self) {
                match self {
                    $(
                        $name::$t(tree) => tree.build(),
                    )*
                }
            }

            unsafe fn rebuild(&mut self, last: &mut dyn Any, is_changed: bool) {
                let last =  last.downcast_mut::<Self>().unwrap();
                match (self, last) {
                    $(
                        ($name::$t(tree), $name::$t(last_tree)) => {
                            tree.rebuild(last_tree, is_changed)
                        }
                    ),*
                    (me, last) => {
                        last.remove();
                        me.build();
                    }
                }

            }

            unsafe fn remove(&mut self) {
                match self {
                    $(
                        $name::$t(tree) => tree.remove(),
                    )*
                }
            }
        }
    };
}

one_of!(OneOf2, A, B);
one_of!(OneOf3, A, B, C);
one_of!(OneOf4, A, B, C, D);
one_of!(OneOf5, A, B, C, D, E);
one_of!(OneOf6, A, B, C, D, E, F);
one_of!(OneOf7, A, B, C, D, E, F, G);
one_of!(OneOf8, A, B, C, D, E, F, G, H);
