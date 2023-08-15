mod renderer;

pub use renderer::{Event, Renderer};

pub mod element;

mod tree;
pub use tree::{ElementKey, LayoutContext, Tree};

pub mod view;

pub enum UserEvent {
    Update(ElementKey),
}

pub struct Id {}
