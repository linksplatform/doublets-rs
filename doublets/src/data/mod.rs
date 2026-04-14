//! Core data types and traits for the doublets link model.
//!
//! This module contains the fundamental building blocks: the [`Link`] triple,
//! the [`Doublet`] pair, the [`Doublets`] / [`DoubletsExt`] traits that define
//! the full store API, and the [`Error`] and [`Fuse`] utility types.

mod doublet;
mod error;
mod handler;
mod link;
mod traits;

pub use doublet::Doublet;
pub use error::Error;
pub use handler::Fuse;
pub use link::Link;
pub use traits::{Doublets, DoubletsExt, Links, ReadHandler, WriteHandler};

#[cfg(feature = "data")]
pub use data::*;
