use super::{Chain, ModifyExt};
use crate::{
    composable::interaction_source::InteractionSource, semantics::Handler, Modify, Semantics, Composable,
};
use accesskit::{NodeId, Role};

pub mod clickable;
use clickable::ClickHandler;

pub mod keyboard_input;
use self::{
    clickable::ClickInteration,
    keyboard_input::{KeyboardHandler, KeyboardInputHandler},
    scrollable::Scrollable,
};

pub mod scrollable;

pub trait HandlerModifier: Modify {
    fn handler<H>(self, handler: H) -> Chain<Self, ModifierHandler<H>>
    where
        Self: Sized,
        H: Handler + 'static,
    {
        self.chain(ModifierHandler {
            handler: Some(handler),
        })
    }

    fn clickable<F>(
        self,
        _role: Role,
        on_click: F,
    ) -> Chain<Self, ModifierHandler<ClickHandler<(), F>>>
    where
        Self: Sized,
        F: FnMut() + 'static,
    {
        self.handler(ClickHandler::new((), on_click))
    }

    fn clickable_interaction<F, I>(
        self,
        _role: Role,
        on_click: F,
        interaction_source: I,
    ) -> Chain<Self, ModifierHandler<ClickHandler<I, F>>>
    where
        Self: Sized,
        F: Composable + 'static,
        I: InteractionSource<ClickInteration> + 'static,
    {
        self.handler(ClickHandler::new(interaction_source, on_click))
    }

    fn keyboard_handler<H>(
        self,
        handler: H,
    ) -> Chain<Self, ModifierHandler<KeyboardInputHandler<H>>>
    where
        Self: Sized,
        H: KeyboardHandler + 'static,
    {
        self.handler(KeyboardInputHandler::new(handler))
    }

    /// Detect scroll gestures without offsetting contents
    fn scrollable<F>(self, on_delta: F) -> Chain<Self, ModifierHandler<Scrollable<(), F>>>
    where
        Self: Sized,
        F: FnMut(f64) + 'static,
    {
        self.handler(Scrollable::new((), on_delta))
    }

    /// Detect scroll gestures with an interaction source, without offsetting contents
    fn scrollable_interaction<F, I>(
        self,
        on_delta: F,
        interaction_source: I,
    ) -> Chain<Self, ModifierHandler<Scrollable<I, F>>>
    where
        Self: Sized,
        F: FnMut(f64) + 'static,
        I: InteractionSource<f64> + 'static,
    {
        self.handler(Scrollable::new(interaction_source, on_delta))
    }
}

impl<M> HandlerModifier for M where M: Modify {}

pub struct ModifierHandler<H> {
    handler: Option<H>,
}

impl<H> Modify for ModifierHandler<H>
where
    H: Handler + 'static,
{
    fn semantics(&mut self, node_id: NodeId, semantics: &mut Semantics) {
        if let Some(handler) = self.handler.take() {
            // TODO allow updates
            if !semantics.handlers.contains_key(&node_id) {
                semantics.handlers.insert(node_id, Box::new(handler));
            }
        }
    }

    fn remove(&mut self, node_id: NodeId, semantics: &mut Semantics) {
        semantics.handlers.remove(&node_id);
    }
}
