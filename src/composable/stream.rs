use std::{
    marker::{PhantomData, Unpin},
    panic::Location, any::Any,
};
use futures::{Future, Stream, StreamExt};
use slotmap::DefaultKey;
use tokio::task::JoinHandle;
use crate::{render::UserEvent, Composer, Widget, Semantics};

#[track_caller]
pub fn stream<
    T: Send + 'static,
    Fut: Future<Output = S> + Send + 'static,
    S: Stream<Item = T> + Send + Unpin + 'static,
    F: FnMut(T) + 'static,
>(
    future: Fut,
    on_item: F,
) {
    let location = Location::caller();
    Composer::with(|composer| {
        let mut cx = composer.borrow_mut();
        let id = cx.id(location);

        if let Some(widget) = cx.get_mut::<StreamWidget<T, Fut, S>>(&id) {
            widget.on_item = Some(Box::new(on_item));
            widget.future = Some(future);
            cx.children.push(id);
        } else {
            let widget = StreamWidget {
                on_item: Some(Box::new(on_item)),
                future: Some(future),
                task: None,
                _marker: PhantomData,
            };
            cx.insert(id, widget, None);
        }
    });
}

pub struct StreamWidget<T, Fut, S> {
    on_item: Option<Box<dyn FnMut(T)>>,
    future: Option<Fut>,
    task: Option<(DefaultKey, JoinHandle<()>)>,
    _marker: PhantomData<S>,
}

impl<T, Fut, S> Widget for StreamWidget<T, Fut, S>
where
    T: Send + 'static,
    Fut: Future<Output = S> + Send + 'static,
    S: Stream<Item = T> + Send + 'static + Unpin,
{
    fn layout(&mut self, semantics: &mut Semantics) {
        if let Some(mut on_item) = self.on_item.take() {
            let task_id = semantics.tasks.insert(Box::new(move |item| {
                let item = item.downcast().unwrap();
                on_item(*item);
            }));

            let proxy = semantics.proxy.as_ref().unwrap().clone();
            let future = self.future.take().unwrap();
            let handle = tokio::spawn(async move {
                let mut stream = future.await;
                while let Some(item) = stream.next().await {
                    proxy
                        .send_event(UserEvent::Task {
                            id: task_id,
                            data: Box::new(item),
                        })
                        .unwrap();
                }
            });

            self.task = Some((task_id, handle));
        }
    }

    fn semantics(&mut self, semantics: &mut Semantics) {}

    fn paint(&mut self, semantics: &Semantics, canvas: &mut skia_safe::Canvas) {}

    fn remove(&mut self, semantics: &mut Semantics) {
        let (task_id, handle) = self.task.as_ref().unwrap();

        semantics.tasks.remove(*task_id);
        handle.abort();
    }

    fn any(&self) -> &dyn Any {
        self
    }

    fn any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
