//! A doublets graph database library for working with associative link structures.
//!
//! Doublets stores data as a graph of typed links, each represented as a triple
//! `(index, source, target)`. Two store implementations are provided:
//!
//! - [`unit::Store`] — a single-memory store for smaller graphs.
//! - [`split::Store`] — a two-memory (data + index) store that separates link
//!   data from tree-index metadata for larger graphs.
//!
//! # Quick start
//!
//! ```rust,ignore
//! use doublets::{unit, Doublets, DoubletsExt};
//! use mem::FileMapped;
//!
//! let store = unit::Store::<u64, _>::new(FileMapped::from_path("db.links")?)?;
//! ```

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
#![allow(
    clippy::needless_pass_by_value,
    clippy::comparison_chain,
    clippy::transmute_ptr_to_ptr,
    clippy::ref_as_ptr,
    clippy::borrow_as_ptr,
    clippy::missing_const_for_fn,
    clippy::needless_pass_by_ref_mut,
    clippy::non_send_fields_in_send_ty,
    clippy::too_many_lines,
    clippy::redundant_pub_crate,
    clippy::implied_bounds_in_impls,
    clippy::only_used_in_recursion,
    clippy::option_if_let_else,
    clippy::assign_op_pattern,
    dead_code
)]

pub mod data;
pub mod mem;

pub(crate) fn funty<T: data::LinkType>(n: u8) -> T {
    T::try_from(n).expect("conversion from u8 should succeed for all unsigned types")
}

pub(crate) use trees::LinkType as TreesLinkType;

pub use self::mem::{parts, split, unit};

pub use self::data::{Doublet, Doublets, DoubletsExt, Error, Fuse, Link, Links};
pub(crate) use self::data::{Error as LinksError, ReadHandler, WriteHandler};
