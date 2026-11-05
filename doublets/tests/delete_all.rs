//! Regression coverage for deleting stores with holes (issue #65).

use doublets::{decorators::DecoratorsExt, split, unit, Doublets, DoubletsExt, Error, Link};
use mem::Global;

fn delete_all_with_a_middle_hole(mut store: impl Doublets<u32>) -> Result<(), Error<u32>> {
    for _ in 0..3 {
        store.create_point()?;
    }
    store.delete(2)?;
    assert_eq!(
        store.iter().collect::<Vec<_>>(),
        [Link::point(1), Link::point(3)]
    );

    store.delete_all()?;

    assert_eq!(store.count(), 0);
    assert_eq!(store.iter().next(), None);
    Ok(())
}

fn delete_all_with_a_leading_hole(mut store: impl Doublets<u32>) -> Result<(), Error<u32>> {
    for _ in 0..3 {
        store.create_point()?;
    }
    store.delete(1)?;
    assert_eq!(
        store.iter().collect::<Vec<_>>(),
        [Link::point(2), Link::point(3)]
    );

    store.delete_all()?;

    assert_eq!(store.count(), 0);
    assert_eq!(store.iter().next(), None);
    Ok(())
}

fn delete_all_with_every_small_hole_pattern(
    mut store: impl Doublets<u32>,
) -> Result<(), Error<u32>> {
    // All 32 subsets of five points cover empty, dense, sparse, and single-link stores.
    for holes in 0..32 {
        for expected in 1..=5 {
            assert_eq!(store.create_point()?, expected);
        }
        for index in 1..=5 {
            if holes & (1 << (index - 1)) != 0 {
                store.delete(index)?;
            }
        }

        store.delete_all()?;

        assert_eq!(store.count(), 0, "hole pattern {holes:05b}");
        assert_eq!(store.iter().next(), None, "hole pattern {holes:05b}");
        for index in 1..=5 {
            assert!(
                !store.exist(index),
                "hole pattern {holes:05b}, index {index}"
            );
        }
        // Repeated deletion is harmless, and allocation starts over at one.
        store.delete_all()?;
        let point = store.create_point()?;
        assert_eq!(point, 1);
        assert_eq!(store.get_link(point), Some(Link::point(point)));
        store.delete(point)?;
    }
    Ok(())
}

fn delete_all_preserves_cascading_deletion(
    mut store: impl Doublets<u32>,
) -> Result<(), Error<u32>> {
    let usage = store.create_point()?;
    let hole = store.create_point()?;
    let root = store.create_point()?;
    store.update(usage, root, usage)?;
    store.delete(hole)?;
    let mut store = store.with_automatic_uniqueness_and_usages_resolution();

    // Deleting the highest address also deletes the lower address through the cascade.
    store.delete_all()?;

    assert_eq!(store.count(), 0);
    assert_eq!(store.iter().next(), None);
    store.delete_all()?;
    assert_eq!(store.create_point()?, 1);
    Ok(())
}

fn delete_all_deletes_dependents_before_dependencies(
    mut store: impl Doublets<u32>,
) -> Result<(), Error<u32>> {
    let source = store.create_point()?;
    let target = store.create_point()?;
    let hole = store.create_point()?;
    let usage = store.create_link(source, target)?;
    store.create_link(usage, usage)?;
    store.delete(hole)?;
    let mut store = store.with_usages_validation();

    store.delete_all()?;

    assert_eq!(store.count(), 0);
    assert_eq!(store.iter().next(), None);
    Ok(())
}

fn delete_all_propagates_deletion_errors(mut store: impl Doublets<u32>) -> Result<(), Error<u32>> {
    let usage = store.create_point()?;
    let hole = store.create_point()?;
    let root = store.create_point()?;
    store.update(usage, root, usage)?;
    store.delete(hole)?;
    let mut store = store.with_usages_validation();

    assert!(matches!(store.delete_all(), Err(Error::HasUsages(_))));
    assert_eq!(store.count(), 2);
    assert!(store.exist(usage));
    assert!(store.exist(root));
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
    delete_all_with_a_middle_hole,
    delete_all_with_a_leading_hole,
    delete_all_with_every_small_hole_pattern,
    delete_all_preserves_cascading_deletion,
    delete_all_deletes_dependents_before_dependencies,
    delete_all_propagates_deletion_errors,
);
