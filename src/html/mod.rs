use web_sys::Node;

use crate::{use_context, Child, IntoView, View};
use std::borrow::Cow;

pub trait HtmlPlatform: Sized {
    fn html<C: IntoView>(
        &mut self,
        html: &mut Builder,
        parent: Option<Node>,
        child: Child<C>,
    ) -> impl IntoView;
}

#[derive(Default, PartialEq, Eq)]
pub struct Builder {
    pub attrs: Vec<(Cow<'static, str>, Cow<'static, str>)>,
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
        self.builder.attrs.push((name.into(), value.into()));
        self
    }

    pub fn on_click(self, _f: impl FnMut()) -> Self {
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
