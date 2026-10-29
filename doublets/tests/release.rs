//! Release policy, safety boundary and structural-index regressions for issue 81.

use data::Flow;
use doublets::{
    decorators::{CheckedRelease, DecoratorsExt, NonNullContentsLinkDeletionResolver},
    mem::Release,
    split, unit, Doublets, Error, Link,
};
use mem::Global;

fn checked_release_requires_reset(mut store: impl Release<u32>) -> Result<(), Error<u32>> {
    for endpoints in [(1, 1), (1, 0), (0, 1), (99, 100)] {
        let index = store.create()?;
        store.update(index, endpoints.0, endpoints.1)?;
        let before = store.try_get_link(index)?;
        let mut events = Vec::new();
        assert!(matches!(
            store.release_with(index, &mut |a, b| {
                events.push((a, b));
                Flow::Continue
            }),
            Err(Error::NonNullContents(link)) if link == before
        ));
        assert_eq!(events, []);
        assert_eq!(store.get_link(index), Some(before));
        store.update(index, 0, 0)?;
        assert_eq!(store.release(index)?, index);
    }
    Ok(())
}

fn checked_release_rejects_invalid_addresses(
    mut store: impl Release<u32>,
) -> Result<(), Error<u32>> {
    for _ in 0..3 {
        store.create()?;
    }
    store.release(2)?;
    let any = store.constants().any;
    for index in [0, 2, 4, any, u32::MAX] {
        let mut calls = 0;
        assert!(matches!(store.release_with(index, &mut |_, _| {
            calls += 1;
            Flow::Continue
        }), Err(Error::NotExists(value)) if value == index));
        assert_eq!(calls, 0);
        assert_eq!(store.count(), 2);
    }
    store.release(3)?;
    store.release(1)?;
    assert!(matches!(store.release(1), Err(Error::NotExists(1))));
    assert_eq!(store.create()?, 1);
    Ok(())
}

fn release_callbacks_and_reuse(mut store: impl Release<u32>) -> Result<(), Error<u32>> {
    for _ in 0..4 {
        store.create()?;
    }
    for index in [2, 3, 4, 1] {
        let mut events = Vec::new();
        assert_eq!(
            store.release_with(index, &mut |before, after| {
                events.push((before, after));
                Flow::Break
            })?,
            Flow::Break
        );
        assert_eq!(events, [(Link::new(index, 0, 0), Link::nothing())]);
        assert!(!store.exist(index));
    }
    assert_eq!(store.count(), 0);
    for index in 1..=4 {
        assert_eq!(store.create()?, index);
        assert_eq!(store.get_link(index), Some(Link::new(index, 0, 0)));
    }
    store.release(2)?;
    store.release(3)?;
    assert_eq!(store.create()?, 3);
    assert_eq!(store.create()?, 2);
    Ok(())
}

fn incoming_references_survive_release_and_reuse(
    mut store: impl Release<u32>,
) -> Result<(), Error<u32>> {
    for _ in 0..6 {
        store.create()?;
    }
    // Both an interior free-list slot and the allocated tail.
    for root in [5, 6] {
        for (index, source, target) in [(1, root, 2), (2, 3, root), (3, root, root)] {
            store.update(index, source, target)?;
        }
        for released in [true, false] {
            if released {
                assert_eq!(store.release(root)?, root);
            } else {
                assert_eq!(store.create()?, root);
            }
            assert_eq!(store.exist(root), !released);
            let any = store.constants().any;
            for (query, expected) in [
                ([any, root, any], vec![1, 3]),
                ([any, any, root], vec![2, 3]),
            ] {
                let mut found = Vec::new();
                store.each_by(query, |link| {
                    found.push(link.index);
                    Flow::Continue
                });
                found.sort_unstable();
                assert_eq!(found, expected);
                assert_eq!(store.count_by(query), 2);
            }
            for (source, target, index) in [(root, 2, 1), (3, root, 2), (root, root, 3)] {
                assert_eq!(store.search(source, target), Some(index));
            }
        }
        // Mutation after reclassification must detach from the right indexes.
        for index in 1..=3 {
            store.update(index, 0, 0)?;
        }
    }
    Ok(())
}

