use crate::{html::Parent, use_context, use_hook, Runtime};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::JsCast;
use web_sys::window;

pub trait View {
    fn view(&mut self);
}

impl<F, V> View for F
where
    F: FnMut() -> V + Clone + 'static,
    V: View + 'static,
{
    fn view(&mut self) {
        self().view()
    }
}

impl View for Box<dyn View> {
    fn view(&mut self) {
        (&mut **self).view()
    }
}

impl View for Rc<RefCell<dyn View>> {
    fn view(&mut self) {
        self.borrow_mut().view()
    }
}

impl View for () {
    fn view(&mut self) {}
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
}

impl<V> View for Option<V>
where
    V: View + Clone + 'static,
{
    fn view(&mut self) {
        let mut last = use_hook(|| self.clone());
        let mut handle = use_hook(|| None);

        if let Some(view) = self {
            *handle = Some(Runtime::current().spawn(view.clone()));
        } else if let Some(_) = &*last {
            *last = None;
            *handle = None;
        }
    }
}

macro_rules! impl_view_for_tuple {
    ( $( $name:ident ),+ ) => {
        impl<$($name: View + Clone + 'static),+> View for ($($name,)+)
        {
            fn view(&mut self) {
                let ($($name,)+) = self;
                use_hook(|| ($(Runtime::current().spawn($name.clone()),)+));
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
