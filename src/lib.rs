use std::sync::Arc;
use tokio::sync::mpsc;

pub mod composable;
pub use composable::Composable;

pub struct Context<M> {
    send: Arc<dyn Fn(M) + Send + Sync>,
}

impl<M> Context<M> {
    pub fn new(send: Arc<dyn Fn(M) + Send + Sync>) -> Self {
        Self { send }
    }

    pub fn send(&self, msg: M) {
        (self.send)(msg)
    }
}

impl<M> Clone for Context<M> {
    fn clone(&self) -> Self {
        Self {
            send: self.send.clone(),
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
    rx: mpsc::UnboundedReceiver<M>,
}

impl<T, F, S, M> Composer<T, F, S, M> {
    pub fn new(model: T, composable: F) -> Self
    where
        M: Send + 'static,
    {
        let (tx, rx) = mpsc::unbounded_channel();
        let cx = Context::new(Arc::new(move |msg| {
            tx.send(msg).unwrap();
        }));
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

    pub  fn try_handle(&mut self)
    where
        T: Model<M>,
        M: 'static,
    {
        if let Ok(msg) = self.rx.try_recv() {
            self.model.handle(msg);
        }
       
    }
}
