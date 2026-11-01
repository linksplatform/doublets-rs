//! Checked mutation, unchecked primitives and missing-deletion policies (issue 80).
//!
//! Unchecked primitives are only exercised with addresses that satisfy their
//! documented preconditions; invalid addresses are tested through safe APIs.

use data::Flow;
use doublets::{
    decorators::{
        CheckedMutation, CheckedRelease, DecoratorsExt, NoExceptionsDecorator,
        NonExistentDeletionResolver,
    },
    mem::UncheckedMutation,
    split, unit, Doublets, Error, Link,
};
use mem::Global;

type Snapshot = (u32, Vec<Link<u32>>);

fn snapshot(store: &impl Doublets<u32>) -> Snapshot {
    let mut links = Vec::new();
    store.each(|link| {
        links.push(link);
        Flow::Continue
    });
    (store.count(), links)
}

/// Live links 1, 3 and 4 with references; 2 is a released (free-list) slot.
fn populate(store: &mut impl Doublets<u32>) -> Result<(), Error<u32>> {
    for _ in 0..4 {
        store.create()?;
    }
    store.update(1, 1, 3)?;
    store.update(3, 4, 1)?;
    store.update(4, 4, 4)?;
    store.update(2, 0, 0)?;
    store.delete(2)?;
    Ok(())
}

fn missing_addresses(store: &impl Doublets<u32>) -> [u32; 5] {
    [0, 2, 5, store.constants().any, u32::MAX]
}

fn safe_mutation_rejects_missing_addresses(
    mut store: impl UncheckedMutation<u32>,
) -> Result<(), Error<u32>> {
    populate(&mut store)?;
    let before = snapshot(&store);
    for index in missing_addresses(&store) {
        let mut calls = 0;
        let mut count = |_, _| {
            calls += 1;
            Flow::Continue
        };
        assert!(matches!(
            store.update_with(index, 1, 1, &mut count),
            Err(Error::NotExists(value)) if value == index
        ));
        assert!(matches!(
            store.delete_with(index, &mut count),
            Err(Error::NotExists(value)) if value == index
        ));
        {
            let mut checked = CheckedMutation::new(&mut store);
            assert!(matches!(
                checked.update_with(index, 1, 1, &mut count),
                Err(Error::NotExists(value)) if value == index
            ));
            assert!(matches!(
                checked.delete_with(index, &mut count),
                Err(Error::NotExists(value)) if value == index
            ));
        }
        assert_eq!(calls, 0, "rejected calls must not report a change");
        assert_eq!(snapshot(&store), before, "rejected calls must not mutate");
    }
    // Short queries name the null address instead of panicking.
    assert!(matches!(
        store.update_by([], [1, 1]),
        Err(Error::NotExists(0))
    ));
    assert!(matches!(store.delete_by([]), Err(Error::NotExists(0))));
    assert_eq!(snapshot(&store), before);
    // The free list is intact: the released slot is reused first.
    assert_eq!(store.create()?, 2);
    assert_eq!(store.create()?, 5);
    Ok(())
}

fn valid_unchecked_updates(mut store: impl UncheckedMutation<u32>) -> Result<(), Error<u32>> {
    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create()?;
    let any = store.constants().any;
    let unallocated = 1000;
    for (source, target) in [(a, b), (c, c), (b, unallocated), (0, a), (b, a)] {
        let previous = store.try_get_link(c)?;
        let mut events = Vec::new();
        // SAFETY: `c` is live and every mutation went through the store API.
        let flow = unsafe {
            store.update_unchecked(c, source, target, &mut |before, after| {
                events.push((before, after));
                Flow::Break
            })
        };
        // `Break` reports the handler's flow; it does not undo the update.
        assert_eq!(flow, Flow::Break);
        let current = Link::new(c, source, target);
        assert_eq!(events, [(previous.clone(), current.clone())]);
        assert_eq!(store.get_link(c), Some(current));
        if source != 0 && target != 0 {
            assert_eq!(store.search(source, target), Some(c));
        }
        if previous.source != 0 && previous.target != 0 {
            assert_eq!(store.search(previous.source, previous.target), None);
        }
        let mut by_source = Vec::new();
        store.each_by([any, source, any], |link| {
            by_source.push(link.index);
            Flow::Continue
        });
        assert_eq!(by_source.contains(&c), source != 0);
    }
    // The other links and their indexes are untouched.
    assert_eq!(store.search(a, a), Some(a));
    assert_eq!(store.search(b, b), Some(b));
    assert_eq!(store.count_by([any, any, a]), 2);

    // Unchecked deletion is the caller's composition of reset and release.
    let mut events = Vec::new();
    // SAFETY: `c` is live; the reset detaches its outgoing index memberships.
    unsafe {
        store.update_unchecked(c, 0, 0, &mut |_, _| Flow::Continue);
        store.release_unchecked(c, &mut |before, after| {
            events.push((before, after));
            Flow::Continue
        });
    }
    assert_eq!(events, [(Link::new(c, 0, 0), Link::nothing())]);
    assert!(!store.exist(c));
    assert_eq!(store.count_by([any, any, a]), 1);
    assert_eq!(store.create()?, c);
    Ok(())
}