fn reset_is_an_explicit_layer(mut store: impl Release<u32>) -> Result<(), Error<u32>> {
    let index = store.create_point()?;
    let mut checked = CheckedRelease::new(&mut store);
    assert!(matches!(
        checked.delete(index),
        Err(Error::NonNullContents(_))
    ));
    assert!(matches!(checked.delete_by([]), Err(Error::NotExists(0))));
    let mut resetting = checked.with_non_null_contents_deletion_resolution();
    let mut seen = Vec::new();
    assert_eq!(
        resetting.delete_with(index, |before, after| {
            seen.push((before, after));
            Flow::Continue
        })?,
        Flow::Continue
    );
    assert_eq!(
        seen,
        [
            (Link::point(index), Link::new(index, 0, 0)),
            (Link::new(index, 0, 0), Link::nothing()),
        ]
    );
    assert!(!resetting.exist(index));
    Ok(())
}

fn resetting_layer_fuses_callbacks(mut store: impl Release<u32>) -> Result<(), Error<u32>> {
    for stop in 1..=2 {
        let index = store.create_point()?;
        let mut resetting =
            NonNullContentsLinkDeletionResolver::new(CheckedRelease::new(&mut store));
        let mut calls = 0;
        assert_eq!(
            resetting.delete_with(index, |_, _| {
                calls += 1;
                if calls == stop {
                    Flow::Break
                } else {
                    Flow::Continue
                }
            })?,
            Flow::Break
        );
        assert_eq!(calls, stop);
        assert_eq!(resetting.count(), 0);
        let index = resetting.create()?;
        let mut seen = Vec::new();
        assert_eq!(resetting.delete(index)?, index);
        let index = resetting.create()?;
        resetting.delete_with(index, |a, b| {
            seen.push((a, b));
            Flow::Continue
        })?;
        assert_eq!(seen, [(Link::new(index, 0, 0), Link::nothing())]);
    }
    Ok(())
}

fn unchecked_release_of_valid_reset_links(mut store: impl Release<u32>) -> Result<(), Error<u32>> {
    let index = store.create_point()?;
    store.update(index, 0, 0)?;
    let mut seen = Vec::new();
    // SAFETY: index is live, reset through the store API, and all indexes are consistent.
    let flow = unsafe {
        store.release_unchecked(index, &mut |before, after| {
            seen.push((before, after));
            Flow::Break
        })
    };
    assert_eq!(flow, Flow::Break);
    assert_eq!(seen, [(Link::new(index, 0, 0), Link::nothing())]);
    assert!(!store.exist(index));
    assert_eq!(store.create()?, index);
    Ok(())
}

fn usage_policy_composes_with_release(mut store: impl Release<u32>) -> Result<(), Error<u32>> {
    let root = store.create()?;
    let usage = store.create_link(root, root)?;
    {
        let mut checked = CheckedRelease::new(&mut store).with_usages_validation();
        assert!(matches!(checked.delete(root), Err(Error::HasUsages(_))));
        assert!(checked.exist(root));
        assert!(checked.exist(usage));
    }
    let mut automatic =
        CheckedRelease::new(&mut store).with_automatic_uniqueness_and_usages_resolution();
    let mut seen = Vec::new();
    assert_eq!(
        automatic.delete_with(root, |before, after| {
            seen.push((before, after));
            Flow::Continue
        })?,
        Flow::Continue
    );
    assert_eq!(
        seen,
        [
            (Link::new(usage, root, root), Link::new(usage, 0, 0)),
            (Link::new(usage, 0, 0), Link::nothing()),
            (Link::new(root, 0, 0), Link::nothing()),
        ]
    );
    assert_eq!(automatic.count(), 0);
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
    checked_release_requires_reset,
    checked_release_rejects_invalid_addresses,
    release_callbacks_and_reuse,
    incoming_references_survive_release_and_reuse,
    reset_is_an_explicit_layer,
    resetting_layer_fuses_callbacks,
    unchecked_release_of_valid_reset_links,
    usage_policy_composes_with_release,
);
