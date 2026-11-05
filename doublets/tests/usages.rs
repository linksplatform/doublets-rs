//! Regression tests for overlapping source and target usages (issue #63).

use data::Flow;
use doublets::{split, unit, Doublets, DoubletsExt, Error, Link};
use mem::Global;

fn referenced_link(store: &mut impl Doublets<u32>) -> Result<u32, Error<u32>> {
    let source = store.create_point()?;
    let target = store.create_point()?;
    store.create_link(source, target)
}

fn count_usages_counts_a_dual_reference_once(
    mut store: impl Doublets<u32>,
) -> Result<(), Error<u32>> {
    let value = referenced_link(&mut store)?;
    store.create_link(value, value)?;

    assert_eq!(store.count_usages(value)?, 1);
    Ok(())
}

fn usages_lists_a_dual_reference_once(mut store: impl Doublets<u32>) -> Result<(), Error<u32>> {
    let value = referenced_link(&mut store)?;
    let usage = store.create_link(value, value)?;

    assert_eq!(store.usages(value)?, [usage]);
    Ok(())
}

fn delete_usages_deletes_a_dual_reference_once(
    mut store: impl Doublets<u32>,
) -> Result<(), Error<u32>> {
    let value = referenced_link(&mut store)?;
    let usage = store.create_link(value, value)?;

    store.delete_usages(value)?;

    assert!(!store.exist(usage));
    assert!(store.exist(value));
    assert_eq!(store.count(), 3);
    assert_eq!(store.count_usages(value)?, 0);
    Ok(())
}

fn delete_usages_reports_a_dual_reference_once(
    mut store: impl Doublets<u32>,
) -> Result<(), Error<u32>> {
    let value = referenced_link(&mut store)?;
    let usage = store.create_link(value, value)?;
    let mut deleted = Vec::new();

    store.delete_usages_with(value, |before, after| {
        deleted.push((before, after));
        Flow::Continue
    })?;

    assert_eq!(deleted, [(Link::new(usage, value, value), Link::nothing())]);
    assert!(!store.exist(usage));
    assert!(store.exist(value));
    Ok(())
}

fn delete_query_deletes_a_point_once(mut store: impl Doublets<u32>) -> Result<(), Error<u32>> {
    let source = store.create_point()?;
    let target = store.create_point()?;
    let usage = store.create_link(source, target)?;
    let any = store.constants().any;
    let mut deleted = Vec::new();

    store.delete_query_with([any, source], |before, after| {
        assert_eq!(after, Link::nothing());
        deleted.push(before.index);
        Flow::Continue
    })?;

    deleted.sort_unstable();
    assert_eq!(deleted, [source, usage]);
    assert!(!store.exist(source));
    assert!(!store.exist(usage));
    assert!(store.exist(target));
    assert_eq!(store.count(), 1);
    Ok(())
}

fn delete_query_deletes_a_dual_reference_once(
    mut store: impl Doublets<u32>,
) -> Result<(), Error<u32>> {
    let value = referenced_link(&mut store)?;
    let usage = store.create_link(value, value)?;
    let any = store.constants().any;
    let mut deleted = Vec::new();

    store.delete_query_with([any, value], |before, _| {
        deleted.push(before.index);
        Flow::Continue
    })?;

    assert_eq!(deleted, [usage]);
    assert!(!store.exist(usage));
    assert!(store.exist(value));
    Ok(())
}

fn mixed_usages_keep_source_then_target_order(
    mut store: impl Doublets<u32>,
) -> Result<(), Error<u32>> {
    let value = referenced_link(&mut store)?;
    let dual = store.create_link(value, value)?;
    let source_only = store.create_link(value, 1)?;
    let target_only = store.create_link(2, value)?;
    let any = store.constants().any;
    let mut expected: Vec<_> = store
        .each_iter([any, value, any])
        .map(|link| link.index)
        .collect();
    assert_eq!(expected.len(), 2);
    assert!(expected.contains(&dual));
    assert!(expected.contains(&source_only));
    expected.push(target_only);

    assert_eq!(store.count_usages(value)?, 3);
    assert_eq!(store.usages(value)?, expected);

    let mut deleted = Vec::new();
    store.delete_usages_with(value, |before, _| {
        deleted.push(before.index);
        Flow::Continue
    })?;
    expected.reverse();
    assert_eq!(deleted, expected);
    assert_eq!(store.count(), 3);
    assert_eq!(store.usages(value)?, [] as [u32; 0]);
    Ok(())
}

