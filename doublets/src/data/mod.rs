mod doublet;
mod error;
mod handler;
mod link;
mod traits;

// Stable Rust fallback when platform dependencies unavailable
#[cfg(not(feature = "data"))]
mod stable_fallback;

pub use doublet::Doublet;
pub use error::Error;
pub use handler::{Fuse, Handler};
pub use link::Link;
pub use traits::{Doublets, DoubletsExt, Links, ReadHandler, WriteHandler};

#[cfg(feature = "data")]
pub use data::*;

#[cfg(not(feature = "data"))]
pub use stable_fallback::{Flow, LinkType};
