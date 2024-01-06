use std::{any::Any, marker::PhantomData, sync::Arc};
use tokio::sync::mpsc;

pub mod composable;
pub use composable::Composable;

pub struct Receiver<M> {
    rx: mpsc::UnboundedReceiver<Box<dyn Any + Send>>,
    _marker: PhantomData<M>,
}

impl<M> Receiver<M> {
    pub async fn recv(&mut self) -> Option<M>
    where
        M: 'static,
    {
        self.rx.recv().await.map(|any| *any.downcast().unwrap())
    }
}

type Mapper<M> = Arc<dyn Fn(M) -> Box<dyn Any + Send> + Send + Sync>;

pub struct Context<M> {
    mapper: Option<Mapper<M>>,
    tx: mpsc::UnboundedSender<Box<dyn Any + Send>>,
}

impl<M> Context<M> {
    pub fn new() -> (Self, Receiver<M>) {
        let (tx, rx) = mpsc::unbounded_channel();
        (
            Self { mapper: None, tx },
            Receiver {
                rx,
                _marker: PhantomData,
            },
        )
    }

    pub fn send(&self, msg: M)
    where
        M: Send + 'static,
    {
        let boxed = if let Some(mapper) = self.mapper.as_ref() {
            mapper(msg)
        } else {
            Box::new(msg)
        };
        self.tx.send(boxed).unwrap();
    }
}

impl<M> Clone for Context<M> {
    fn clone(&self) -> Self {
        Self {
            mapper: self.mapper.clone(),
            tx: self.tx.clone(),
        }
    }
}

pub trait Model<M> {
    fn handle(&mut self, msg: M);
}

pub struct Composer<T, F, S, M> {
    model: T,
    composable: F,
    state: Option<S>,
    cx: Context<M>,
    rx: Receiver<M>,
}

impl<T, F, S, M> Composer<T, F, S, M> {
    pub fn new(model: T, composable: F) -> Self {
        let (cx, rx) = Context::new();
        Self {
            model,
            composable,
            state: None,
            cx,
            rx,
        }
    }

    pub fn compose<C>(&mut self)
    where
        T: Model<M>,
        F: FnMut(&T) -> C,
        C: Composable<M, State = S>,
    {
        let state = (self.composable)(&self.model).compose(&mut self.cx);
        self.state = Some(state);
    }

    pub fn recompose<C>(&mut self)
    where
        T: Model<M>,
        F: FnMut(&T) -> C,
        C: Composable<M, State = S>,
    {
        let state = self.state.as_mut().unwrap();
        (self.composable)(&self.model).recompose(&mut self.cx, state);
    }

    pub async fn handle(&mut self)
    where
        T: Model<M>,
        M: 'static,
    {
        let msg = self.rx.recv().await.unwrap();
        self.model.handle(msg);
    }
}
