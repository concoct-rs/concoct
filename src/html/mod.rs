use web_sys::Node;

use crate::{use_context, Child, IntoView, View, AnyChild};
use std::{borrow::Cow, cell::RefCell, rc::Rc};

pub trait HtmlPlatform: Sized {
    fn html(
        &mut self,
        html: &mut Builder,
        parent: Option<Node>,
        child: Option<AnyChild>,
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
    pub tag: Cow<'static, str>,
    pub attrs: Vec<(Cow<'static, str>, AttributeValue)>,
}

#[derive(PartialEq)]
pub struct Html<P> {
    platform: P,
    builder: Builder,
    child: Option<AnyChild>,
}

impl<P> Html<P> {
    pub fn new(tag: impl Into<Cow<'static, str>>, platform: P) -> Self {
        Self {
            platform,
            builder: Builder {
                tag: tag.into(),
                attrs: Vec::new(),
            },
            child: None
        }
    }

    pub fn child(mut self, view: impl IntoView) -> Self {
        self.child = Some(AnyChild::new(view));
        self
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

pub struct HtmlParent {
    pub(crate) node: Node,
}

impl<P> View for Html<P>
where
    P: HtmlPlatform + PartialEq + 'static,

{
    fn view(&mut self) -> impl IntoView {
        let parent = use_context::<HtmlParent>().map(|parent| parent.get().node.clone());

        self.platform
            .html(&mut self.builder, parent, self.child.clone())
    }
}
