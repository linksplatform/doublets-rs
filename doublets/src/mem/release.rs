//! Slot release without automatic endpoint reset.

use data::{Flow, LinkReference};

use crate::{data::WriteHandler, Doublets, Error};

/// Releases already reset links, independently of the endpoint reset policy.
///
/// Implemented by both memory stores. Unlike their compatibility `delete*`
/// helpers, these operations never update endpoints. To compose safe release
/// with decorators, use [`CheckedRelease`](crate::decorators::CheckedRelease).
///
/// Incoming references are allowed and retain their values. The split layout
/// moves their index entries to external trees; the unit layout uses global
/// endpoint trees. Reuse of the address makes these references internal again.
/// Rejecting or deleting usages is a separate decorator policy.
///
/// # Examples
///
/// ```
/// use doublets::{mem::Release, unit, Doublets};
/// use mem::Global;
/// # fn main() -> Result<(), doublets::Error<u32>> {
/// let mut store = unit::Store::<u32, _>::new(Global::new())?;
/// let index = store.create_point()?;
/// store.update(index, 0, 0)?;
/// assert_eq!(store.release(index)?, index);
/// assert!(!store.exist(index));
/// # Ok(())
/// # }
/// ```
pub trait Release<T: LinkReference>: Doublets<T> {
    /// Releases a slot without checking the contract or resetting its contents.
    ///
    /// Performs only structural bookkeeping (including incoming-usage index
    /// reclassification where required) and free-list/tail maintenance. Calls
    /// `handler` once with `(index, 0, 0)` and [`crate::Link::nothing()`], after
    /// release, returning its flow. `Break` does not undo the release.
    ///
    /// # Safety
    ///
    /// - `index` must be a live allocated address, never null, a service constant,
    ///   an unused slot or an address that has already been released.
    /// - Both endpoints must be zero and their outgoing index memberships must
    ///   already be detached. Creating an empty link or updating a live link to
    ///   `(0, 0)` through the store API establishes this condition. Merely
    ///   overwriting raw endpoint memory does not.
    /// - All incoming source/target index entries must be consistent with the
    ///   live address. They need not be absent: release preserves them, including
    ///   links using the address in both endpoints. Do not manipulate raw index
    ///   memory to bypass that invariant.
    ///
    /// The caller must uphold all conditions even in release builds. Safe code
    /// should use [`Release::release_with`] or [`Release::release`].
    unsafe fn release_unchecked(&mut self, index: T, handler: WriteHandler<'_, T>) -> Flow;

    /// Checks that `index` is live and reset, then releases it without an update.
    ///
    /// Returns [`Error::NotExists`] for a non-live address, or
    /// [`Error::NonNullContents`] when either endpoint is nonzero. Errors leave
    /// the store unchanged and do not call the handler. Incoming usages remain.
    #[inline]
    fn release_with(&mut self, index: T, handler: WriteHandler<'_, T>) -> Result<Flow, Error<T>> {
        let link = self.try_get_link(index)?;
        if link.source != T::from_byte(0) || link.target != T::from_byte(0) {
            return Err(Error::NonNullContents(link));
        }
        // SAFETY: The address is live and reset. Safe store operations maintain
        // outgoing memberships and incoming index consistency.
        Ok(unsafe { self.release_unchecked(index, handler) })
    }

    /// Checks and releases an already reset live link, returning its former index.
    #[inline]
    fn release(&mut self, index: T) -> Result<T, Error<T>> {
        self.release_with(index, &mut |_, _| Flow::Continue)
            .map(|_| index)
    }
}
