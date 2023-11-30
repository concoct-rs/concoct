use crate::{AnyView, Hook};
use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
};

pub struct Node {
    pub(crate) make_view: Box<dyn FnMut() -> Box<dyn AnyView>>,
    pub(crate) view: Option<Rc<RefCell<Box<dyn AnyView>>>>,
    pub(crate) hooks: Rc<RefCell<Vec<Hook>>>,
    pub(crate) contexts: HashMap<TypeId, Rc<dyn Any>>,
    pub(crate) on_drops: Rc<RefCell<Vec<Box<dyn FnMut()>>>>,
}

impl Drop for Node {
    fn drop(&mut self) {
        for on_drop in &mut *self.on_drops.borrow_mut() {
            on_drop()
        }
    }
}
