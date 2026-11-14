//! Transactions with reversible writes and optional durable recovery.
//!
//! The store's callbacks run *after* mutation. A journal of callbacks alone
//! therefore cannot recover a crash between mutation and callback. This layer
//! persists a complete before snapshot at begin and an after snapshot at commit,
//! as well as every callback transition. Recovery restores the last committed
//! snapshot, or the before snapshot of an interrupted transaction.
//!
//! Snapshots use O(number of links) space per begin/commit and grow the journal;
//! restore uses O(number of links + highest address) store operations, plus sorting
//! and backend index costs. There is no isolation from readers
//! outside the exclusively owned store, no nesting, and no asynchronous commit.
//! Recovery assumes the underlying store remains structurally readable; repairing
//! torn tree-index writes requires support from the storage backend itself.
//!
//! Put validation/cascade policies around the transaction *handle*, with a raw
//! store underneath this layer. This records all policy writes and lets recovery
//! restore arbitrary addresses without validation rejecting intermediate states.

mod log;
mod types;

pub use log::{FileTransitionLog, MemoryTransitionLog, TransitionLog};
pub use types::{JournalEntry, Transition, TransitionKind};

use std::{
    collections::{BTreeMap, BTreeSet},
    io,
    ops::{Deref, DerefMut},
};

use data::{Flow, LinkReference, LinksConstants};

use super::macros::forward;
use crate::{Doublets, Error, Fuse, Link, Links, ReadHandler, WriteHandler};

struct Pending<T: LinkReference> {
    id: u64,
    before: Vec<Link<T>>,
    start: usize,
    failed: bool,
}

/// Adds transactions to any raw [`Doublets`] store.
///
/// Writes outside an explicit transaction are committed individually. Use a
/// handle to group compound helpers such as `create_point` or `create_link`.
/// A successful file-backed commit is recoverable even if the store has not
/// flushed, because the journal contains its complete committed state.
///
/// ```
/// use doublets::{decorators::{DecoratorsExt, MemoryTransitionLog}, mem, unit, Doublets};
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut store = unit::Store::<usize, _>::new(mem::Global::new())?
///     .with_transactions(MemoryTransitionLog::default())?;
/// let mut tx = store.begin_transaction()?;
/// let point = tx.create_point()?;
/// tx.commit()?;
/// assert!(store.exist(point));
/// # Ok(())
/// # }
/// ```
pub struct TransactionsDecorator<T: LinkReference, L: Doublets<T>, J: TransitionLog<T>> {
    links: L,
    journal: J,
    transitions: Vec<Transition<T>>,
    pending: Option<Pending<T>>,
    next_id: u64,
    next_sequence: u64,
    poisoned: bool,
    tracing: bool,
    rollback_error: Option<String>,
}

impl<T: LinkReference, L: Doublets<T>, J: TransitionLog<T>> TransactionsDecorator<T, L, J> {
    /// Wraps the store and recovers its journal before returning.
    ///
    /// An empty journal adopts the current graph. A nonempty journal belongs to
    /// its original store and overrides the graph with its last recovery state.
    pub fn new(links: L, journal: J) -> Result<Self, Error<T>> {
        let mut store = Self {
            links,
            journal,
            transitions: Vec::new(),
            pending: None,
            next_id: 1,
            next_sequence: 1,
            poisoned: false,
            tracing: false,
            rollback_error: None,
        };
        store.recover()?;
        Ok(store)
    }

