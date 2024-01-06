use std::{any::Any, marker::PhantomData, sync::Arc};
use tokio::sync::mpsc;

pub struct Receiver<M> {
    rx: mpsc::UnboundedReceiver<Box<dyn Any>>,
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
    mapper: Option<Arc<dyn Fn(M) -> Box<dyn Any>>>,
    tx: mpsc::UnboundedSender<Box<dyn Any>>,
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
        M: 'static,
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

pub trait Composable<M> {
    type State;

    fn compose(&mut self, cx: &mut Context<M>) -> Self::State;

    fn recompose(&mut self, cx: &mut Context<M>, state: &mut Self::State);

    fn map<F, M1>(self, f: F) -> Map<Self, F, M>
    where
        Self: Sized,
        F: Fn(M) -> M1 + 'static,
        M1: 'static,
    {
        Map {
            view: self,
            f: Arc::new(f),
            _marker: PhantomData,
        }
    }
}

impl<M> Composable<M> for () {
    type State = ();

    fn compose(&mut self, cx: &mut Context<M>) -> Self::State {}

    fn recompose(&mut self, cx: &mut Context<M>, state: &mut Self::State) {}
}

impl<M, C1: Composable<M>, C2: Composable<M>> Composable<M> for (C1, C2) {
    type State = (C1::State, C2::State);

    fn compose(&mut self, cx: &mut Context<M>) -> Self::State {
        (self.0.compose(cx), self.1.compose(cx))
    }

    fn recompose(&mut self, cx: &mut Context<M>, state: &mut Self::State) {
        self.0.recompose(cx, &mut state.0);
        self.1.recompose(cx, &mut state.1);
    }
}

pub struct Map<V, F, M> {
    view: V,
    f: Arc<F>,
    _marker: PhantomData<M>,
}

impl<V, F, M1, M2> Composable<M1> for Map<V, F, M2>
where
    V: Composable<M2>,
    F: Fn(M2) -> M1 + 'static,
    M1: 'static,
{
    type State = V::State;

    fn compose(&mut self, cx: &mut Context<M1>) -> Self::State {
        let f = self.f.clone();
        let mapper: Arc<dyn Fn(M2) -> Box<dyn Any>> = Arc::new(move |msg| Box::new((f)(msg)));
        let mut cx = Context {
            mapper: Some(mapper),
            tx: cx.tx.clone(),
        };
        self.view.compose(&mut cx)
    }

    fn recompose(&mut self, cx: &mut Context<M1>, state: &mut Self::State) {
        let f = self.f.clone();
        let mapper: Arc<dyn Fn(M2) -> Box<dyn Any>> = Arc::new(move |msg| Box::new((f)(msg)));
        let mut cx = Context {
            mapper: Some(mapper),
            tx: cx.tx.clone(),
        };
        self.view.recompose(&mut cx, state)
    }
}

pub fn from_fn<F, M>(f: F) -> FromFn<F>
where
    F: FnMut(&mut Context<M>),
{
    FromFn { f }
}

pub struct FromFn<F> {
    f: F,
}

impl<M, F> Composable<M> for FromFn<F>
where
    F: FnMut(&mut Context<M>),
{
    type State = ();

    fn compose(&mut self, cx: &mut Context<M>) -> Self::State {
        (self.f)(cx)
    }

    fn recompose(&mut self, cx: &mut Context<M>, state: &mut Self::State) {
        (self.f)(cx)
    }
}
