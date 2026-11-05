//! Reversible writes and journal records.

use data::LinkReference;

use crate::{Doublets, Error, Link};

use super::restore;

/// The mutation represented by a [`Transition`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransitionKind {
    /// A previously absent link was created.
    Create,
    /// The contents of an existing link changed.
    Update,
    /// An existing link was deleted.
    Delete,
}

/// A reversible mutation, including both link states.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transition<T: LinkReference> {
    /// Journal-local transaction identifier.
    pub transaction_id: u64,
    /// Journal-local, monotonically increasing write sequence.
    pub sequence: u64,
    /// The kind of mutation.
    pub kind: TransitionKind,
    /// Link state before mutation; the null link for creation.
    pub before: Link<T>,
    /// Link state after mutation; the null link for deletion.
    pub after: Link<T>,
}

impl<T: LinkReference> Transition<T> {
    /// Constructs a transition, inferring its kind from the null side.
    pub fn new(transaction_id: u64, sequence: u64, before: Link<T>, after: Link<T>) -> Self {
        let kind = if before.is_null() {
            TransitionKind::Create
        } else if after.is_null() {
            TransitionKind::Delete
        } else {
            TransitionKind::Update
        };
        Self {
            transaction_id,
            sequence,
            kind,
            before,
            after,
        }
    }

    /// Applies this write without generating another journal record.
    ///
    /// Repeated application is harmless. The store must support raw writes and
    /// exact-address restoration through sequential allocation (as both supplied
    /// stores do). This reconstructs the graph with O(number of links + highest
    /// address) store operations, plus sorting and backend index costs.
    pub fn apply<L: Doublets<T>>(&self, links: &mut L) -> Result<(), Error<T>> {
        self.validate(links)?;
        Self::set_state(links, &self.before, &self.after)
    }

    /// Reverts this write without generating another journal record.
    ///
    /// Like [`Self::apply`], this is idempotent and preserves link addresses.
    pub fn revert<L: Doublets<T>>(&self, links: &mut L) -> Result<(), Error<T>> {
        self.validate(links)?;
        Self::set_state(links, &self.after, &self.before)
    }

    pub(super) fn validate<L: Doublets<T>>(&self, links: &L) -> Result<(), Error<T>> {
        let inferred = Self::new(
            self.transaction_id,
            self.sequence,
            self.before.clone(),
            self.after.clone(),
        )
        .kind;
        if self.kind != inferred
            || (self.before.is_null() && self.after.is_null())
            || (!self.before.is_null()
                && !self.after.is_null()
                && self.before.index != self.after.index)
            || [&self.before, &self.after]
                .iter()
                .any(|link| !link.is_null() && !links.constants().is_internal(link.index))
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "invalid transition states",
            )
            .into());
        }
        Ok(())
    }

    fn set_state<L: Doublets<T>>(
        links: &mut L,
        from: &Link<T>,
        to: &Link<T>,
    ) -> Result<(), Error<T>> {
        let index = if to.is_null() { from.index } else { to.index };
        let mut snapshot = super::snapshot(links);
        snapshot.retain(|link| link.index != index);
        if !to.is_null() {
            snapshot.push(to.clone());
        }
        restore(links, &snapshot)
    }
}

/// A durable recovery record.
///
/// Start and commit snapshots bridge the gap between a store mutation and its
/// after-write callback, and remove the need for a store-specific flush hook.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JournalEntry<T: LinkReference> {
    /// Persisted before the transaction can modify the store.
    Begin {
        /// Journal-local transaction identifier.
        transaction_id: u64,
        /// Complete graph before any writes.
        snapshot: Vec<Link<T>>,
    },
    /// One callback's before/after pair, in write order.
    Transition(Transition<T>),
    /// Persisted before commit returns successfully.
    Commit {
        /// The transaction being committed.
        transaction_id: u64,
        /// Complete committed graph, used for redo.
        snapshot: Vec<Link<T>>,
    },
    /// Persisted after restoring the begin snapshot.
    Rollback {
        /// The transaction being rolled back.
        transaction_id: u64,
    },
}