    /// Starts a transaction, persisting its before snapshot first.
    ///
    /// Nested transactions are rejected. The handle exclusively borrows this
    /// decorator; dropping it without completion rolls back, including on unwind.
    pub fn begin_transaction(&mut self) -> Result<Transaction<'_, T, L, J>, Error<T>> {
        self.begin()?;
        Ok(Transaction { store: self })
    }

    /// Returns the wrapped store for read-only inspection.
    #[must_use]
    pub const fn inner(&self) -> &L {
        &self.links
    }

    /// Returns the journal for read-only inspection.
    #[must_use]
    pub const fn journal(&self) -> &J {
        &self.journal
    }

    /// All recorded transitions, including writes subsequently rolled back.
    #[must_use]
    pub fn transitions(&self) -> &[Transition<T>] {
        &self.transitions
    }

    /// Whether an I/O or restoration failure has blocked further writes.
    #[must_use]
    pub const fn is_poisoned(&self) -> bool {
        self.poisoned
    }

    /// Enables diagnostic transaction and transition messages on stderr.
    /// Disabled by default.
    pub fn set_tracing(&mut self, enabled: bool) {
        self.tracing = enabled;
    }

    /// The last rollback failure, including one encountered by a handle's drop.
    ///
    /// Drop cannot return an error. A failed rollback poisons the decorator and
    /// leaves the begin record available for recovery when the store is reopened.
    #[must_use]
    pub fn rollback_error(&self) -> Option<&str> {
        self.rollback_error.as_deref()
    }

    /// Unwraps the decorator after rolling back any forgotten transaction.
    pub fn into_parts(mut self) -> Result<(L, J), Error<T>> {
        self.rollback()?;
        Ok((self.links, self.journal))
    }

    fn begin(&mut self) -> Result<(), Error<T>> {
        if self.poisoned {
            return Err(failure(
                "transaction journal is poisoned; reopen for recovery",
            ));
        }
        if self.pending.is_some() {
            return Err(failure("nested transactions are not supported"));
        }
        let id = self.next_id;
        self.next_id = id
            .checked_add(1)
            .ok_or_else(|| failure("transaction id exhausted"))?;
        let before = snapshot(&self.links);
        self.append(&JournalEntry::Begin {
            transaction_id: id,
            snapshot: before.clone(),
        })?;
        self.pending = Some(Pending {
            id,
            before,
            start: self.transitions.len(),
            failed: false,
        });
        if self.tracing {
            eprintln!("[doublets transactions] begin {id}");
        }
        Ok(())
    }

    fn append(&mut self, entry: &JournalEntry<T>) -> Result<(), Error<T>> {
        if let Err(error) = self.journal.append(entry) {
            self.poisoned = true;
            return Err(error.into());
        }
        Ok(())
    }

    fn commit(&mut self) -> Result<(), Error<T>> {
        let Some(pending) = &self.pending else {
            return Ok(());
        };
        if pending.failed || self.poisoned {
            return Err(failure("failed transaction cannot be committed"));
        }
        let id = pending.id;
        let entry = JournalEntry::Commit {
            transaction_id: id,
            snapshot: snapshot(&self.links),
        };
        self.append(&entry)?;
        if self.tracing {
            eprintln!("[doublets transactions] commit {id}");
        }
        self.pending = None;
        Ok(())
    }

    fn rollback(&mut self) -> Result<(), Error<T>> {
        let Some(pending) = &self.pending else {
            return Ok(());
        };
        let id = pending.id;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            restore(&mut self.links, &pending.before)
        }))
        .unwrap_or_else(|_| {
            Err(failure(
                "wrapped store panicked while restoring transaction",
            ))
        });
        if let Err(error) = result {
            self.poisoned = true;
            self.rollback_error = Some(error.to_string());
            return Err(error);
        }
        self.pending = None;
        if self.poisoned {
            return Ok(());
        }
        if self.tracing {
            eprintln!("[doublets transactions] rollback {id}");
        }
        if let Err(error) = self.append(&JournalEntry::Rollback { transaction_id: id }) {
            self.rollback_error = Some(error.to_string());
            return Err(error);
        }
        Ok(())
    }

    fn recover(&mut self) -> Result<(), Error<T>> {
        let entries = self.journal.read_entries()?;
        let mut pending = None;
        let mut desired = None;
        for entry in entries {
            match entry {
                JournalEntry::Begin {
                    transaction_id,
                    snapshot,
                } => {
                    if pending.is_some() || transaction_id < self.next_id {
                        return Err(failure("invalid transaction begin order"));
                    }
                    validate(&self.links, &snapshot)?;
                    self.next_id = transaction_id
                        .checked_add(1)
                        .ok_or_else(|| failure("transaction id exhausted"))?;
                    pending = Some(transaction_id);
                    desired = Some(snapshot);
                }
                JournalEntry::Transition(transition) => {
                    transition.validate(&self.links)?;
                    if pending != Some(transition.transaction_id)
                        || transition.sequence < self.next_sequence
                    {
                        return Err(failure("invalid transition order"));
                    }
                    self.next_sequence = transition
                        .sequence
                        .checked_add(1)
                        .ok_or_else(|| failure("transition sequence exhausted"))?;
                    self.transitions.push(transition);
                }
                JournalEntry::Commit {
                    transaction_id,
                    snapshot,
                } => {
                    if pending != Some(transaction_id) {
                        return Err(failure("commit without matching begin"));
                    }
                    validate(&self.links, &snapshot)?;
                    desired = Some(snapshot);
                    pending = None;
                }
                JournalEntry::Rollback { transaction_id } => {
                    if pending != Some(transaction_id) {
                        return Err(failure("rollback without matching begin"));
                    }
                    pending = None;
                }
            }
        }
        if let Some(desired) = desired {
            restore(&mut self.links, &desired)?;
        }
        if let Some(transaction_id) = pending {
            self.append(&JournalEntry::Rollback { transaction_id })?;
        }
        Ok(())
    }

    fn write<F>(&mut self, handler: WriteHandler<'_, T>, operation: F) -> Result<Flow, Error<T>>
    where
        F: FnOnce(&mut L, WriteHandler<'_, T>) -> Result<Flow, Error<T>>,
    {
        if self.poisoned {
            return Err(failure(
                "transaction journal is poisoned; reopen for recovery",
            ));
        }
        let automatic = self.pending.is_none();
        if automatic {
            self.begin()?;
        }
        let pending = self
            .pending
            .as_mut()
            .ok_or_else(|| failure("no active transaction"))?;
        if pending.failed {
            return Err(failure("transaction has a failed write; roll it back"));
        }
        let id = pending.id;
        let mut handler = Fuse::new(handler);
        let mut log_error = None;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            operation(&mut self.links, &mut |before, after| {
                if log_error.is_some() {
                    return Flow::Break;
                }
                if before != after {
                    let Some(next) = self.next_sequence.checked_add(1) else {
                        log_error = Some(io::Error::other("transition sequence exhausted"));
                        return Flow::Break;
                    };
                    let transition =
                        Transition::new(id, self.next_sequence, before.clone(), after.clone());
                    if self.tracing {
                        eprintln!("[doublets transactions] {transition:?}");
                    }
                    // Keep the record even if append fails: the mutation already happened.
                    self.transitions.push(transition.clone());
                    self.next_sequence = next;
                    if let Err(error) = self.journal.append(&JournalEntry::Transition(transition)) {
                        log_error = Some(error);
                        return Flow::Break;
                    }
                }
                handler.call(before, after)
            })
        }));
        let result = match result {
            Ok(result) => result,
            Err(panic) => {
                pending.failed = true;
                if automatic {
                    let _ = self.rollback();
                }
                std::panic::resume_unwind(panic);
            }
        };
        let result = if let Some(error) = log_error {
            self.poisoned = true;
            Err(error.into())
        } else {
            result
        };
        if result.is_err() {
            pending.failed = true;
        }
        if automatic {
            match result {
                Ok(flow) => {
                    if let Err(error) = self.commit() {
                        self.rollback()?;
                        return Err(error);
                    }
                    Ok(flow)
                }
                Err(error) => {
                    self.rollback()?;
                    Err(error)
                }
            }
        } else {
            result
        }
    }
}

