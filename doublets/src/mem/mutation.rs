//! Store mutation primitives without contract validation.

use data::{Flow, LinkReference};

use crate::{data::WriteHandler, mem::Release};

/// Minimal endpoint update of a live link, without contract validation.
///
/// Implemented by both memory stores. Their safe [`Links::update_links`] and
/// [`Links::delete_links`] methods are checked convenience routes: they
/// delegate to [`CheckedMutation`], which rejects a missing address with
/// [`Error::NotExists`] before calling these primitives. A caller that already
/// guarantees the preconditions below can call the primitives directly and
/// skip that redundant verification.
///
/// Deletion is composed from two primitives: an optional endpoint reset via
/// [`update_unchecked`](Self::update_unchecked) to `(0, 0)`, followed by
/// [`Release::release_unchecked`]. Resetting is a policy, not part of
/// release, so the caller decides whether it is needed.
///
/// See `docs/mutation.md` for the full contract and the checked, ignore-missing
/// and unchecked routes.
///
/// # Examples
///
/// ```
/// use doublets::{
///     mem::{Global, Release, UncheckedMutation},
///     unit, Doublets,
/// };
/// use data::Flow;
/// # fn main() -> Result<(), doublets::Error<u32>> {
/// let mut store = unit::Store::<u32, _>::new(Global::new())?;
/// let a = store.create_point()?;
/// let b = store.create()?;
///
/// // SAFETY: `b` was just created by this store and is still live.
/// unsafe { store.update_unchecked(b, a, a, &mut |_, _| Flow::Continue) };
/// assert_eq!(store.search(a, a), Some(b));
///
/// // SAFETY: `b` is live; resetting it detaches its outgoing index memberships,
/// // which is the remaining precondition of `release_unchecked`.
/// unsafe {
///     store.update_unchecked(b, 0, 0, &mut |_, _| Flow::Continue);
///     store.release_unchecked(b, &mut |_, _| Flow::Continue);
/// }
/// assert!(!store.exist(b));
/// # Ok(())
/// # }
/// ```
///
/// [`Links::update_links`]: crate::Links::update_links
/// [`Links::delete_links`]: crate::Links::delete_links
/// [`CheckedMutation`]: crate::decorators::CheckedMutation
/// [`Error::NotExists`]: crate::Error::NotExists
pub trait UncheckedMutation<T: LinkReference>: Release<T> {
    /// Sets the endpoints of a live link without checking that it exists.
    ///
    /// Detaches the link's current outgoing source/target index memberships,
    /// writes `(source, target)`, and attaches the new memberships. Calls
    /// `handler` once, after the update, with the previous and the new state,
    /// returning its flow. `Break` does not undo the update.
    ///
    /// No other contract is verified: the new endpoints may be zero, the link
    /// itself, other live links, or unallocated addresses (the split layout
    /// indexes the latter externally). Uniqueness, reference existence and
    /// usages are separate decorator policies.
    ///
    /// # Safety
    ///
    /// - `index` must be a live allocated address of this store: never null, a
    ///   service constant, an address beyond the allocated range, an unused
    ///   (free-list) slot or an already released address.
    /// - The index structures of the store must be consistent with its link
    ///   data, as maintained by the store API. Do not write raw link or index
    ///   memory directly.
    ///
    /// The caller must uphold these conditions even in release builds: an
    /// unused slot shares memory with the free list, so updating it corrupts
    /// allocation state. Safe code should use [`crate::Doublets::update`] or
    /// [`crate::Links::update_links`].
    unsafe fn update_unchecked(
        &mut self,
        index: T,
        source: T,
        target: T,
        handler: WriteHandler<'_, T>,
    ) -> Flow;
}
