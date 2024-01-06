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

pub struct Context<M> {
    mapper: Option<Arc<dyn Fn(M) -> Box<dyn Any + Send> + Send + Sync>>,
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
