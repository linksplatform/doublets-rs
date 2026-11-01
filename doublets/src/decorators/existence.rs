//! Decorators that check — or establish — the existence of the links an operation refers
//! to.
//!
//! Ports of C# `LinksInnerReferenceExistenceValidator` and
//! `LinksNonExistentDependenciesCreator`.

use std::{
    fmt::{self, Debug, Formatter},
    marker::PhantomData,
};

use data::{Flow, LinkReference, LinksConstants};

use super::{
    helpers,
    macros::{decorator_struct, forward},
};
use crate::{
    data::{ReadHandler, WriteHandler},
    Doublets, Error, Link, Links,
};

decorator_struct! {
    /// Rejects an operation that names an internal reference which is not stored.
    ///
    /// Port of C# `LinksInnerReferenceExistenceValidator`. Service constants such as
    /// `any` and `itself` lie outside the internal range and are never checked.
    InnerReferenceExistenceValidator
}

impl<T: LinkReference, L: Doublets<T>> InnerReferenceExistenceValidator<T, L> {
    /// The fallible form of [`Links::each_links`].
    ///
    /// [`Links::each_links`] has no error channel, so it reports a dangling reference by
    /// returning [`Flow::Continue`] without yielding anything. Call this method instead
    /// when the dangling reference itself is what you want to know about.
    #[inline]
    pub fn try_each_links(
        &self,
        query: &[T],
        handler: ReadHandler<'_, T>,
    ) -> Result<Flow, Error<T>> {
        helpers::ensure_inner_reference_exists(&self.links, query)?;
        Ok(self.links.each_links(query, handler))
    }
}

impl<T: LinkReference, L: Doublets<T>> Links<T> for InnerReferenceExistenceValidator<T, L> {
    forward!(count_links, create_links);

    #[inline]
    fn each_links(&self, query: &[T], handler: ReadHandler<'_, T>) -> Flow {
        self.try_each_links(query, handler)
            .unwrap_or(Flow::Continue)
    }

    #[inline]
    fn update_links(
        &mut self,
        query: &[T],
        change: &[T],
        handler: WriteHandler<'_, T>,
    ) -> Result<Flow, Error<T>> {
        helpers::ensure_inner_reference_exists(&self.links, query)?;
        helpers::ensure_inner_reference_exists(&self.links, change)?;
        self.links.update_links(query, change, handler)
    }

    #[inline]
    fn delete_links(
        &mut self,
        query: &[T],
        handler: WriteHandler<'_, T>,
    ) -> Result<Flow, Error<T>> {
        let index = helpers::part(query, self.links.constants().index_part);
        helpers::ensure_link_exists(&self.links, index)?;
        self.links.delete_links(query, handler)
    }
}

decorator_struct! {
    /// Creates the links an update depends on before performing the update.
    ///
    /// Port of C# `LinksNonExistentDependenciesCreator`.
    NonExistentDependenciesCreator
}

impl<T: LinkReference, L: Doublets<T>> Links<T> for NonExistentDependenciesCreator<T, L> {
    forward!(count_links, each_links, create_links, delete_links);

    #[inline]
    fn update_links(
        &mut self,
        query: &[T],
        change: &[T],
        handler: WriteHandler<'_, T>,
    ) -> Result<Flow, Error<T>> {
        let constants = self.links.constants();
        let source = helpers::part(change, constants.source_part);
        let target = helpers::part(change, constants.target_part);
        helpers::ensure_created(&mut self.links, &[source, target])?;
        self.links.update_links(query, change, handler)
    }
}

decorator_struct! {
    /// Ignores deletion of a missing link without calling the inner store or the handler.
    ///
    /// Port of C# `LinksNonExistentDeletionResolver`. A deletion whose requested
    /// index is not a live link returns `Ok(Flow::Continue)` with no writes and no
    /// callbacks. A deletion of a live link is forwarded unchanged, so every error
    /// the inner layers return — usages, contents, I/O, capacity — still
    /// propagates. Unlike [`NoExceptionsDecorator`](super::NoExceptionsDecorator),
    /// it decides before the operation and never swallows an error.
    ///
    /// This is the *ignore* policy for a missing deletion; the *error* policy is
    /// [`InnerReferenceExistenceValidator`] (or the stores' checked
    /// [`CheckedMutation`](super::CheckedMutation) route). It only guards the
    /// requested address: endpoint reset, usages and other contracts remain
    /// separate policies. Place it outside reset and cascade resolvers so it
    /// runs before they read the link. Other operations are forwarded unchanged.
    ///
    /// ```
    /// use doublets::{decorators::DecoratorsExt, unit, Doublets};
    /// use mem::Global;
    /// # fn main() -> Result<(), doublets::Error<u32>> {
    /// let mut store = unit::Store::<u32, _>::new(Global::new())?
    ///     .with_automatic_uniqueness_and_usages_resolution()
    ///     .with_non_existent_deletion_resolution();
    /// let index = store.create_point()?;
    /// assert_eq!(store.delete(index)?, index);
    /// // Deleting it again is a no-op instead of `Error::NotExists`.
    /// assert_eq!(store.delete(index)?, 0);
    /// # Ok(())
    /// # }
    /// ```
    NonExistentDeletionResolver
}

impl<T: LinkReference, L: Doublets<T>> Links<T> for NonExistentDeletionResolver<T, L> {
    forward!(count_links, each_links, create_links, update_links);

    #[inline]
    fn delete_links(
        &mut self,
        query: &[T],
        handler: WriteHandler<'_, T>,
    ) -> Result<Flow, Error<T>> {
        let index = helpers::part(query, self.links.constants().index_part);
        if self.links.get_link(index).is_some() {
            self.links.delete_links(query, handler)
        } else {
            Ok(Flow::Continue)
        }
    }
}
