use std::sync::Arc;

use crate::{Composer, Operation};
use tokio::{
    sync::{
        mpsc::{self, Receiver},
        Mutex,
    },
    task,
};

pub struct Composition<T, U> {
    pub composers: Vec<Arc<Mutex<Composer<T, U>>>>,
}

impl<T, U> Composition<T, U>
where
    T: Send + 'static,
    U: Send + 'static,
{
    pub fn new() -> Self {
        Self {
            composers: Vec::new(),
        }
    }

    pub fn recompose(&mut self) -> Receiver<Vec<Operation<T, U>>> {
        let (tx, rx) = mpsc::channel(self.composers.len());

        for composer in &self.composers {
            let handle = composer.clone();
            let tx = tx.clone();

            task::spawn(async move {
                let mut composer = handle.lock().await;
                let operations = composer.recompose().await;
                tx.send(operations).await.ok().unwrap();
            });
        }
        rx
    }
}
