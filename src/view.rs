use crate::{html::Parent, use_context, use_hook, Runtime};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::JsCast;
use web_sys::window;

pub trait View {
    fn view(&mut self);

    fn remove(&mut self);
}

impl<F, V> View for F
where
    F: FnMut() -> V + Clone + 'static,
    V: View + 'static,
{
    fn view(&mut self) {
        self().view()
    }

    fn remove(&mut self) {
        self().remove()
    }
}

impl View for Box<dyn View> {
    fn view(&mut self) {
        (&mut **self).view()
    }

    fn remove(&mut self) {
        (&mut **self).remove()
    }
}

impl View for Rc<RefCell<dyn View>> {
    fn view(&mut self) {
        self.borrow_mut().view()
    }

    fn remove(&mut self) {
        self.borrow_mut().remove()
    }
}

impl View for () {
    fn view(&mut self) {}

    fn remove(&mut self) {}
}




impl View for String {
    fn view(&mut self) {
        let parent = use_context::<Parent>()
            .map(|cx| cx.0.clone())
            .unwrap_or_else(|| {
                window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .body()
                    .unwrap()
                    .unchecked_into()
            });

        let elem = use_hook(|| {
            let elem = window().unwrap().document().unwrap().create_text_node(self);
            parent.append_child(&elem).unwrap();
            Parent(elem.unchecked_into())
        })
        .0
        .clone();

        elem.set_text_content(Some(self));

        let document = web_sys::window().unwrap().document().unwrap();
        document.create_text_node(self);
    }

    fn remove(&mut self) {
        todo!()
    }
}

impl View for &'static str {
    fn view(&mut self) {
        let parent = use_context::<Parent>()
            .map(|cx| cx.0.clone())
            .unwrap_or_else(|| {
                window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .body()
                    .unwrap()
                    .unchecked_into()
            });

        let elem = use_hook(|| {
            let elem = window().unwrap().document().unwrap().create_text_node(self);
            parent.append_child(&elem).unwrap();
            Parent(elem.unchecked_into())
        })
        .0
        .clone();

        elem.set_text_content(Some(self));

        let document = web_sys::window().unwrap().document().unwrap();
        document.create_text_node(self);
    }

    fn remove(&mut self) {
        todo!()
    }
}

macro_rules! impl_view_for_tuple {
    ( $( $name:ident ),+ ) => {
        impl<$($name: View + Clone + 'static),+> View for ($($name,)+)
        {
            fn view(&mut self) {
                let ($($name,)+) = self;
                $(Runtime::current().spawn($name.clone()));+
            }

            fn remove(&mut self) {
                let ($($name,)+) = self;
                $($name.remove();)+
            }
        }
    };
}

impl_view_for_tuple!(A);
impl_view_for_tuple!(A, B);
impl_view_for_tuple!(A, B, C);
impl_view_for_tuple!(A, B, C, D);
impl_view_for_tuple!(A, B, C, D, E);
impl_view_for_tuple!(A, B, C, D, E, F);
impl_view_for_tuple!(A, B, C, D, E, F, G);
impl_view_for_tuple!(A, B, C, D, E, F, G, H);
impl_view_for_tuple!(A, B, C, D, E, F, G, H, I);
impl_view_for_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_view_for_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_view_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_view_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_view_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_view_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_view_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);

