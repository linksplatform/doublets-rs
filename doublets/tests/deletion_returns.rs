//! Regression coverage for deletion helper return values (issue #64).

use doublets::{decorators::DecoratorsExt, split, unit, Doublets, Error, Link};
use mem::Global;

fn delete_returns_the_former_index(mut store: impl Doublets<u32>) -> Result<(), Error<u32>> {
    for _ in 0..3 {
        store.create_point()?;
    }

    assert_eq!(store.delete(3)?, 3);
    assert_eq!(store.get_link(3), None);
    assert_eq!(store.delete(1)?, 1);
    assert_eq!(store.get_link(1), None);
    assert_eq!(store.get_link(2), Some(Link::point(2)));
    assert_eq!(store.count(), 1);
    Ok(())
}

fn delete_by_returns_the_former_index(mut store: impl Doublets<u32>) -> Result<(), Error<u32>> {
    let source = store.create_point()?;
    let target = store.create_point()?;
    let link = store.create_link(source, target)?;

    assert_eq!(store.delete_by([link])?, link);
    assert_eq!(store.get_link(link), None);
    assert_eq!(store.search(source, target), None);
    assert_eq!(store.count(), 2);
    Ok(())
}

fn delete_returns_an_uninitialized_links_index(
    mut store: impl Doublets<u32>,
) -> Result<(), Error<u32>> {
    let link = store.create()?;
    assert_eq!(store.get_link(link), Some(Link::new(link, 0, 0)));
    assert_eq!(store.delete(link)?, link);
    assert_eq!(store.count(), 0);
    Ok(())
}

fn rebase_and_delete_returns_new_without_usages(
    mut store: impl Doublets<u32>,
) -> Result<(), Error<u32>> {
    for _ in 0..3 {
        store.create_point()?;
    }

    assert_eq!(store.rebase_and_delete(2, 1)?, 1);
    assert_eq!(store.get_link(2), None);
    assert_eq!(store.get_link(1), Some(Link::point(1)));
    assert_eq!(store.get_link(3), Some(Link::point(3)));
    assert_eq!(store.count(), 2);
    Ok(())
}

fn rebase_and_delete_returns_new_with_usages(
    mut store: impl Doublets<u32>,
) -> Result<(), Error<u32>> {
    let new = store.create_point()?;
    let old = store.create_point()?;
    let other = store.create_point()?;
    let source_usage = store.create_link(old, other)?;
    let target_usage = store.create_link(other, old)?;

    assert_eq!(store.rebase_and_delete(old, new)?, new);
    assert_eq!(store.get_link(old), None);
    assert_eq!(store.get_link(new), Some(Link::point(new)));
    assert_eq!(
        store.get_link(source_usage),
        Some(Link::new(source_usage, new, other))
    );
    assert_eq!(
        store.get_link(target_usage),
        Some(Link::new(target_usage, other, new))
    );
    assert!(!store.has_usages(old));
    assert_eq!(store.count(), 4);
    Ok(())
}

fn deletion_helpers_preserve_errors_and_self_rebase(
    mut store: impl Doublets<u32>,
) -> Result<(), Error<u32>> {
    let point = store.create_point()?;
    assert!(matches!(store.delete(99), Err(Error::NotExists(99))));
    assert!(matches!(store.delete_by([99]), Err(Error::NotExists(99))));
    assert!(matches!(
        store.rebase_and_delete(99, point),
        Err(Error::NotExists(99))
    ));
    assert_eq!(store.rebase_and_delete(point, point)?, point);
    assert_eq!(store.get_link(point), Some(Link::point(point)));
    assert_eq!(store.count(), 1);
    Ok(())
}

fn cascading_delete_returns_the_requested_index(
    store: impl Doublets<u32>,
) -> Result<(), Error<u32>> {
    let mut store = store.with_automatic_uniqueness_and_usages_resolution();
    let root = store.create_point()?;
    let other = store.create_point()?;
    let usage = store.create_link(root, other)?;
    let nested = store.create_link(usage, usage)?;

    assert_eq!(store.delete(root)?, root);
    for index in [root, usage, nested] {
        assert_eq!(store.get_link(index), None);
    }
    assert_eq!(store.get_link(other), Some(Link::point(other)));
    assert_eq!(store.count(), 1);
    Ok(())
}

macro_rules! test_stores {
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

                #[test]
                fn automatic_unit() -> Result<(), Error<u32>> {
                    $test(unit::Store::<u32, _>::new(Global::new())?
                        .with_automatic_uniqueness_and_usages_resolution())
                }

                #[test]
                fn automatic_split() -> Result<(), Error<u32>> {
                    $test(split::Store::<u32, _, _>::new(Global::new(), Global::new())?
                        .with_automatic_uniqueness_and_usages_resolution())
                }
            }
        )+
    };
}

test_stores!(
    delete_returns_the_former_index,
    delete_by_returns_the_former_index,
    delete_returns_an_uninitialized_links_index,
    rebase_and_delete_returns_new_without_usages,
    rebase_and_delete_returns_new_with_usages,
    deletion_helpers_preserve_errors_and_self_rebase,
);

#[test]
fn unit_cascading_delete_returns_the_requested_index() -> Result<(), Error<u32>> {
    cascading_delete_returns_the_requested_index(unit::Store::<u32, _>::new(Global::new())?)
}

#[test]
fn split_cascading_delete_returns_the_requested_index() -> Result<(), Error<u32>> {
    cascading_delete_returns_the_requested_index(split::Store::<u32, _, _>::new(
        Global::new(),
        Global::new(),
    )?)
}
