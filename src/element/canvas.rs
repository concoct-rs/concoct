use super::Element;
use crate::{ElementKey, LayoutContext, UserEvent};
use futures_signals::signal::{ReadOnlyMutable, SignalExt};
use skia_safe::Rect;
use slotmap::DefaultKey;
use taffy::{prelude::Layout, style::Style, Taffy};
use winit::event_loop::EventLoopProxy;

/// Canvas element.
/// This lets you draw directly to the skia canvas.
pub struct Canvas<S, F> {
    layout_key: Option<DefaultKey>,
    pub state: ReadOnlyMutable<S>,
    pub draw: F,
    pub style: Style,
}

impl<S, F> Canvas<S, F>
where
    F: FnMut(S, &Layout, &mut skia_safe::Canvas),
{
    /// Create a new canvas element that will draw its content with the given function.
    pub fn new(state: ReadOnlyMutable<S>, draw: F) -> Self {
        Self {
            state,
            draw,
            layout_key: None,
            style: Style::default(),
        }
    }
}

impl<S, F> Element for Canvas<S, F>
where
    F: FnMut(S, &Layout, &mut skia_safe::Canvas),
    S: Clone + 'static,
{
    fn spawn(&mut self, key: ElementKey, proxy: EventLoopProxy<UserEvent>) {
        let state = self.state.clone();
        tokio::task::spawn_local(async move {
            state
                .signal_cloned()
                .for_each(|_state| async {
                    proxy.send_event(UserEvent::Update(key)).ok().unwrap();
                })
                .await;
        });
    }

    fn layout(&mut self, key: ElementKey, cx: LayoutContext) {
        let layout_key = cx.insert(key, self.style.clone());
        self.layout_key = Some(layout_key);
    }

    fn semantics(&mut self, _taffy: &Taffy) {
        todo!()
    }

    fn paint(&mut self, taffy: &Taffy, canvas: &mut skia_safe::Canvas) {
        let layout = taffy.layout(self.layout_key.unwrap()).unwrap();
        canvas.save();
        canvas.clip_rect(
            Rect::new(
                layout.location.x,
                layout.location.y,
                layout.location.x + layout.size.width,
                layout.location.y + layout.size.height,
            ),
            None,
            None,
        );
        canvas.translate((layout.location.x, layout.location.y));

        (self.draw)(self.state.get_cloned(), layout, canvas);

        canvas.restore();
    }

    fn children(&mut self, _children: &mut Vec<ElementKey>) {}
}
