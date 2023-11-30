use web_sys::Node;

use crate::{use_context, Child, IntoView, View};
use std::{borrow::Cow, cell::RefCell, rc::Rc};

pub trait HtmlPlatform: Sized {
    fn html<C: IntoView>(
        &mut self,
        html: &mut Builder,
        parent: Option<Node>,
        child: Child<C>,
    ) -> impl IntoView;
}

pub enum AttributeValue {
    String(Cow<'static, str>),
    Callback(Rc<RefCell<dyn FnMut()>>),
}

impl PartialEq for AttributeValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::String(l0), Self::String(r0)) => l0 == r0,
            (Self::Callback(_l0), Self::Callback(_r0)) => true,
            _ => false,
        }
    }
}

#[derive(Default, PartialEq)]
pub struct Builder {
    pub attrs: Vec<(Cow<'static, str>, AttributeValue)>,
}

pub struct Html<P, C> {
    platform: P,
    builder: Builder,
    child: Child<C>,
}

impl<P, C> Html<P, C> {
    pub fn new(platform: P, child: C) -> Self {
        Self {
            platform,
            builder: Builder::default(),
            child: Child::new(child),
        }
    }

    pub fn attr(
        mut self,
        name: impl Into<Cow<'static, str>>,
        value: impl Into<Cow<'static, str>>,
    ) -> Self {
        self.builder
            .attrs
            .push((name.into(), AttributeValue::String(value.into())));
        self
    }

    pub fn on_click(mut self, callback: impl FnMut() + 'static) -> Self {
        self.builder.attrs.push((
            Cow::Borrowed("click"),
            AttributeValue::Callback(Rc::new(RefCell::new(callback))),
        ));
        self
    }
}

impl<P, C> PartialEq for Html<P, C> {
    fn eq(&self, other: &Self) -> bool {
        self.builder == other.builder
    }
}

pub struct HtmlParent {
    pub(crate) node: Node,
}

impl<P, C> View for Html<P, C>
where
    P: HtmlPlatform + 'static,
    C: IntoView,
{
    fn view(&mut self) -> impl IntoView {
        let parent = use_context::<HtmlParent>().map(|parent| parent.get().node.clone());

        self.platform
            .html(&mut self.builder, parent, self.child.clone())
    }
}
