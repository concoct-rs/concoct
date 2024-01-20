use std::borrow::Cow;

use std::cell::RefCell;

pub mod hook;

mod rt;
pub(crate) use self::rt::{Runtime, Scope};

mod tree;
pub(crate) use tree::Node;
pub use tree::Tree;

mod vdom;
pub use self::vdom::{virtual_dom, VirtualDom};

mod view_builder;
pub use self::view_builder::ViewBuilder;

pub mod view;
pub use self::view::View;

pub async fn run(view: impl ViewBuilder) {
    let mut vdom = virtual_dom(view);
    vdom.build();

    loop {
        vdom.rebuild().await
    }
}

pub struct TextViewContext {
    view: RefCell<Box<dyn FnMut(Cow<'static, str>)>>,
}

impl TextViewContext {
    pub fn new(view: impl FnMut(Cow<'static, str>) + 'static) -> Self {
        Self {
            view: RefCell::new(Box::new(view)),
        }
    }
}
