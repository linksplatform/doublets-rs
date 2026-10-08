use std::{
    io,
    panic::{catch_unwind, AssertUnwindSafe},
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc,
    },
};

use data::Flow;
use doublets::{
    decorators::{
        DecoratorsExt, JournalEntry, MemoryTransitionLog, Resolve, TransactionsDecorator,
        Transition, TransitionKind, TransitionLog,
    },
    mem::Global,
    split, unit, Doublets, DoubletsExt, Error, Link, Links,
};

#[test]
fn dropping_an_uncommitted_transaction_restores_the_graph() {
    let mut raw = unit::Store::<usize, _>::new(Global::new()).unwrap();
    let a = raw.create_point().unwrap();
    let b = raw.create_point().unwrap();
    let mut store = TransactionsDecorator::new(raw, MemoryTransitionLog::default()).unwrap();
    {
        let mut transaction = store.begin_transaction().unwrap();
        transaction.update(a, b, a).unwrap();
        transaction.delete(b).unwrap();
        transaction.create_point().unwrap();
    }
    assert_eq!(store.get_link(a), Some(Link::point(a)));
    assert_eq!(store.get_link(b), Some(Link::point(b)));
    assert_eq!(store.count(), 2);
}

fn exercise_transactions<L: Doublets<usize>>(mut raw: L) {
    let a = raw.create_point().unwrap();
    let b = raw.create_point().unwrap();
    let ab = raw.create_link(a, b).unwrap();
    let gap = raw.create_point().unwrap();
    let last = raw.create_point().unwrap();
    raw.delete(gap).unwrap();
    let original = raw.iter().collect::<Vec<_>>();
    let mut store = raw
        .with_transactions(MemoryTransitionLog::default())
        .unwrap();
    {
        let mut tx = store.begin_transaction().unwrap();
        tx.delete(ab).unwrap();
        tx.delete(last).unwrap();
        tx.update(a, b, a).unwrap();
        tx.create_point().unwrap();
        tx.create_link(a, b).unwrap();
        tx.rollback().unwrap();
    }
    assert_eq!(store.iter().collect::<Vec<_>>(), original);
    assert_eq!(store.search(a, b), Some(ab));
    assert!(!store.exist(gap));
    {
        let mut tx = store.begin_transaction().unwrap();
        assert!(tx.begin_transaction().is_err());
        tx.update(ab, b, a).unwrap();
        assert_eq!(tx.transitions()[0].kind, TransitionKind::Update);
        tx.commit().unwrap();
    }
    assert_eq!(store.get_link(ab), Some(Link::new(ab, b, a)));
    assert_eq!(store.search(b, a), Some(ab));
    let (raw, journal) = store.into_parts().unwrap();
    let store = raw.with_transactions(journal).unwrap();
    assert_eq!(store.get_link(ab), Some(Link::new(ab, b, a)));
}

#[test]
fn unit_commit_rollback_and_address_reuse() {
    exercise_transactions(unit::Store::<usize, _>::new(Global::new()).unwrap());
}

#[test]
fn split_commit_rollback_and_address_reuse() {
    exercise_transactions(split::Store::<usize, _, _>::new(Global::new(), Global::new()).unwrap());
}

#[test]
fn transition_pairs_and_handler_break_are_preserved() {
    let raw = unit::Store::<usize, _>::new(Global::new()).unwrap();
    let mut store = raw
        .with_transactions(MemoryTransitionLog::default())
        .unwrap();
    let mut tx = store.begin_transaction().unwrap();
    let mut seen = Vec::new();
    assert_eq!(
        tx.create_with(|before, after| {
            seen.push((before, after));
            Flow::Break
        })
        .unwrap(),
        Flow::Break
    );
    assert_eq!(seen, vec![(Link::nothing(), Link::new(1, 0, 0))]);
    tx.update(1, 1, 1).unwrap();
    tx.update(1, 1, 1).unwrap(); // unchanged writes do not add transitions
    tx.delete(1).unwrap();
    let transitions = tx.transitions();
    assert_eq!(transitions.len(), 3);
    assert_eq!(
        transitions.iter().map(|t| t.kind).collect::<Vec<_>>(),
        vec![
            TransitionKind::Create,
            TransitionKind::Update,
            TransitionKind::Delete
        ]
    );
    assert_eq!(transitions[1].before, Link::new(1, 0, 0));
    assert_eq!(transitions[1].after, Link::point(1));
    assert_eq!(transitions[2].before, Link::point(1));
    assert!(transitions[2].after.is_null());
    assert!(transitions
        .windows(2)
        .all(|pair| pair[0].sequence < pair[1].sequence));
    tx.commit().unwrap();
}

