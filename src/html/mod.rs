use crate::{Composable, IntoComposable};
use std::borrow::Cow;

pub trait Platform: Sized {
    fn html(&mut self, html: &mut Builder) -> impl IntoComposable;
}

#[derive(Default, PartialEq, Eq)]
pub struct Builder {
    pub attrs: Vec<(Cow<'static, str>, Cow<'static, str>)>,
}

#[derive(PartialEq, Eq)]
pub struct Html<P> {
    platform: P,
    builder: Builder,
}

impl<P> Html<P> {
    pub fn new(platform: P) -> Self {
        Self {
            platform,
            builder: Builder::default(),
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
}

impl<P> Composable for Html<P>
where
    P: Platform + PartialEq + 'static,
{
    fn compose(&mut self) -> impl IntoComposable {
        self.platform.html(&mut self.builder)
    }
}
