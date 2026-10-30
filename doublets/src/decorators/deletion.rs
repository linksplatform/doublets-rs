//! Checked release and optional endpoint-reset policy for deletion.
//!
//! Port of C# `NonNullContentsLinkDeletionResolver`.

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
    mem::Release,
    Doublets, Error, Fuse, Link, Links,
};

/// A safe release-only view of a memory store.
///
/// Deletion checks for a live address and zero endpoints, then invokes the
/// minimal release primitive. Nonzero endpoints return [`Error::NonNullContents`]
/// without mutation. Incoming usages are preserved, not rejected or cascaded.
/// Other operations are forwarded unchanged. Add
/// [`NonNullContentsLinkDeletionResolver`] to opt into automatic reset, or add
/// [`super::UsagesValidator`] / [`super::CascadeUsagesResolver`] for usage policy.
///
/// ```
/// use doublets::{decorators::{CheckedRelease, DecoratorsExt}, unit, Doublets, Error};
/// use mem::Global;
/// # fn main() -> Result<(), Error<u32>> {
/// let mut store = unit::Store::<u32, _>::new(Global::new())?;
/// let index = store.create_point()?;
/// let mut release = CheckedRelease::new(&mut store);
/// assert!(matches!(release.delete(index), Err(Error::NonNullContents(_))));
/// let mut resetting = release.with_non_null_contents_deletion_resolution();
/// assert_eq!(resetting.delete(index)?, index);
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct CheckedRelease<'a, T: LinkReference, L: Release<T>> {
    links: &'a mut L,
    _marker: PhantomData<fn(T)>,
}

impl<'a, T: LinkReference, L: Release<T>> CheckedRelease<'a, T, L> {
    /// Borrows a store as a checked release-only view.
    #[inline]
    pub const fn new(links: &'a mut L) -> Self {
        Self {
            links,
            _marker: PhantomData,
        }
    }

    /// Returns a shared reference to the underlying store.
    #[inline]
    #[must_use]
    pub const fn inner(&self) -> &L {
        self.links
    }

    /// Returns a mutable reference to the underlying store.
    #[inline]
    #[must_use]
    pub fn inner_mut(&mut self) -> &mut L {
        self.links
    }

    /// Releases the view's borrow and returns the underlying store.
    #[inline]
    #[must_use]
    pub fn into_inner(self) -> &'a mut L {
        self.links
    }
}

impl<T: LinkReference, L: Release<T>> Links<T> for CheckedRelease<'_, T, L> {
    forward!(count_links, each_links, create_links, update_links);

    #[inline]
    fn delete_links(
        &mut self,
        query: &[T],
        handler: WriteHandler<'_, T>,
    ) -> Result<Flow, Error<T>> {
        let index = helpers::part(query, self.links.constants().index_part);
        self.links.release_with(index, handler)
    }
}

impl<T: LinkReference, L: Release<T>> Doublets<T> for CheckedRelease<'_, T, L> {
    #[inline]
    fn get_link(&self, index: T) -> Option<Link<T>> {
        self.links.get_link(index)
    }
}

/// Compatibility deletion: automatic reset with the original single callback.
/// Endpoint policy lives in the resolver; the backend only releases the slot.
#[inline]
pub(crate) fn delete_with_reset<T: LinkReference, L: Release<T>>(
    links: &mut L,
    query: &[T],
    handler: WriteHandler<'_, T>,
) -> Result<Flow, Error<T>> {
    let index = helpers::part(query, links.constants().index_part);
    let original = links.try_get_link(index)?;
    NonNullContentsLinkDeletionResolver::new(CheckedRelease::new(links)).delete_links(
        query,
        &mut |_, after| {
            if after.is_null() {
                handler(original.clone(), after)
            } else {
                Flow::Continue
            }
        },
    )
}

decorator_struct! {
    /// Resets a link to `(null, null)` before deleting it. Already reset links
    /// skip the update. Compose over [`CheckedRelease`] to make resetting an
    /// explicit choice above the store's minimal release primitive.
    ///
    /// Port of C# `NonNullContentsLinkDeletionResolver`.
    NonNullContentsLinkDeletionResolver
}

impl<T: LinkReference, L: Doublets<T>> Links<T> for NonNullContentsLinkDeletionResolver<T, L> {
    forward!(count_links, each_links, create_links, update_links);

    #[inline]
    fn delete_links(
        &mut self,
        query: &[T],
        handler: WriteHandler<'_, T>,
    ) -> Result<Flow, Error<T>> {
        let index = helpers::part(query, self.links.constants().index_part);
        let mut fuse = Fuse::new(&mut *handler);
        helpers::enforce_reset_values(&mut self.links, index, &mut |before, after| {
            fuse.call(before, after)
        })?;
        self.links
            .delete_links(query, &mut |before, after| fuse.call(before, after))
    }
}
