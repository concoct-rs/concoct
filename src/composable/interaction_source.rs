use super::{
    state::{state, State},
    stream,
};
use futures::StreamExt;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

#[track_caller]
pub fn interaction_source<I: Clone + 'static>() -> MutableInteractionSource<I> {
    let sender = state(|| {
        let (sender, _) = broadcast::channel(100);
        sender
    });

    MutableInteractionSource { sender }
}

pub trait InteractionSource<I> {
    fn emit(&self, interaction: I);
}

impl<I> InteractionSource<I> for () {
    fn emit(&self, _interaction: I) {}
}

pub struct MutableInteractionSource<I> {
    sender: State<broadcast::Sender<I>>,
}

impl<I> Clone for MutableInteractionSource<I> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl<I> Copy for MutableInteractionSource<I> {}

impl<I: 'static> InteractionSource<I> for MutableInteractionSource<I> {
    fn emit(&self, interaction: I) {
        self.sender.get().as_ref().send(interaction).ok();
    }
}

impl<I: 'static> MutableInteractionSource<I> {
    pub fn receiver(&self) -> broadcast::Receiver<I> {
        self.sender.get().as_ref().subscribe()
    }

    pub fn on_item(&self, on_item: impl FnMut(I) + 'static)
    where
        I: Send + Clone,
    {
        let receiver = self.receiver();
        stream(
            async { BroadcastStream::new(receiver).map(|res| res.unwrap()) },
            on_item,
        )
    }
}
