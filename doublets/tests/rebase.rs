//! Regression tests for rebasing links that reference the old address on both sides.

use data::Flow;
use doublets::{decorators::DecoratorsExt, mem::Global, split, unit, Doublets, Error, Link};

fn rebase_both_endpoints(mut store: impl Doublets<u32>) -> Result<(), Error<u32>> {
    let a = store.create_point()?;
    let b = store.create_point()?;
    let old = store.create_link(a, b)?;
    let usage = store.create_link(old, old)?;
    let new = store.create_link(b, a)?;

    let mut seen = Vec::new();
    store.rebase_with(old, new, |before, after| {
        seen.push((before, after));
        Flow::Continue
    })?;

    assert_eq!(
        seen,
        vec![(Link::new(usage, old, old), Link::new(usage, new, new))]
    );
    assert_eq!(store.get_link(usage), Some(Link::new(usage, new, new)));
    assert_eq!(store.get_link(old), Some(Link::new(old, a, b)));
    assert!(!store.has_usages(old));
    assert_eq!(store.count(), 5);
    Ok(())
}

#[test]
fn unit_rebase_both_endpoints_once() -> Result<(), Error<u32>> {
    rebase_both_endpoints(unit::Store::<u32, _>::new(Global::new())?)
}

#[test]
fn split_rebase_both_endpoints_once() -> Result<(), Error<u32>> {
    rebase_both_endpoints(split::Store::<u32, _, _>::new(
        Global::new(),
        Global::new(),
    )?)
}

#[test]
fn unit_automatic_resolution_rebase_both_endpoints_once() -> Result<(), Error<u32>> {
    rebase_both_endpoints(
        unit::Store::<u32, _>::new(Global::new())?
            .with_automatic_uniqueness_and_usages_resolution(),
    )
}

#[test]
fn split_automatic_resolution_rebase_both_endpoints_once() -> Result<(), Error<u32>> {
    rebase_both_endpoints(
        split::Store::<u32, _, _>::new(Global::new(), Global::new())?
            .with_automatic_uniqueness_and_usages_resolution(),
    )
}

fn rebase_mixed_usages(mut store: impl Doublets<u32>) -> Result<(), Error<u32>> {
    let a = store.create_point()?;
    let b = store.create_point()?;
    let old = store.create_link(a, b)?;
    let new = store.create_link(b, a)?;
    let both = store.create_link(old, old)?;
    let source = store.create_link(old, a)?;
    let target = store.create_link(b, old)?;
    let unrelated = store.create_link(a, new)?;

    let mut seen = Vec::new();
    store.rebase_with(old, new, |before, after| {
        seen.push((before, after));
        Flow::Continue
    })?;
    seen.sort_by_key(|(before, _)| before.index);
    assert_eq!(
        seen,
        vec![
            (Link::new(both, old, old), Link::new(both, new, new)),
            (Link::new(source, old, a), Link::new(source, new, a)),
            (Link::new(target, b, old), Link::new(target, b, new)),
        ]
    );
    assert_eq!(store.get_link(both), Some(Link::new(both, new, new)));
    assert_eq!(store.get_link(source), Some(Link::new(source, new, a)));
    assert_eq!(store.get_link(target), Some(Link::new(target, b, new)));
    assert_eq!(
        store.get_link(unrelated),
        Some(Link::new(unrelated, a, new))
    );
    assert!(!store.has_usages(old));
    Ok(())
}

#[test]
fn unit_rebase_mixed_usages_once_each() -> Result<(), Error<u32>> {
    rebase_mixed_usages(unit::Store::<u32, _>::new(Global::new())?)
}

#[test]
fn split_rebase_mixed_usages_once_each() -> Result<(), Error<u32>> {
    rebase_mixed_usages(split::Store::<u32, _, _>::new(
        Global::new(),
        Global::new(),
    )?)
}

fn rebase_preserves_existing_contract(mut store: impl Doublets<u32>) -> Result<(), Error<u32>> {
    let old = store.create_point()?;
    let new = store.create_point()?;
    let usage = store.create()?;
    store.update(usage, old, usage)?;

    store.rebase_with(old, old, |_, _| panic!("rebasing to self must not update"))?;
    assert_eq!(store.get_link(usage), Some(Link::new(usage, old, usage)));
    assert!(matches!(
        store.rebase_with(99, new, |_, _| panic!("missing old must not update")),
        Err(Error::NotExists(99))
    ));

    assert_eq!(store.rebase(old, new)?, new);
    assert_eq!(store.get_link(old), Some(Link::point(old)));
    assert_eq!(store.get_link(usage), Some(Link::new(usage, new, usage)));
    assert!(!store.has_usages(old));
    Ok(())
}

#[test]
fn unit_rebase_preserves_existing_contract() -> Result<(), Error<u32>> {
    rebase_preserves_existing_contract(unit::Store::<u32, _>::new(Global::new())?)
}

#[test]
fn split_rebase_preserves_existing_contract() -> Result<(), Error<u32>> {
    rebase_preserves_existing_contract(split::Store::<u32, _, _>::new(
        Global::new(),
        Global::new(),
    )?)
}