#[test]
fn policy_stack_around_handle_captures_duplicate_resolution() {
    let raw = unit::Store::<usize, _>::new(Global::new()).unwrap();
    let mut store = raw
        .with_transactions(MemoryTransitionLog::default())
        .unwrap();
    {
        let tx = store.begin_transaction().unwrap();
        let mut policies = tx.with_uniqueness(Resolve);
        let a = policies.create_point().unwrap();
        let b = policies.create_point().unwrap();
        assert_eq!(
            policies.create_link(a, b).unwrap(),
            policies.create_link(a, b).unwrap()
        );
        assert_eq!(policies.count(), 3);
        assert!(policies
            .inner()
            .transitions()
            .iter()
            .any(|t| t.kind == TransitionKind::Delete));
        // Dropping the policy stack also drops its transaction handle.
    }
    assert_eq!(store.count(), 0);
}

#[test]
fn explicit_and_automatic_transactions_roll_back_on_unwind() {
    let raw = unit::Store::<usize, _>::new(Global::new()).unwrap();
    let mut store = raw
        .with_transactions(MemoryTransitionLog::default())
        .unwrap();
    assert!(catch_unwind(AssertUnwindSafe(|| {
        let mut tx = store.begin_transaction().unwrap();
        tx.create_point().unwrap();
        panic!("abort explicit transaction");
    }))
    .is_err());
    assert_eq!(store.count(), 0);
    assert!(catch_unwind(AssertUnwindSafe(|| {
        store.create_with(|_, _| panic!("handler panic")).unwrap();
    }))
    .is_err());
    assert_eq!(store.count(), 0);
    store.create_point().unwrap();
    assert_eq!(store.count(), 1);
}

#[test]
fn caught_handler_panic_prevents_commit() {
    let raw = unit::Store::<usize, _>::new(Global::new()).unwrap();
    let mut store = raw
        .with_transactions(MemoryTransitionLog::default())
        .unwrap();
    let mut tx = store.begin_transaction().unwrap();
    assert!(catch_unwind(AssertUnwindSafe(|| {
        tx.create_with(|_, _| panic!("callback")).unwrap();
    }))
    .is_err());
    assert!(tx.commit().is_err());
    assert_eq!(store.count(), 0);
}

#[test]
fn failed_store_write_cannot_be_committed() {
    let raw = unit::Store::<usize, _>::new(Global::new()).unwrap();
    let mut store = raw
        .with_transactions(MemoryTransitionLog::default())
        .unwrap();
    let mut tx = store.begin_transaction().unwrap();
    tx.create_point().unwrap();
    assert!(tx.update(100, 1, 1).is_err());
    assert!(tx.create().is_err());
    assert!(tx.commit().is_err());
    assert_eq!(store.count(), 0);
}

fn exercise_replay<L: Doublets<usize>>(mut raw: L) {
    let a = raw.create_point().unwrap();
    let gap = raw.create_point().unwrap();
    let b = raw.create_point().unwrap();
    raw.delete(gap).unwrap();
    let t = Transition::new(1, 1, Link::point(b), Link::nothing());
    t.apply(&mut raw).unwrap();
    t.apply(&mut raw).unwrap();
    assert!(!raw.exist(b));
    t.revert(&mut raw).unwrap();
    t.revert(&mut raw).unwrap();
    assert_eq!(raw.get_link(b), Some(Link::point(b)));
    assert!(!raw.exist(gap));
    assert_eq!(raw.get_link(a), Some(Link::point(a)));
    let t = Transition::new(1, 2, Link::point(a), Link::new(a, b, a));
    t.apply(&mut raw).unwrap();
    assert_eq!(raw.get_link(a), Some(Link::new(a, b, a)));
    t.revert(&mut raw).unwrap();
    assert_eq!(raw.get_link(a), Some(Link::point(a)));
    let t = Transition::new(1, 3, Link::nothing(), Link::new(gap, a, b));
    t.apply(&mut raw).unwrap();
    assert_eq!(raw.get_link(gap), Some(Link::new(gap, a, b)));
    t.revert(&mut raw).unwrap();
    assert!(!raw.exist(gap));
}

#[test]
fn replay_and_revert_preserve_addresses_on_both_backends() {
    exercise_replay(unit::Store::<usize, _>::new(Global::new()).unwrap());
    exercise_replay(split::Store::<usize, _, _>::new(Global::new(), Global::new()).unwrap());
}

struct FailingLog {
    inner: MemoryTransitionLog<usize>,
    fail_at: usize,
    writes: usize,
}

impl TransitionLog<usize> for FailingLog {
    fn read_entries(&mut self) -> io::Result<Vec<JournalEntry<usize>>> {
        self.inner.read_entries()
    }
    fn append(&mut self, entry: &JournalEntry<usize>) -> io::Result<()> {
        self.writes += 1;
        if self.writes == self.fail_at {
            Err(io::Error::other("injected journal failure"))
        } else {
            self.inner.append(entry)
        }
    }
}

#[test]
fn begin_transition_and_commit_io_failures_do_not_leave_writes() {
    for fail_at in 1..=3 {
        let raw = unit::Store::<usize, _>::new(Global::new()).unwrap();
        let log = FailingLog {
            inner: MemoryTransitionLog::default(),
            fail_at,
            writes: 0,
        };
        let mut store = raw.with_transactions(log).unwrap();
        assert!(store.create().is_err());
        assert_eq!(store.count(), 0);
        assert!(store.is_poisoned());
        assert!(store.create().is_err());
        let (raw, journal) = store.into_parts().unwrap();
        let store = raw.with_transactions(journal.inner).unwrap();
        assert_eq!(store.count(), 0);
    }
}

