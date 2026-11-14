//! Rust ports of the `Platform.Data.Doublets.ILinksExtensions` helpers that the C#
//! decorators are built on.
//!
//! Every function here mirrors one C# extension method, so each decorator can stay as
//! close to its original as the two type systems allow.

use std::cmp::min;

use data::{Flow, LinkReference, LinksConstants};

use crate::{data::WriteHandler, Doublet, Doublets, DoubletsExt, Error, Fuse, Link};

/// Reads one part (`index_part` / `source_part` / `target_part`) out of a raw query slice.
///
/// Mirrors C# `Link<TLinkAddress>.SetValues`, where a slice shorter than the requested
/// part yields the null constant instead of failing.
#[inline]
pub(crate) fn part<T: LinkReference>(query: &[T], part: T) -> T {
    query
        .get(part.as_())
        .copied()
        .unwrap_or_else(|| T::from_byte(0))
}

/// Port of C# `ILinksExtensions.EnsureDoesNotExists`.
///
/// Fails with [`Error::AlreadyExists`] when a link with the same `(source, target)` pair
/// is already stored — the analogue of C#'s `LinkWithSameValueAlreadyExistsException`.
#[inline]
pub(crate) fn ensure_does_not_exist<T, L>(links: &L, source: T, target: T) -> Result<(), Error<T>>
where
    T: LinkReference,
    L: Doublets<T>,
{
    if links.search(source, target).is_some() {
        Err(Error::AlreadyExists(Doublet::new(source, target)))
    } else {
        Ok(())
    }
}

/// Port of C# `ILinksExtensions.EnsureNoUsages`.
///
/// Fails with [`Error::HasUsages`] when other links still reference `link` — the
/// analogue of C#'s `ArgumentLinkHasDependenciesException`.
#[inline]
pub(crate) fn ensure_no_usages<T, L>(links: &L, link: T) -> Result<(), Error<T>>
where
    T: LinkReference,
    L: Doublets<T>,
{
    if links.has_usages(link) {
        Err(Error::HasUsages(usages_of(links, link)))
    } else {
        Ok(())
    }
}

/// Collects every link that references `link` as a source or a target, excluding `link`
/// itself.
#[inline]
pub(crate) fn usages_of<T, L>(links: &L, link: T) -> Vec<Link<T>>
where
    T: LinkReference,
    L: Doublets<T>,
{
    let any = links.constants().any;
    let mut usages: Vec<_> = links.each_iter([any, link, any]).collect();
    usages.extend(
        links
            .each_iter([any, any, link])
            .filter(|usage| usage.source != link),
    );
    usages.retain(|usage| usage.index != link);
    usages
}

/// Port of C# `ILinksExtensions.EnsureInnerReferenceExists`.
///
/// Every element of `query` that is an internal reference must resolve to a stored link;
/// `any`, `itself` and the other service constants live outside the internal range and
/// are therefore skipped, exactly as in C#.
#[inline]
pub(crate) fn ensure_inner_reference_exists<T, L>(links: &L, query: &[T]) -> Result<(), Error<T>>
where
    T: LinkReference,
    L: Doublets<T>,
{
    for &reference in query {
        if links.constants().is_internal(reference) && !links.exist(reference) {
            return Err(Error::NotExists(reference));
        }
    }
    Ok(())
}

/// Port of C# `ILinksExtensions.EnsureLinkExists`.
#[inline]
pub(crate) fn ensure_link_exists<T, L>(links: &L, link: T) -> Result<(), Error<T>>
where
    T: LinkReference,
    L: Doublets<T>,
{
    if links.exist(link) {
        Ok(())
    } else {
        Err(Error::NotExists(link))
    }
}

/// Port of C# `ILinksExtensions.ResolveConstantAsSelfReference`.
///
/// Replaces every occurrence of `constant` in the substitution's source and target with
/// the substitution's own index, falling back to the restriction's index when the
/// substitution carries no index of its own.
#[inline]
pub(crate) fn resolve_constant_as_self_reference<T: LinkReference>(
    constant: T,
    restriction: &[T],
    substitution: &[T],
    constants: &LinksConstants<T>,
) -> [T; 3] {
    let null = T::from_byte(0);
    let mut index = part(substitution, constants.index_part);
    if index == null {
        index = part(restriction, constants.index_part);
    }

    let mut source = part(substitution, constants.source_part);
    let mut target = part(substitution, constants.target_part);
    if source == constant {
        source = index;
    }
    if target == constant {
        target = index;
    }
    [index, source, target]
}