fn checked_mutation_routes(mut store: impl UncheckedMutation<u32>) -> Result<(), Error<u32>> {
    let a = store.create_point()?;
    let b = store.create()?;
    let mut checked = CheckedMutation::new(&mut store);
    let mut events = Vec::new();
    checked.update_with(b, a, b, |before, after| {
        events.push((before, after));
        Flow::Continue
    })?;
    assert_eq!(events, [(Link::new(b, 0, 0), Link::new(b, a, b))]);
    // Endpoint references are not this adapter's contract.
    assert_eq!(checked.update(b, a, 77)?, b);
    events.clear();
    // Deletion resets and releases, reported as one original-to-nothing event.
    for index in [b, a] {
        assert_eq!(
            checked.delete_with(index, |before, after| {
                events.push((before, after));
                Flow::Break
            })?,
            Flow::Break
        );
    }
    assert_eq!(
        events,
        [
            (Link::new(b, a, 77), Link::nothing()),
            (Link::point(a), Link::nothing()),
        ]
    );
    assert_eq!(checked.count(), 0);
    assert_eq!(checked.inner().count(), 0);
    assert_eq!(store.create()?, a);
    Ok(())
}

fn missing_deletion_is_ignored_only_when_selected(
    mut store: impl UncheckedMutation<u32>,
) -> Result<(), Error<u32>> {
    populate(&mut store)?;
    let before = snapshot(&store);
    let mut resolver = NonExistentDeletionResolver::new(CheckedMutation::new(&mut store));
    for index in missing_addresses(&resolver) {
        let mut calls = 0;
        assert_eq!(
            resolver.delete_with(index, |_, _| {
                calls += 1;
                Flow::Break
            })?,
            Flow::Continue
        );
        assert_eq!(calls, 0);
        // The resolver only changes deletion; updates still report the error.
        assert!(matches!(
            resolver.update(index, 1, 1),
            Err(Error::NotExists(_))
        ));
    }
    assert_eq!(resolver.delete_by([])?, 0);
    assert_eq!(snapshot(&resolver), before);
    // Live links are forwarded with their events.
    let mut events = Vec::new();
    resolver.delete_with(4, |before, after| {
        events.push((before, after));
        Flow::Continue
    })?;
    assert_eq!(events, [(Link::new(4, 4, 4), Link::nothing())]);
    // Without the resolver the same missing deletion is an error.
    assert!(matches!(store.delete(4), Err(Error::NotExists(4))));
    let mut validated =
        CheckedMutation::new(&mut store).with_inner_reference_existence_validation();
    assert!(matches!(validated.delete(4), Err(Error::NotExists(4))));
    Ok(())
}

fn missing_deletion_resolver_propagates_unrelated_errors(
    mut store: impl UncheckedMutation<u32>,
) -> Result<(), Error<u32>> {
    let root = store.create_point()?;
    let usage = store.create_link(root, root)?;
    {
        let mut guarded = CheckedMutation::new(&mut store)
            .with_usages_validation()
            .with_non_existent_deletion_resolution();
        assert!(matches!(guarded.delete(root), Err(Error::HasUsages(_))));
        assert!(guarded.exist(root));
    }
    {
        let mut guarded = CheckedRelease::new(&mut store).with_non_existent_deletion_resolution();
        assert!(matches!(
            guarded.delete(usage),
            Err(Error::NonNullContents(link)) if link == Link::new(usage, root, root)
        ));
        assert!(guarded.exist(usage));
    }
    // The swallow-everything policy is a different decorator.
    let mut swallowing = NoExceptionsDecorator::new(CheckedRelease::new(&mut store));
    assert!(swallowing.delete(usage).is_ok());
    assert!(swallowing.exist(usage));
    Ok(())
}

macro_rules! test_stores {
    ($($test:ident),+ $(,)?) => { $(
        mod $test {
            use super::*;
            #[test]
            fn unit() -> Result<(), Error<u32>> {
                $test(unit::Store::<u32, _>::new(Global::new())?)
            }
            #[test]
            fn split() -> Result<(), Error<u32>> {
                $test(split::Store::<u32, _, _>::new(Global::new(), Global::new())?)
            }
        }
    )+ };
}

test_stores!(
    safe_mutation_rejects_missing_addresses,
    valid_unchecked_updates,
    checked_mutation_routes,
    missing_deletion_is_ignored_only_when_selected,
    missing_deletion_resolver_propagates_unrelated_errors,
);
