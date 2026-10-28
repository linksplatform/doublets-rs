//! Regression tests for reset callbacks during cascaded deletion (issue #67).

use data::Flow;
use doublets::{decorators::DecoratorsExt, split, unit, Doublets, DoubletsExt, Error, Link};
use mem::Global;

fn automatic_resolution_reports_nested_resets(store: impl Doublets<u32>) -> Result<(), Error<u32>> {
    let mut store = store.with_automatic_uniqueness_and_usages_resolution();
    let a = store.create_point()?;
    let b = store.create_point()?;
    let ab = store.create_link(a, b)?;
    let nested = store.create_link(ab, ab)?;
    let mut seen = Vec::new();

    let flow = store.delete_with(a, |before, after| {
        seen.push((before, after));
        Flow::Continue
    })?;

    assert_eq!(flow, Flow::Continue);
    assert_eq!(
        seen,
        [
            (Link::point(a), Link::new(a, 0, 0)),
            (Link::new(ab, a, b), Link::new(ab, 0, 0)),
            (Link::new(nested, ab, ab), Link::new(nested, 0, 0)),
            (Link::new(nested, 0, 0), Link::nothing()),
            (Link::new(ab, 0, 0), Link::nothing()),
            (Link::new(a, 0, 0), Link::nothing()),
        ]
    );
    assert_eq!(store.iter().collect::<Vec<_>>(), [Link::point(b)]);
    Ok(())
}

fn automatic_resolution_fuses_handler_at_every_step(
    store: impl Doublets<u32>,
) -> Result<(), Error<u32>> {
    let mut store = store.with_automatic_uniqueness_and_usages_resolution();
    for stop_after in 1..=6 {
        let a = store.create_point()?;
        let b = store.create_point()?;
        let ab = store.create_link(a, b)?;
        store.create_link(ab, ab)?;
        let mut calls = 0;

        let flow = store.delete_with(a, |_, _| {
            calls += 1;
            if calls == stop_after {
                Flow::Break
            } else {
                Flow::Continue
            }
        })?;

        assert_eq!(flow, Flow::Break);
        assert_eq!(calls, stop_after);
        // Break fuses callbacks; the entire deletion still completes.
        assert_eq!(store.iter().collect::<Vec<_>>(), [Link::point(b)]);
        store.delete(b)?;
    }
    Ok(())
}

fn automatic_resolution_terminates_on_cycles(
    mut store: impl Doublets<u32>,
) -> Result<(), Error<u32>> {
    // Finite self, two-link, and three-link cycles exercise the visited-set behavior.
    for size in 1..=3 {
        let indices = (0..size)
            .map(|_| store.create_point())
            .collect::<Result<Vec<_>, _>>()?;
        let mut original = Vec::new();
        for (position, &index) in indices.iter().enumerate() {
            let next = indices[(position + 1) % indices.len()];
            store.update(index, index, next)?;
            original.push(Link::new(index, index, next));
        }
        let mut decorated = store.with_automatic_uniqueness_and_usages_resolution();
        let mut resets = Vec::new();
        let mut deletions = Vec::new();

        decorated.delete_with(indices[0], |before, after| {
            if after.is_null() {
                assert_eq!(before, Link::new(before.index, 0, 0));
                deletions.push(before.index);
            } else {
                assert_eq!(after, Link::new(before.index, 0, 0));
                resets.push(before);
            }
            Flow::Continue
        })?;

        original.sort_by_key(|link| link.index);
        resets.sort_by_key(|link| link.index);
        deletions.sort_unstable();
        assert_eq!(resets, original);
        assert_eq!(deletions, indices);
        assert_eq!(decorated.count(), 0);
        store = decorated.into_inner().into_inner().into_inner();
    }
    Ok(())
}

fn automatic_resolution_deletes_shared_usages_once(
    store: impl Doublets<u32>,
) -> Result<(), Error<u32>> {
    let mut store = store.with_automatic_uniqueness_and_usages_resolution();
    let a = store.create_point()?;
    let b = store.create_point()?;
    let source_usage = store.create_link(a, b)?;
    let target_usage = store.create_link(b, a)?;
    let shared = store.create_link(source_usage, target_usage)?;
    let nested = store.create_link(shared, shared)?;
    let mut resets = Vec::new();
    let mut deletions = Vec::new();

    store.delete_with(a, |before, after| {
        if after.is_null() {
            assert_eq!(before, Link::new(before.index, 0, 0));
            assert!(resets.contains(&before.index));
            assert!(!deletions.contains(&before.index));
            deletions.push(before.index);
        } else {
            assert_eq!(after, Link::new(before.index, 0, 0));
            assert!(!resets.contains(&before.index));
            resets.push(before.index);
        }
        Flow::Continue
    })?;

    let expected = [a, source_usage, target_usage, shared, nested];
    resets.sort_unstable();
    deletions.sort_unstable();
    assert_eq!(resets, expected);
    assert_eq!(deletions, expected);
    assert_eq!(store.iter().collect::<Vec<_>>(), [Link::point(b)]);
    Ok(())
}

fn automatic_resolution_skips_an_already_reset_root(
    store: impl Doublets<u32>,
) -> Result<(), Error<u32>> {
    let mut store = store.with_automatic_uniqueness_and_usages_resolution();
    let root = store.create()?;
    let b = store.create_point()?;
    let usage = store.create_link(root, b)?;
    let mut seen = Vec::new();

    store.delete_with(root, |before, after| {
        seen.push((before, after));
        Flow::Continue
    })?;

    assert_eq!(
        seen,
        [
            (Link::new(usage, root, b), Link::new(usage, 0, 0)),
            (Link::new(usage, 0, 0), Link::nothing()),
            (Link::new(root, 0, 0), Link::nothing()),
        ]
    );
    assert_eq!(store.iter().collect::<Vec<_>>(), [Link::point(b)]);
    Ok(())
}

macro_rules! test_both_stores {
    ($($test:ident),+ $(,)?) => {
        $(
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
        )+
    };
}

test_both_stores!(
    automatic_resolution_reports_nested_resets,
    automatic_resolution_fuses_handler_at_every_step,
    automatic_resolution_terminates_on_cycles,
    automatic_resolution_deletes_shared_usages_once,
    automatic_resolution_skips_an_already_reset_root,
);