impl<T: LinkReference, L: Doublets<T>, J: TransitionLog<T>> Links<T>
    for TransactionsDecorator<T, L, J>
{
    forward!(count_links, each_links);

    fn create_links(
        &mut self,
        query: &[T],
        handler: WriteHandler<'_, T>,
    ) -> Result<Flow, Error<T>> {
        self.write(handler, |links, handler| links.create_links(query, handler))
    }
    fn update_links(
        &mut self,
        query: &[T],
        change: &[T],
        handler: WriteHandler<'_, T>,
    ) -> Result<Flow, Error<T>> {
        self.write(handler, |links, handler| {
            links.update_links(query, change, handler)
        })
    }
    fn delete_links(
        &mut self,
        query: &[T],
        handler: WriteHandler<'_, T>,
    ) -> Result<Flow, Error<T>> {
        self.write(handler, |links, handler| links.delete_links(query, handler))
    }
}

impl<T: LinkReference, L: Doublets<T>, J: TransitionLog<T>> Doublets<T>
    for TransactionsDecorator<T, L, J>
{
    fn get_link(&self, index: T) -> Option<Link<T>> {
        self.links.get_link(index)
    }
}

/// An exclusively borrowed transaction; unfinished handles roll back on drop.
///
/// `commit` and `rollback` consume the handle. If commit returns an I/O error,
/// its outcome is indeterminate until journal recovery: an append may have reached
/// disk despite reporting failure. Further writes are blocked on that decorator.
/// Call explicit rollback to receive restoration errors; drop records them in
/// [`TransactionsDecorator::rollback_error`] and never panics intentionally.
#[must_use = "commit or rollback the transaction; dropping it rolls back"]
pub struct Transaction<'a, T: LinkReference, L: Doublets<T>, J: TransitionLog<T>> {
    store: &'a mut TransactionsDecorator<T, L, J>,
}

impl<T: LinkReference, L: Doublets<T>, J: TransitionLog<T>> Transaction<'_, T, L, J> {
    /// Commits the transaction after persisting the complete resulting graph.
    pub fn commit(self) -> Result<(), Error<T>> {
        self.store.commit()
    }
    /// Restores the original graph and records the rollback.
    pub fn rollback(self) -> Result<(), Error<T>> {
        self.store.rollback()
    }
    /// Transitions recorded by this transaction so far.
    #[must_use]
    pub fn transitions(&self) -> &[Transition<T>] {
        let start = self
            .store
            .pending
            .as_ref()
            .map_or(self.store.transitions.len(), |p| p.start);
        &self.store.transitions[start..]
    }
}

