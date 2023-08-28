#![cfg_attr(docsrs, feature(doc_cfg))]

//! A cross-platform framework for efficient user interfaces.
//!
//! Concoct is statically-typed UI library for building applications with Rust
//! that run on multiple platforms.

mod modify;
pub use modify::Modify;

pub mod view;
pub use view::View;

#[cfg(feature = "web")]
#[cfg_attr(docsrs, doc(cfg(feature = "web")))]
pub mod web;
