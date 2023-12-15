//! Thread-safe channels.

use crate::{Context, Object, Signal};
use futures::{channel::mpsc, StreamExt};

/// Create an unbounded channel between two objects.
pub fn unbounded<M: 'static>() -> (UnboundedChannel<M>, UnboundedChannel<M>) {
    let (a_tx, a_rx) = mpsc::unbounded();
    let (b_tx, b_rx) = mpsc::unbounded();
    (UnboundedChannel { tx: a_tx, rx: b_rx }, UnboundedChannel { tx: b_tx, rx: a_rx })
}

/// A channel between two objects.
pub struct UnboundedChannel<M> {
    tx: mpsc::UnboundedSender<M>,
    rx: mpsc::UnboundedReceiver<M>,
}

impl<M: 'static> UnboundedChannel<M> {
    /// Send a message to the object on the other end of the channel.
    pub fn send(&self, msg: M) {
        self.tx.unbounded_send(msg).unwrap();
    }

    /// Receive a message from the object on the other end of the channel.
    pub async fn recv(cx: &mut Context<'_, Self>) {
        let msg = cx.rx.next().await.unwrap();
        cx.emit(msg);
    }
}

impl<M> Object for UnboundedChannel<M> {}

impl<M: 'static> Signal<M> for UnboundedChannel<M> {}