impl<T: LinkReference, L: Doublets<T>, J: TransitionLog<T>> Drop for Transaction<'_, T, L, J> {
    fn drop(&mut self) {
        let _ = self.store.rollback();
    }
}

impl<T: LinkReference, L: Doublets<T>, J: TransitionLog<T>> Deref for Transaction<'_, T, L, J> {
    type Target = TransactionsDecorator<T, L, J>;
    fn deref(&self) -> &Self::Target {
        self.store
    }
}
impl<T: LinkReference, L: Doublets<T>, J: TransitionLog<T>> DerefMut for Transaction<'_, T, L, J> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.store
    }
}

// Implement Links on the handle itself so it composes with DecoratorsExt.
impl<T: LinkReference, L: Doublets<T>, J: TransitionLog<T>> Links<T> for Transaction<'_, T, L, J> {
    fn constants(&self) -> &LinksConstants<T> {
        self.store.constants()
    }
    fn count_links(&self, query: &[T]) -> T {
        self.store.count_links(query)
    }
    fn each_links(&self, query: &[T], handler: ReadHandler<'_, T>) -> Flow {
        self.store.each_links(query, handler)
    }
    fn create_links(
        &mut self,
        query: &[T],
        handler: WriteHandler<'_, T>,
    ) -> Result<Flow, Error<T>> {
        self.store.create_links(query, handler)
    }
    fn update_links(
        &mut self,
        query: &[T],
        change: &[T],
        handler: WriteHandler<'_, T>,
    ) -> Result<Flow, Error<T>> {
        self.store.update_links(query, change, handler)
    }
    fn delete_links(
        &mut self,
        query: &[T],
        handler: WriteHandler<'_, T>,
    ) -> Result<Flow, Error<T>> {
        self.store.delete_links(query, handler)
    }
}
impl<T: LinkReference, L: Doublets<T>, J: TransitionLog<T>> Doublets<T>
    for Transaction<'_, T, L, J>
{
    fn get_link(&self, index: T) -> Option<Link<T>> {
        self.store.get_link(index)
    }
}

fn failure<T: LinkReference>(message: &str) -> Error<T> {
    io::Error::other(message).into()
}

fn snapshot<T: LinkReference, L: Doublets<T>>(links: &L) -> Vec<Link<T>> {
    let mut snapshot = Vec::new();
    links.each_links(&[], &mut |link| {
        snapshot.push(link);
        Flow::Continue
    });
    snapshot.sort_by_key(|link| link.index);
    snapshot
}

fn validate<T: LinkReference, L: Doublets<T>>(
    links: &L,
    desired: &[Link<T>],
) -> Result<(), Error<T>> {
    let mut indexes = BTreeSet::new();
    for link in desired {
        if !links.constants().is_internal(link.index) || !indexes.insert(link.index) {
            return Err(failure(
                "snapshot contains an invalid or duplicate link address",
            ));
        }
    }
    Ok(())
}

fn restore<T: LinkReference, L: Doublets<T>>(
    links: &mut L,
    desired: &[Link<T>],
) -> Result<(), Error<T>> {
    validate(links, desired)?;
    let current = snapshot(links);
    let desired: BTreeMap<_, _> = desired.iter().map(|link| (link.index, link)).collect();
    if current.iter().eq(desired.values().copied()) {
        return Ok(());
    }
    // Clear indexed contents first: applying snapshots one link at a time can
    // transiently duplicate pairs (e.g. swapping two doublets) and damage trees.
    let null = T::from_byte(0);
    for link in &current {
        if link.source != null || link.target != null {
            links.update(link.index, null, null)?;
        }
    }
    let mut allocated: BTreeSet<_> = current.iter().map(|link| link.index).collect();
    let mut missing: BTreeSet<_> = desired
        .keys()
        .copied()
        .filter(|index| !allocated.contains(index))
        .collect();
    let limit = current
        .iter()
        .map(|link| link.index)
        .chain(desired.keys().copied())
        .max()
        .unwrap_or(null);
    while !missing.is_empty() {
        let index = links.create()?;
        if !allocated.insert(index) {
            return Err(failure("store did not allocate a fresh link address"));
        }
        if index > limit {
            return Err(failure("store cannot restore requested link addresses"));
        }
        missing.remove(&index);
    }
    for index in allocated.into_iter().rev() {
        if !desired.contains_key(&index) {
            links.delete(index)?;
        }
    }
    for link in desired.values() {
        if link.source != null || link.target != null {
            links.update(link.index, link.source, link.target)?;
        }
    }
    Ok(())
}
