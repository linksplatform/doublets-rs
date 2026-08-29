//! Decorators that react to links which are still referenced by other links.
//!
//! Ports of C# `LinksUsagesValidator` and `LinksCascadeUsagesResolver`.

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
    Doublets, Error, Fuse, Link, Links,
};

decorator_struct! {
    /// Rejects updates and deletions of a link that other links still reference.
    ///
    /// Port of C# `LinksUsagesValidator`. The operation fails with [`Error::HasUsages`],
    /// the analogue of C#'s `ArgumentLinkHasDependenciesException`.
    UsagesValidator
}

impl<T: LinkReference, L: Doublets<T>> Links<T> for UsagesValidator<T, L> {
    forward!(count_links, each_links, create_links);

    #[inline]
    fn update_links(
        &mut self,
        query: &[T],
        change: &[T],
        handler: WriteHandler<'_, T>,
    ) -> Result<Flow, Error<T>> {
        let index = helpers::part(query, self.links.constants().index_part);
        helpers::ensure_no_usages(&self.links, index)?;
        self.links.update_links(query, change, handler)
    }

    #[inline]
    fn delete_links(
        &mut self,
        query: &[T],
        handler: WriteHandler<'_, T>,
    ) -> Result<Flow, Error<T>> {
        let index = helpers::part(query, self.links.constants().index_part);
        helpers::ensure_no_usages(&self.links, index)?;
        self.links.delete_links(query, handler)
    }
}

decorator_struct! {
    /// Deletes everything that references a link before deleting the link itself.
    ///
    /// Port of C# `LinksCascadeUsagesResolver` together with
    /// `ILinksExtensions.DeleteAllUsages`.
    ///
    /// # Deviation from C\#
    ///
    /// C# recurses through the facade and keeps no record of what it has already visited,
    /// so a reference cycle (`a` → `b` → `a`) makes it recurse until the stack overflows.
    /// This port threads a visited set through the cascade, so cyclic graphs terminate.
    /// The price is that the nested deletions run from this layer inwards rather than from
    /// the top of the stack; place this decorator above the layers whose behaviour the
    /// cascade must observe.
    CascadeUsagesResolver
}

impl<T: LinkReference, L: Doublets<T>> CascadeUsagesResolver<T, L> {
    /// Port of C# `ILinksExtensions.DeleteAllUsages` followed by the inner deletion.
    fn cascade(
        &mut self,
        index: T,
        visited: &mut Vec<T>,
        handler: &mut dyn FnMut(Link<T>, Link<T>) -> Flow,
    ) -> Result<Flow, Error<T>> {
        if visited.contains(&index) {
            return Ok(Flow::Continue);
        }
        visited.push(index);

        for usage in helpers::usages_of(self, index) {
            if usage.index == index || !self.links.exist(usage.index) {
                continue;
            }
            self.cascade(usage.index, visited, handler)?;
        }
        self.links.delete_links(&[index], handler)
    }
}

impl<T: LinkReference, L: Doublets<T>> Links<T> for CascadeUsagesResolver<T, L> {
    forward!(count_links, each_links, create_links, update_links);

    #[inline]
    fn delete_links(
        &mut self,
        query: &[T],
        handler: WriteHandler<'_, T>,
    ) -> Result<Flow, Error<T>> {
        let index = helpers::part(query, self.links.constants().index_part);
        let mut fuse = Fuse::new(&mut *handler);
        let mut visited = Vec::new();
        self.cascade(index, &mut visited, &mut |before, after| {
            fuse.call(before, after)
        })
    }
}
