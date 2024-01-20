use crate::{Tree, View};
use std::any::Any;

macro_rules! one_of {
    ($name:tt, $($t:tt),*) => {
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
            fn build(&mut self) {
                match self {
                    $(
                        $name::$t(tree) => tree.build(),
                    )*
                }
            }

            fn rebuild(&mut self, last: &mut dyn Any) {
                let last =  last.downcast_mut::<Self>().unwrap();
                match (self, last) {
                    $(
                        ($name::$t(tree), $name::$t(last_tree)) => {
                            tree.rebuild(last_tree)
                        }
                    ),*
                    (me, last) => {
                        last.remove();
                        me.build();
                    }
                }

            }

            fn remove(&mut self) {
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