/// Port of C# `ILinksExtensions.CreatePoint`.
///
/// Creates a link and immediately points it at itself, reporting both the creation and
/// the update through `handler`.
#[inline]
pub(crate) fn create_point_with<T, L, F>(links: &mut L, handler: F) -> Result<Flow, Error<T>>
where
    T: LinkReference,
    L: Doublets<T>,
    F: FnMut(Link<T>, Link<T>) -> Flow,
{
    let mut point = T::from_byte(0);
    let mut handler = Fuse::new(handler);
    links.create_links(&[], &mut |before, after| {
        point = after.index;
        handler.call(before, after)
    })?;
    links.update_links(&[point], &[point, point, point], &mut |before, after| {
        handler.call(before, after)
    })
}

/// Port of C# `ILinksExtensions.AreValuesReset`.
#[inline]
pub(crate) fn are_values_reset<T, L>(links: &L, index: T) -> Result<bool, Error<T>>
where
    T: LinkReference,
    L: Doublets<T>,
{
    let null = T::from_byte(0);
    let link = links.try_get_link(index)?;
    Ok(link.source == null && link.target == null)
}

/// Port of C# `ILinksExtensions.EnforceResetValues`.
///
/// Resets `index` to `(null, null)` unless it is reset already.
#[inline]
pub(crate) fn enforce_reset_values<T, L>(
    links: &mut L,
    index: T,
    handler: WriteHandler<'_, T>,
) -> Result<Flow, Error<T>>
where
    T: LinkReference,
    L: Doublets<T>,
{
    if are_values_reset(links, index)? {
        return Ok(Flow::Continue);
    }
    let null = T::from_byte(0);
    links.update_links(&[index], &[index, null, null], handler)
}

/// Port of C# `ILinksExtensions.MergeUsages`.
///
/// Re-points every link that references `old` so that it references `new` instead.
///
/// # Deviation from C\#
///
/// The C# implementation builds its substitutions with the two-argument
/// `Link<TLinkAddress>` constructor, which fills `(index, source)` and leaves the target
/// null. As a result it writes null targets for usages-as-source, and re-points the
/// *source* of usages-as-target. This port implements the intended behaviour instead,
/// which is the behaviour of [`Doublets::rebase_with`].
#[inline]
pub(crate) fn merge_usages<T, L, F>(
    links: &mut L,
    old: T,
    new: T,
    handler: F,
) -> Result<Flow, Error<T>>
where
    T: LinkReference,
    L: Doublets<T>,
    F: FnMut(Link<T>, Link<T>) -> Flow,
{
    if old == new {
        return Ok(Flow::Continue);
    }

    let mut handler = Fuse::new(handler);
    for usage in usages_of(links, old) {
        let source = if usage.source == old {
            new
        } else {
            usage.source
        };
        let target = if usage.target == old {
            new
        } else {
            usage.target
        };
        links.update_links(
            &[usage.index],
            &[usage.index, source, target],
            &mut |before, after| handler.call(before, after),
        )?;
    }
    Ok(Flow::Continue)
}

/// Port of C# `ILinksExtensions.EnsureCreated`.
///
/// Creates links until every address in `addresses` exists, then deletes the addresses
/// that were only created to reach the requested ones.
///
/// # Deviation from C\#
///
/// The C# loop is `while (createdLink != max) { createdLinks.Add(createdLink); }` — it
/// never calls the creator again and therefore never terminates. This port runs the loop
/// the C# code describes, and stops as soon as the store hands out an address at or
/// beyond `max`.
#[inline]
pub(crate) fn ensure_created<T, L>(links: &mut L, addresses: &[T]) -> Result<(), Error<T>>
where
    T: LinkReference,
    L: Doublets<T>,
{
    let null = T::from_byte(0);
    let non_existent: Vec<_> = addresses
        .iter()
        .copied()
        .filter(|&address| address != null && !links.exist(address))
        .collect();

    let Some(&requested) = non_existent.iter().max() else {
        return Ok(());
    };
    let max = min(requested, *links.constants().internal_range.end());

    let mut created = Vec::new();
    loop {
        let link = links.create_by([])?;
        created.push(link);
        if link >= max {
            break;
        }
    }

    for link in created {
        if !non_existent.contains(&link) {
            links.delete_by([link])?;
        }
    }
    Ok(())
}

/// Formats a link the way C# `Link<TLinkAddress>.ToString` does.
#[inline]
pub(crate) fn format_link<T: LinkReference>(link: &Link<T>) -> String {
    if link.index == T::from_byte(0) {
        format!("({}->{})", link.source, link.target)
    } else {
        format!("({}: {}->{})", link.index, link.source, link.target)
    }
}
