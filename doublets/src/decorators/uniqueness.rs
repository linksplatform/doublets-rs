//! Decorators that keep `(source, target)` pairs unique.
//!
//! Ports of C# `LinksUniquenessValidator`, `LinksUniquenessResolver` and
//! `LinksCascadeUniquenessAndUsagesResolver`.

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
    /// Rejects an update that would make two links share the same `(source, target)` pair.
    ///
    /// Port of C# `LinksUniquenessValidator`. The update fails with
    /// [`Error::AlreadyExists`], the analogue of C#'s
    /// `LinkWithSameValueAlreadyExistsException`.
    UniquenessValidator
}

impl<T: LinkReference, L: Doublets<T>> Links<T> for UniquenessValidator<T, L> {
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
        helpers::ensure_does_not_exist(&self.links, source, target)?;
        self.links.update_links(query, change, handler)
    }
}

decorator_struct! {
    /// Redirects an update that would produce a duplicate to the link that already exists,
    /// then deletes the now-redundant link.
    ///
    /// Port of C# `LinksUniquenessResolver`. Composed with the default
    /// [`Doublets::create_link`] this turns `create_link` into the equivalent of
    /// [`Doublets::get_or_create`]: the temporary link created first is deleted, and the
    /// surviving link is reported to the handler.
    ///
    /// # Deviation from C\#
    ///
    /// C# reports nothing for the surviving link, so `CreateAndUpdate` returns the address
    /// of the link it just deleted. This port reports the surviving link as
    /// `(before, after)` instead, so [`Doublets::update_by`] and [`Doublets::create_link`]
    /// return an address that is actually valid.
    UniquenessResolver
}

impl<T: LinkReference, L: Doublets<T>> UniquenessResolver<T, L> {
    /// Port of C# `LinksUniquenessResolver.ResolveAddressChangeConflict`.
    #[inline]
    fn resolve_address_change_conflict(
        &mut self,
        old: T,
        new: T,
        handler: WriteHandler<'_, T>,
    ) -> Result<Flow, Error<T>> {
        resolve_address_change_conflict(self, old, new, handler)
    }
}

impl<T: LinkReference, L: Doublets<T>> Links<T> for UniquenessResolver<T, L> {
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
        let old = helpers::part(query, constants.index_part);

        let Some(new) = self.links.search(source, target) else {
            return self.links.update_links(query, change, handler);
        };
        self.resolve_address_change_conflict(old, new, handler)
    }
}

decorator_struct! {
    /// Like [`UniquenessResolver`], but first re-points every link that referenced the
    /// redundant link at the surviving one.
    ///
    /// Port of C# `LinksCascadeUniquenessAndUsagesResolver`.
    CascadeUniquenessAndUsagesResolver
}

impl<T: LinkReference, L: Doublets<T>> CascadeUniquenessAndUsagesResolver<T, L> {
    /// Port of C# `LinksCascadeUniquenessAndUsagesResolver.ResolveAddressChangeConflict`.
    #[inline]
    fn resolve_address_change_conflict(
        &mut self,
        old: T,
        new: T,
        handler: WriteHandler<'_, T>,
    ) -> Result<Flow, Error<T>> {
        let mut fuse = Fuse::new(&mut *handler);
        // C# uses the facade here so the merge recurses through the whole stack; the
        // static equivalent is `self`, which is this layer plus everything it wraps.
        helpers::merge_usages(self, old, new, |before, after| fuse.call(before, after))?;
        resolve_address_change_conflict(self, old, new, &mut |before, after| {
            fuse.call(before, after)
        })
    }
}

impl<T: LinkReference, L: Doublets<T>> Links<T> for CascadeUniquenessAndUsagesResolver<T, L> {
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
        let old = helpers::part(query, constants.index_part);

        let Some(new) = self.links.search(source, target) else {
            return self.links.update_links(query, change, handler);
        };
        self.resolve_address_change_conflict(old, new, handler)
    }
}

/// Shared body of C# `LinksUniquenessResolver.ResolveAddressChangeConflict`.
///
/// `links` is the decorator itself, standing in for C#'s `_facade`, so the cascading
/// delete runs through every layer of the stack.
#[inline]
fn resolve_address_change_conflict<T, D>(
    links: &mut D,
    old: T,
    new: T,
    handler: WriteHandler<'_, T>,
) -> Result<Flow, Error<T>>
where
    T: LinkReference,
    D: Doublets<T>,
{
    let mut fuse = Fuse::new(&mut *handler);
    if old != new && links.exist(old) {
        links.delete_links(&[old], &mut |before, after| fuse.call(before, after))?;
    }
    let survivor = links.try_get_link(new)?;
    Ok(fuse.call(survivor.clone(), survivor))
}
