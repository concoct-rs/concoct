use std::{marker::Unpin, panic::Location};

use futures::{Stream, StreamExt};
use slotmap::DefaultKey;
use tokio::task::JoinHandle;

use crate::{render::UserEvent, Composer, Widget};

#[track_caller]
pub fn stream<
    T: Send + 'static,
    S: Stream<Item = T> + Send + Unpin + 'static,
    F: FnMut(T) + 'static,
>(
    stream: S,
    on_item: F,
) {
    let location = Location::caller();
    Composer::with(|composer| {
        let mut cx = composer.borrow_mut();
        let id = cx.id(location);

        if let Some(widget) = cx.get_mut::<StreamWidget<T, S>>(&id) {
            cx.children.push(id);
        } else {
            let widget = StreamWidget {
                on_item: Some(Box::new(on_item)),
                stream: Some(stream),
                task: None,
            };
            cx.insert(id, widget, None);
        }
    });
}

pub struct StreamWidget<T, S> {
    on_item: Option<Box<dyn FnMut(T)>>,
    stream: Option<S>,
    task: Option<(DefaultKey, JoinHandle<()>)>,
}

impl<T, S> Widget for StreamWidget<T, S>
where
    T: Send + 'static,
    S: Stream<Item = T> + Send + 'static + Unpin,
{
    fn layout(&mut self, semantics: &mut crate::Semantics) {
        if let Some(mut on_item) = self.on_item.take() {
            let task_id = semantics.tasks.insert(Box::new(move |item| {
                let item = item.downcast().unwrap();
                on_item(*item);
            }));


            let proxy = semantics.proxy.as_ref().unwrap().clone();
            let mut stream = self.stream.take().unwrap();
            let handle = tokio::spawn(async move {
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

    fn semantics(&mut self, semantics: &mut crate::Semantics) {}

    fn paint(&mut self, semantics: &crate::Semantics, canvas: &mut skia_safe::Canvas) {}

    fn remove(&mut self, semantics: &mut crate::Semantics) {
        let (task_id, handle) = self.task.as_ref().unwrap();
      
        semantics.tasks.remove(*task_id);
        handle.abort();
    }

    fn any(&self) -> &dyn std::any::Any {
        self
    }

    fn any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
