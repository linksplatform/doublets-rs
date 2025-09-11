// Nightly features are only enabled when the "nightly" feature is active
#![cfg_attr(feature = "nightly", feature(fn_traits))]
#![cfg_attr(feature = "nightly", feature(generators))]
#![cfg_attr(feature = "nightly", feature(try_trait_v2))]
#![cfg_attr(feature = "nightly", feature(default_free_fn))]
#![cfg_attr(feature = "nightly", feature(unboxed_closures))]
#![cfg_attr(feature = "nightly", feature(nonnull_slice_from_raw_parts))]
#![cfg_attr(feature = "nightly", feature(associated_type_defaults))]
#![cfg_attr(feature = "nightly", feature(type_alias_impl_trait))]
#![cfg_attr(feature = "nightly", feature(maybe_uninit_uninit_array))]
#![cfg_attr(feature = "nightly", feature(allocator_api))]
#![cfg_attr(feature = "nightly", feature(maybe_uninit_array_assume_init))]
#![cfg_attr(not(test), forbid(clippy::unwrap_used))]
#![warn(
    clippy::perf,
    clippy::single_match_else,
    clippy::dbg_macro,
    clippy::doc_markdown,
    clippy::wildcard_imports,
    clippy::struct_excessive_bools,
    clippy::semicolon_if_nothing_returned,
    clippy::pedantic,
    clippy::nursery
)]
// for `clippy::pedantic`
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::missing_safety_doc
)]
#![deny(
    clippy::all,
    clippy::cast_lossless,
    clippy::redundant_closure_for_method_calls,
    clippy::use_self,
    clippy::unnested_or_patterns,
    clippy::trivially_copy_pass_by_ref,
    clippy::needless_pass_by_value,
    clippy::match_wildcard_for_single_variants,
    clippy::map_unwrap_or,
    unused_qualifications,
    unused_import_braces,
    unused_lifetimes,
    unreachable_pub,
    trivial_numeric_casts,
    // rustdoc,
    // missing_debug_implementations,
    // missing_copy_implementations,
    deprecated_in_future,
    meta_variable_misuse,
    non_ascii_idents,
    rust_2018_compatibility,
    rust_2018_idioms,
    future_incompatible,
    nonstandard_style,
)]
// must be fixed later
#![allow(clippy::needless_pass_by_value, clippy::comparison_chain)]

// Full functionality with nightly features and platform dependencies  
#[cfg(all(feature = "data", feature = "mem"))]
pub mod data;

#[cfg(all(feature = "data", feature = "mem"))]
pub mod mem;

#[cfg(all(feature = "data", feature = "mem"))]
pub use self::mem::{parts, split, unit};

#[cfg(all(feature = "data", feature = "mem"))]
pub use self::data::{Doublet, Doublets, DoubletsExt, Error, Fuse, Handler, Link, Links};

#[cfg(all(feature = "data", feature = "mem"))]
pub(crate) use self::data::{Error as LinksError, ReadHandler, WriteHandler};

// Stable Rust functionality - minimal but working
#[cfg(not(all(feature = "data", feature = "mem")))]
pub mod stable_lib;

#[cfg(not(all(feature = "data", feature = "mem")))]
pub use stable_lib::{StableDoublets, StableError, StableLink, StableMemoryStore};
