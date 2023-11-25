use crate::{Composable, IntoComposable};
use std::borrow::Cow;

pub trait HtmlPlatform: Sized {
    fn html(&mut self, html: &mut Builder) -> impl IntoComposable;
}

#[derive(Default, PartialEq, Eq)]
pub struct Builder {
    pub attrs: Vec<(Cow<'static, str>, Cow<'static, str>)>,
}

pub struct Html<P, C> {
    platform: P,
    builder: Builder,
    child: C,
}

impl<P, C> Html<P, C> {
    pub fn new(platform: P, child: C) -> Self {
        Self {
            platform,
            builder: Builder::default(),
            child,
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

impl<P, C> IntoComposable for Html<P, C>
where
    P: HtmlPlatform + 'static,
    C: IntoComposable,
{
    fn into_composer(self) -> impl Composable {
        self.child.into_composer()
    }
}