#[test]
fn rollback_marker_failure_is_exposed_and_recoverable() {
    let raw = unit::Store::<usize, _>::new(Global::new()).unwrap();
    let log = FailingLog {
        inner: MemoryTransitionLog::default(),
        fail_at: 3,
        writes: 0,
    };
    let mut store = raw.with_transactions(log).unwrap();
    {
        let mut tx = store.begin_transaction().unwrap();
        tx.create().unwrap();
    }
    assert_eq!(store.count(), 0);
    assert!(store
        .rollback_error()
        .unwrap()
        .contains("injected journal failure"));
    assert!(store.is_poisoned());
    let (raw, log) = store.into_parts().unwrap();
    assert_eq!(raw.with_transactions(log.inner).unwrap().count(), 0);
}

struct BrokenRestore {
    raw: unit::Store<usize, Global<doublets::parts::LinkPart<usize>>>,
    mode: Arc<AtomicU8>,
}

impl Links<usize> for BrokenRestore {
    fn constants(&self) -> &data::LinksConstants<usize> {
        self.raw.constants()
    }
    fn count_links(&self, query: &[usize]) -> usize {
        self.raw.count_links(query)
    }
    fn each_links(&self, query: &[usize], handler: doublets::data::ReadHandler<'_, usize>) -> Flow {
        self.raw.each_links(query, handler)
    }
    fn create_links(
        &mut self,
        query: &[usize],
        handler: doublets::data::WriteHandler<'_, usize>,
    ) -> Result<Flow, Error<usize>> {
        self.raw.create_links(query, handler)
    }
    fn delete_links(
        &mut self,
        query: &[usize],
        handler: doublets::data::WriteHandler<'_, usize>,
    ) -> Result<Flow, Error<usize>> {
        self.raw.delete_links(query, handler)
    }
    fn update_links(
        &mut self,
        query: &[usize],
        change: &[usize],
        handler: doublets::data::WriteHandler<'_, usize>,
    ) -> Result<Flow, Error<usize>> {
        match self.mode.load(Ordering::Relaxed) {
            1 => Err(io::Error::other("injected restore failure").into()),
            2 => panic!("injected store panic"),
            _ => self.raw.update_links(query, change, handler),
        }
    }
}
impl Doublets<usize> for BrokenRestore {
    fn get_link(&self, index: usize) -> Option<Link<usize>> {
        self.raw.get_link(index)
    }
}

#[test]
fn drop_records_store_restoration_errors_and_panics_without_unwinding() {
    for failure_mode in [1, 2] {
        let mut raw = unit::Store::<usize, _>::new(Global::new()).unwrap();
        raw.create_point().unwrap();
        let mode = Arc::new(AtomicU8::new(0));
        let raw = BrokenRestore {
            raw,
            mode: mode.clone(),
        };
        let mut store = raw
            .with_transactions(MemoryTransitionLog::default())
            .unwrap();
        {
            let mut tx = store.begin_transaction().unwrap();
            tx.update(1, 0, 0).unwrap();
            mode.store(failure_mode, Ordering::Relaxed);
        }
        assert!(store.is_poisoned());
        assert!(store.rollback_error().is_some());
        assert!(store.create().is_err());
        mode.store(0, Ordering::Relaxed);
        let (raw, log) = store.into_parts().unwrap();
        let store = raw.with_transactions(log).unwrap();
        assert_eq!(store.get_link(1), Some(Link::point(1)));
    }
}

#[test]
fn transactions_support_u32_and_u64_addresses() {
    let raw = unit::Store::<u32, _>::new(Global::new()).unwrap();
    let mut store = raw
        .with_transactions(MemoryTransitionLog::default())
        .unwrap();
    let mut tx = store.begin_transaction().unwrap();
    let point = tx.create_point().unwrap();
    tx.commit().unwrap();
    assert_eq!(store.get_link(point), Some(Link::point(point)));
    let raw = split::Store::<u64, _, _>::new(Global::new(), Global::new()).unwrap();
    let mut store = raw
        .with_transactions(MemoryTransitionLog::default())
        .unwrap();
    let mut tx = store.begin_transaction().unwrap();
    tx.create_point().unwrap();
    tx.rollback().unwrap();
    assert_eq!(store.count(), 0);
}

#[test]
fn invalid_transition_states_are_rejected_before_mutation() {
    let mut raw = unit::Store::<usize, _>::new(Global::new()).unwrap();
    raw.create_point().unwrap();
    let transition = Transition::new(1, 1, Link::point(1), Link::point(2));
    assert!(transition.apply(&mut raw).is_err());
    assert!(transition.revert(&mut raw).is_err());
    assert_eq!(raw.get_link(1), Some(Link::point(1)));
    assert_eq!(raw.count(), 1);
}