fn usages_exclude_full_and_partial_self_references(
    mut store: impl Doublets<u32>,
) -> Result<(), Error<u32>> {
    for self_reference in 0..3 {
        let other = store.create_point()?;
        let value = store.create()?;
        let usage_target = store.create_point()?;
        let (source, target) = match self_reference {
            0 => (value, value),
            1 => (value, other),
            _ => (other, value),
        };
        store.update(value, source, target)?;
        assert_eq!(store.count_usages(value)?, 0);
        assert_eq!(store.usages(value)?, [] as [u32; 0]);
        store.delete_usages_with(value, |_, _| panic!("a self reference is not a usage"))?;
        assert_eq!(
            store.get_link(value),
            Some(Link::new(value, source, target))
        );

        // Use a unique pair even when `value` is itself a full point.
        let usage = store.create_link(value, usage_target)?;
        assert_eq!(store.count_usages(value)?, 1);
        assert_eq!(store.usages(value)?, [usage]);
        store.delete_usages(value)?;
        assert!(!store.exist(usage));
        assert!(store.exist(value));
    }
    Ok(())
}

fn delete_query_keeps_reverse_unique_enumeration_order(
    mut store: impl Doublets<u32>,
) -> Result<(), Error<u32>> {
    let value = referenced_link(&mut store)?;
    store.create_link(value, value)?;
    store.create_link(value, 1)?;
    store.create_link(2, value)?;
    let any = store.constants().any;
    let mut expected = Vec::new();
    store.each_by([any, value], |link| {
        if !expected.contains(&link.index) {
            expected.push(link.index);
        }
        Flow::Continue
    });
    assert_eq!(expected.len(), 3);
    expected.reverse();

    let mut deleted = Vec::new();
    store.delete_query_with([any, value], |before, _| {
        deleted.push(before.index);
        Flow::Continue
    })?;

    assert_eq!(deleted, expected);
    assert_eq!(store.count(), 3);
    Ok(())
}

fn usage_reads_still_reject_missing_links(mut store: impl Doublets<u32>) -> Result<(), Error<u32>> {
    let missing = store.create_point()?;
    store.delete(missing)?;

    assert!(
        matches!(store.count_usages(missing), Err(Error::NotExists(index)) if index == missing)
    );
    assert!(matches!(store.usages(missing), Err(Error::NotExists(index)) if index == missing));
    Ok(())
}

fn deletion_handlers_remain_fused_after_break(
    mut store: impl Doublets<u32>,
) -> Result<(), Error<u32>> {
    for query_deletion in [false, true] {
        let value = referenced_link(&mut store)?;
        let source_only = store.create_link(value, 1)?;
        let dual = store.create_link(value, value)?;
        let mut calls = 0;
        let handler = |_: Link<u32>, _: Link<u32>| {
            calls += 1;
            Flow::Break
        };

        if query_deletion {
            let any = store.constants().any;
            store.delete_query_with([any, value], handler)?;
        } else {
            store.delete_usages_with(value, handler)?;
        }

        assert_eq!(calls, 1);
        assert!(!store.exist(source_only));
        assert!(!store.exist(dual));
        assert!(store.exist(value));
        store.delete_all()?;
    }
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
    count_usages_counts_a_dual_reference_once,
    usages_lists_a_dual_reference_once,
    delete_usages_deletes_a_dual_reference_once,
    delete_usages_reports_a_dual_reference_once,
    delete_query_deletes_a_point_once,
    delete_query_deletes_a_dual_reference_once,
    mixed_usages_keep_source_then_target_order,
    delete_query_keeps_reverse_unique_enumeration_order,
    usages_exclude_full_and_partial_self_references,
    usage_reads_still_reject_missing_links,
    deletion_handlers_remain_fused_after_break,
);
