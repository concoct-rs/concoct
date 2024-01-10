use std::{rc::Rc, task::Poll};

pub trait Wake {
    fn wake(self: Rc<Self>);
}

#[derive(Clone)]
pub struct Waker {
    wake: Rc<dyn Wake>,
}

impl Waker {
    pub fn wake(self) {
        self.wake.wake()
    }
}

impl<W: Wake + 'static> From<Rc<W>> for Waker {
    fn from(value: Rc<W>) -> Self {
        Self { wake: value }
    }
}

pub struct Context<'a> {
    waker: &'a Waker,
}

pub trait Task {
    type Output;

    fn poll(&mut self, cx: &mut Context) -> Poll<Self::Output>;
}
