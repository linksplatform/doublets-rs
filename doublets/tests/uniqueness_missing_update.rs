//! Updating a missing link through the uniqueness resolvers (issue 66).
//!
//! The uniqueness resolvers implement one policy — keeping `(source, target)`
//! pairs unique — and deliberately do not verify that the requested address is
//! live. A caller who wants `Error::NotExists` for a missing address selects
//! that contract explicitly: either by composing
//! `with_inner_reference_existence_validation()` *outside* the resolver or by a
//! per-call `try_get_link(index)?`. These tests pin both explicit routes and the
//! fact that omitting them adds no hidden check.

use data::Flow;
use doublets::{
    decorators::{CascadeResolve, DecoratorsExt, Resolve},
    split, unit, Doublets, Error, Link,
};
use mem::Global;

type Snapshot = (u32, Vec<Link<u32>>);

const MISSING: u32 = 99;

fn snapshot(store: &impl Doublets<u32>) -> Snapshot {
    let mut links = Vec::new();
    store.each(|link| {
        links.push(link);
        Flow::Continue
    });
    (store.count(), links)
}

/// `(1: 1 1) (2: 2 2) (3: 1 2)`, the store from the issue.
fn populate(store: &mut impl Doublets<u32>) -> Result<(), Error<u32>> {
    store.create_point()?;
    store.create_point()?;
    store.create_link(1, 2)?;
    Ok(())
}

/// Updates `MISSING` to `(source, target)` and returns the result together
/// with every `(before, after)` pair the handler observed.
fn update_missing(
    store: &mut impl Doublets<u32>,
    source: u32,
    target: u32,
) -> (Result<Flow, Error<u32>>, Vec<(Link<u32>, Link<u32>)>) {
    let mut events = Vec::new();
    let result = store.update_with(MISSING, source, target, |before, after| {
        events.push((before, after));
        Flow::Continue
    });
    (result, events)
}

/// Both repro updates fail with `NotExists(99)`, before any write or callback.
fn rejects_missing_address(mut store: impl Doublets<u32>) -> Result<(), Error<u32>> {
    populate(&mut store)?;
    let before = snapshot(&store);
    // `(3, 3)` is a new pair; `(1, 2)` already exists as link 3.
    for (source, target) in [(3, 3), (1, 2)] {
        let (result, events) = update_missing(&mut store, source, target);
        assert!(
            matches!(result, Err(Error::NotExists(MISSING))),
            "update({MISSING}, {source}, {target}) = {result:?}"
        );
        assert!(events.is_empty(), "no callback for a rejected update");
        assert_eq!(snapshot(&store), before, "a rejected update writes nothing");
    }
    Ok(())
}

/// Without an existence policy the resolver resolves the pair conflict and
/// nothing else: there is no hidden lookup of the requested address, so the
/// existing pair's link is reported and the store is not modified.
fn resolver_adds_no_existence_check(mut store: impl Doublets<u32>) -> Result<(), Error<u32>> {
    populate(&mut store)?;
    let before = snapshot(&store);
    let (result, events) = update_missing(&mut store, 1, 2);
    assert_eq!(result?, Flow::Continue);
    assert_eq!(events, vec![(Link::new(3, 1, 2), Link::new(3, 1, 2))]);
    assert_eq!(snapshot(&store), before);
    Ok(())
}

macro_rules! for_each_backend {
    ($($name:ident => $test:ident($($decorate:tt)*);)*) => {
        mod unit_store {
            use super::*;
            $(
                #[test]
                fn $name() -> Result<(), Error<u32>> {
                    $test(unit::Store::<u32, _>::new(Global::new())?$($decorate)*)
                }
            )*
        }

        mod split_store {
            use super::*;
            $(
                #[test]
                fn $name() -> Result<(), Error<u32>> {
                    $test(split::Store::<u32, _, _>::new(Global::new(), Global::new())?$($decorate)*)
                }
            )*
        }
    };
}

for_each_backend! {
    bare_store_rejects_missing_address => rejects_missing_address();

    validation_outside_automatic_resolution_rejects_missing_address =>
        rejects_missing_address(
            .with_automatic_uniqueness_and_usages_resolution()
            .with_inner_reference_existence_validation()
        );

    validation_outside_cascade_resolver_rejects_missing_address =>
        rejects_missing_address(
            .with_uniqueness(CascadeResolve)
            .with_inner_reference_existence_validation()
        );

    validation_outside_resolver_rejects_missing_address =>
        rejects_missing_address(
            .with_uniqueness(Resolve)
            .with_inner_reference_existence_validation()
        );

    automatic_resolution_adds_no_existence_check =>
        resolver_adds_no_existence_check(.with_automatic_uniqueness_and_usages_resolution());

    cascade_resolver_adds_no_existence_check =>
        resolver_adds_no_existence_check(.with_uniqueness(CascadeResolve));

    plain_resolver_adds_no_existence_check =>
        resolver_adds_no_existence_check(.with_uniqueness(Resolve));

    // Placement matters: inside the resolver the validator is never reached,
    // because the resolver answers the pair conflict without forwarding.
    validation_inside_resolver_is_not_reached =>
        resolver_adds_no_existence_check(
            .with_inner_reference_existence_validation()
            .with_automatic_uniqueness_and_usages_resolution()
        );
}

/// The per-call alternative: check only the calls that need it.
#[test]
fn per_call_lookup_rejects_missing_address() -> Result<(), Error<u32>> {
    let mut store = unit::Store::<u32, _>::new(Global::new())?
        .with_automatic_uniqueness_and_usages_resolution();
    populate(&mut store)?;
    let before = snapshot(&store);

    for (source, target) in [(3, 3), (1, 2)] {
        let result = store
            .try_get_link(MISSING)
            .and_then(|_| store.update(MISSING, source, target));
        assert!(matches!(result, Err(Error::NotExists(MISSING))));
        assert_eq!(snapshot(&store), before);
    }

    // A live address still goes through the resolver: merging 3 into the
    // existing `(1, 1)` pair at 1 reports the survivor.
    store.try_get_link(3)?;
    assert_eq!(store.update(3, 1, 1)?, 1);
    assert!(!store.exist(3));
    Ok(())
}
