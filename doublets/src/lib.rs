#![feature(fn_traits)]
#![feature(generators)]
#![feature(try_trait_v2)]
#![feature(default_free_fn)]
#![feature(unboxed_closures)]
#![feature(nonnull_slice_from_raw_parts)]
#![feature(associated_type_defaults)]
#![feature(type_alias_impl_trait)]

pub mod data;
pub mod mem;

pub use self::mem::{parts, split, unit};

pub use self::data::{Doublet, Doublets, DoubletsExt, Error, FuseHandler, Handler, Link, Links};
pub(crate) use self::data::{Error as LinksError, ReadHandler, WriteHandler};
