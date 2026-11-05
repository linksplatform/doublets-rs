//! Checked mutation adapter over the unchecked store primitives.

use std::marker::PhantomData;

use data::{Flow, LinkReference, LinksConstants};

use super::{helpers, macros::forward};
use crate::{
    data::{ReadHandler, WriteHandler},
    mem::UncheckedMutation,
    Doublets, Error, Link, Links,
};

/// A safe view that validates the requested address, then calls the
/// [`UncheckedMutation`] and [`Release`](crate::mem::Release) primitives.
///
/// This is the checked convenience behavior of the memory stores' own
/// [`Links::update_links`] and [`Links::delete_links`]: both delegate here. It
/// verifies exactly one contract — the requested index is a live link — and
/// reports a violation as [`Error::NotExists`] without calling the handler or
/// mutating the store. Deletion then resets nonzero endpoints and releases the
/// slot, reporting one `(original, Link::nothing())` event.
///
/// Endpoint references, uniqueness and usages are not checked here; select
/// [`InnerReferenceExistenceValidator`](super::InnerReferenceExistenceValidator),
/// a uniqueness policy or a usages policy for those. To ignore deletion of a
/// missing address instead of returning an error, place
/// [`NonExistentDeletionResolver`](super::NonExistentDeletionResolver) above.
/// A caller that already guarantees a live address can omit this check by
/// calling [`UncheckedMutation::update_unchecked`] and
/// [`Release::release_unchecked`](crate::mem::Release::release_unchecked)
/// in an `unsafe` block.
///
/// ```
/// use doublets::{decorators::CheckedMutation, unit, Doublets, Error};
/// use mem::Global;
/// # fn main() -> Result<(), Error<u32>> {
/// let mut store = unit::Store::<u32, _>::new(Global::new())?;
/// let index = store.create_point()?;
/// let mut checked = CheckedMutation::new(&mut store);
/// assert_eq!(checked.update(index, 0, index)?, index);
/// assert_eq!(checked.delete(index)?, index);
/// assert!(matches!(checked.delete(index), Err(Error::NotExists(i)) if i == index));
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub struct CheckedMutation<'a, T: LinkReference, L: UncheckedMutation<T>> {
    links: &'a mut L,
    _marker: PhantomData<fn(T)>,
}

impl<'a, T: LinkReference, L: UncheckedMutation<T>> CheckedMutation<'a, T, L> {
    /// Borrows a store as a checked mutation view.
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

impl<T: LinkReference, L: UncheckedMutation<T>> Links<T> for CheckedMutation<'_, T, L> {
    forward!(count_links, each_links, create_links);

    #[inline]
    fn update_links(
        &mut self,
        query: &[T],
        change: &[T],
        handler: WriteHandler<'_, T>,
    ) -> Result<Flow, Error<T>> {
        let constants = self.links.constants();
        let index = helpers::part(query, constants.index_part);
        let source = helpers::part(change, constants.source_part);
        let target = helpers::part(change, constants.target_part);
        self.links.try_get_link(index)?;
        // SAFETY: `index` is live, and safe store operations keep the indexes
        // consistent with the link data.
        Ok(unsafe { self.links.update_unchecked(index, source, target, handler) })
    }

    #[inline]
    fn delete_links(
        &mut self,
        query: &[T],
        handler: WriteHandler<'_, T>,
    ) -> Result<Flow, Error<T>> {
        let index = helpers::part(query, self.links.constants().index_part);
        let original = self.links.try_get_link(index)?;
        let null = T::from_byte(0);
        // SAFETY: `index` is live. Resetting it detaches its outgoing index
        // memberships, so the release preconditions hold afterwards; incoming
        // index entries are kept consistent by the safe store operations.
        unsafe {
            if original.source != null || original.target != null {
                self.links
                    .update_unchecked(index, null, null, &mut |_, _| Flow::Continue);
            }
            Ok(self
                .links
                .release_unchecked(index, &mut |_, after| handler(original.clone(), after)))
        }
    }
}

impl<T: LinkReference, L: UncheckedMutation<T>> Doublets<T> for CheckedMutation<'_, T, L> {
    #[inline]
    fn get_link(&self, index: T) -> Option<Link<T>> {
        self.links.get_link(index)
    }
}
