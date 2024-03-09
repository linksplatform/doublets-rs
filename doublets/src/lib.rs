#![feature(
    fn_traits,
    try_trait_v2,
    unboxed_closures,
    associated_type_defaults,
    type_alias_impl_trait,
    maybe_uninit_uninit_array,
    allocator_api,
    maybe_uninit_array_assume_init,
    impl_trait_in_assoc_type,
    never_type
)]
#![feature(let_chains)]
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
#![allow(clippy::missing_errors_doc, clippy::missing_panics_doc, clippy::missing_safety_doc)]
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
    // unreachable_pub,
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
#![allow(clippy::needless_pass_by_value, clippy::comparison_chain, irrefutable_let_patterns)]

pub mod data;
pub mod mem;

pub use data::Link;

//pub use self::mem::{parts, split, unit};

//pub use self::data::{Doublet, Doublets, DoubletsExt, Error, Fuse, Handler, Link, Links};
//pub(crate) use self::data::{Error as LinksError, ReadHandler, WriteHandler};
