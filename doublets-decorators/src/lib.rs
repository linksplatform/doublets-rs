//! Compatibility names for the stable decorators in [`doublets::decorators`].
//!
//! Existing callers can retain the six original type names and `new(store)`
//! constructors. Each export uses the current implementation and callback API:
//! callbacks return [`doublets::data::Flow`] instead of the unstable `Try` trait.

pub use doublets::decorators::{
    CascadeUniquenessAndUsagesResolver as CascadeUniqueResolver, CascadeUsagesResolver,
    NonNullContentsLinkDeletionResolver as NonNullDeletionResolver,
    UniquenessResolver as UniqueResolver, UniquenessValidator as UniqueValidator, UsagesValidator,
};
