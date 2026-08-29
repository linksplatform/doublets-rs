//! The builder that composes decorators into a single concrete type.

use std::io::Write;

use data::LinkReference;

use super::{
    policy::{UniquenessPolicy, UsagesPolicy},
    CascadeUniquenessAndUsagesResolver, CascadeUsagesResolver, InnerReferenceExistenceValidator,
    ItselfConstantToSelfReferenceResolver, LoggingDecorator, NoExceptionsDecorator,
    NonExistentDependenciesCreator, NonNullContentsLinkDeletionResolver,
    NullConstantToSelfReferenceResolver, UsagesValidator,
};
use crate::Doublets;

/// The stack C# `ILinksExtensions.DecorateWithAutomaticUniquenessAndUsagesResolution`
/// builds, spelled as a type.
pub type AutomaticUniquenessAndUsagesResolution<T, L> = CascadeUniquenessAndUsagesResolver<
    T,
    NonNullContentsLinkDeletionResolver<T, CascadeUsagesResolver<T, L>>,
>;

/// Composes decorators around a store.
///
/// Every method takes the store by value and returns the *concrete* composed type, so a
/// stack is built at compile time and every layer is a candidate for inlining:
///
/// ```rust
/// use doublets::{decorators::{DecoratorsExt, Resolve}, mem, unit, Doublets};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut store = unit::Store::<usize, _>::new(mem::Global::new())?
///     .with_uniqueness(Resolve)
///     .with_usages_validation();
///
/// let a = store.create_point()?;
/// let b = store.create_point()?;
/// // The second `create_link` reuses the first one instead of storing a duplicate.
/// assert_eq!(store.create_link(a, b)?, store.create_link(a, b)?);
/// # Ok(())
/// # }
/// ```
pub trait DecoratorsExt<T: LinkReference>: Doublets<T> + Sized {
    /// Applies a [`UniquenessPolicy`] — [`Validate`](super::Validate),
    /// [`Resolve`](super::Resolve) or [`CascadeResolve`](super::CascadeResolve).
    #[inline]
    fn with_uniqueness<P: UniquenessPolicy>(self, _policy: P) -> P::Decorator<T, Self> {
        P::decorate(self)
    }

    /// Applies a [`UsagesPolicy`] — [`Validate`](super::Validate) or
    /// [`CascadeResolve`](super::CascadeResolve).
    #[inline]
    fn with_usages<P: UsagesPolicy>(self, _policy: P) -> P::Decorator<T, Self> {
        P::decorate(self)
    }

    /// Wraps in [`UsagesValidator`].
    #[inline]
    fn with_usages_validation(self) -> UsagesValidator<T, Self> {
        UsagesValidator::new(self)
    }

    /// Wraps in [`CascadeUsagesResolver`].
    #[inline]
    fn with_cascade_usages_resolution(self) -> CascadeUsagesResolver<T, Self> {
        CascadeUsagesResolver::new(self)
    }

    /// Wraps in [`InnerReferenceExistenceValidator`].
    #[inline]
    fn with_inner_reference_existence_validation(
        self,
    ) -> InnerReferenceExistenceValidator<T, Self> {
        InnerReferenceExistenceValidator::new(self)
    }

    /// Wraps in [`NonExistentDependenciesCreator`].
    #[inline]
    fn with_non_existent_dependencies_creation(self) -> NonExistentDependenciesCreator<T, Self> {
        NonExistentDependenciesCreator::new(self)
    }

    /// Wraps in [`ItselfConstantToSelfReferenceResolver`].
    #[inline]
    fn with_itself_constant_resolution(self) -> ItselfConstantToSelfReferenceResolver<T, Self> {
        ItselfConstantToSelfReferenceResolver::new(self)
    }

    /// Wraps in [`NullConstantToSelfReferenceResolver`].
    #[inline]
    fn with_null_constant_resolution(self) -> NullConstantToSelfReferenceResolver<T, Self> {
        NullConstantToSelfReferenceResolver::new(self)
    }

    /// Wraps in [`NonNullContentsLinkDeletionResolver`].
    #[inline]
    fn with_non_null_contents_deletion_resolution(
        self,
    ) -> NonNullContentsLinkDeletionResolver<T, Self> {
        NonNullContentsLinkDeletionResolver::new(self)
    }

    /// Wraps in [`LoggingDecorator`], logging every mutation to `writer`.
    #[inline]
    fn with_logging<W: Write + Send + Sync>(self, writer: W) -> LoggingDecorator<T, Self, W> {
        LoggingDecorator::new(self, writer)
    }

    /// Wraps in [`NoExceptionsDecorator`].
    #[inline]
    fn with_no_exceptions(self) -> NoExceptionsDecorator<T, Self> {
        NoExceptionsDecorator::new(self)
    }

    /// Builds the same stack as C#
    /// `ILinksExtensions.DecorateWithAutomaticUniquenessAndUsagesResolution`:
    /// [`CascadeUsagesResolver`] innermost, then [`NonNullContentsLinkDeletionResolver`],
    /// then [`CascadeUniquenessAndUsagesResolver`] outermost.
    #[inline]
    fn with_automatic_uniqueness_and_usages_resolution(
        self,
    ) -> AutomaticUniquenessAndUsagesResolution<T, Self> {
        CascadeUniquenessAndUsagesResolver::new(NonNullContentsLinkDeletionResolver::new(
            CascadeUsagesResolver::new(self),
        ))
    }
}

impl<T: LinkReference, L: Doublets<T>> DecoratorsExt<T> for L {}
