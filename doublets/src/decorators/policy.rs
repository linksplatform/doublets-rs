//! Compile-time policy selection for the uniqueness and usages decorators.
//!
//! The three markers below are zero-sized types. Selecting one selects a decorator
//! *type*, not a runtime branch, so a policy costs nothing at run time.

use data::LinkReference;

use super::{
    CascadeUniquenessAndUsagesResolver, CascadeUsagesResolver, UniquenessResolver,
    UniquenessValidator, UsagesValidator,
};
use crate::Doublets;

/// Reject the conflicting operation with an error.
///
/// * As a [`UniquenessPolicy`] it selects [`UniquenessValidator`].
/// * As a [`UsagesPolicy`] it selects [`UsagesValidator`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Validate;

/// Resolve the conflict by reusing the link that already exists.
///
/// As a [`UniquenessPolicy`] it selects [`UniquenessResolver`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Resolve;

/// Resolve the conflict and cascade through everything that depends on it.
///
/// * As a [`UniquenessPolicy`] it selects [`CascadeUniquenessAndUsagesResolver`].
/// * As a [`UsagesPolicy`] it selects [`CascadeUsagesResolver`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CascadeResolve;

/// Selects how a store reacts when an update would produce a duplicate
/// `(source, target)` pair.
///
/// Implemented by [`Validate`], [`Resolve`] and [`CascadeResolve`].
pub trait UniquenessPolicy {
    /// The decorator this policy wraps a store in.
    type Decorator<T: LinkReference, L: Doublets<T>>: Doublets<T>;

    /// Wraps `links` in the decorator selected by this policy.
    fn decorate<T: LinkReference, L: Doublets<T>>(links: L) -> Self::Decorator<T, L>;
}

/// Selects how a store reacts when an operation touches a link that other links still
/// reference.
///
/// Implemented by [`Validate`] and [`CascadeResolve`].
pub trait UsagesPolicy {
    /// The decorator this policy wraps a store in.
    type Decorator<T: LinkReference, L: Doublets<T>>: Doublets<T>;

    /// Wraps `links` in the decorator selected by this policy.
    fn decorate<T: LinkReference, L: Doublets<T>>(links: L) -> Self::Decorator<T, L>;
}

impl UniquenessPolicy for Validate {
    type Decorator<T: LinkReference, L: Doublets<T>> = UniquenessValidator<T, L>;

    #[inline]
    fn decorate<T: LinkReference, L: Doublets<T>>(links: L) -> Self::Decorator<T, L> {
        UniquenessValidator::new(links)
    }
}

impl UniquenessPolicy for Resolve {
    type Decorator<T: LinkReference, L: Doublets<T>> = UniquenessResolver<T, L>;

    #[inline]
    fn decorate<T: LinkReference, L: Doublets<T>>(links: L) -> Self::Decorator<T, L> {
        UniquenessResolver::new(links)
    }
}

impl UniquenessPolicy for CascadeResolve {
    type Decorator<T: LinkReference, L: Doublets<T>> = CascadeUniquenessAndUsagesResolver<T, L>;

    #[inline]
    fn decorate<T: LinkReference, L: Doublets<T>>(links: L) -> Self::Decorator<T, L> {
        CascadeUniquenessAndUsagesResolver::new(links)
    }
}

impl UsagesPolicy for Validate {
    type Decorator<T: LinkReference, L: Doublets<T>> = UsagesValidator<T, L>;

    #[inline]
    fn decorate<T: LinkReference, L: Doublets<T>>(links: L) -> Self::Decorator<T, L> {
        UsagesValidator::new(links)
    }
}

impl UsagesPolicy for CascadeResolve {
    type Decorator<T: LinkReference, L: Doublets<T>> = CascadeUsagesResolver<T, L>;

    #[inline]
    fn decorate<T: LinkReference, L: Doublets<T>>(links: L) -> Self::Decorator<T, L> {
        CascadeUsagesResolver::new(links)
    }
}

/// Compile-time proof that the policy markers carry no data.
const _: () = {
    assert!(size_of::<Validate>() == 0);
    assert!(size_of::<Resolve>() == 0);
    assert!(size_of::<CascadeResolve>() == 0);
};
