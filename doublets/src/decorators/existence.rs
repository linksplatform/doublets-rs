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
