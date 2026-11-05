//! Direct-constructor coverage for the six original decorator exports (issue #83).
//! These APIs belong to the main library and require no optional features.

use data::Flow;
use doublets::{
    decorators::{
        CascadeUniquenessAndUsagesResolver, CascadeUsagesResolver,
        NonNullContentsLinkDeletionResolver, UniquenessResolver, UniquenessValidator,
        UsagesValidator,
    },
    split, unit, Doublets, Error, Link,
};
use mem::Global;

fn uniqueness_resolver_reuses_links(store: impl Doublets<u32>) -> Result<(), Error<u32>> {
    let mut links = UniquenessResolver::new(store);
    let point = links.create_point()?;
    assert_eq!(links.create_link(point, point)?, point);
    assert_eq!(links.count(), 1);
    assert_eq!(links.into_inner().get_link(point), Some(Link::point(point)));
    Ok(())
}

fn uniqueness_validator_rejects_duplicates(store: impl Doublets<u32>) -> Result<(), Error<u32>> {
    let mut links = UniquenessValidator::new(store);
    let point = links.create_point()?;
    assert!(matches!(
        links.create_link(point, point),
        Err(Error::AlreadyExists(_))
    ));
    assert_eq!(links.get_link(point), Some(Link::point(point)));
    Ok(())
}

fn usages_validator_protects_referenced_links(store: impl Doublets<u32>) -> Result<(), Error<u32>> {
    let mut links = UsagesValidator::new(store);
    let point = links.create_point()?;
    let usage = links.create_link(point, 0)?;
    assert!(matches!(links.delete(point), Err(Error::HasUsages(_))));
    assert!(matches!(
        links.update(point, 0, 0),
        Err(Error::HasUsages(_))
    ));
    assert_eq!(links.get_link(usage), Some(Link::new(usage, point, 0)));
    Ok(())
}

fn cascade_usages_resolver_composes_with_deletion(
    store: impl Doublets<u32>,
) -> Result<(), Error<u32>> {
    let mut links = NonNullContentsLinkDeletionResolver::new(CascadeUsagesResolver::new(store));
    let point = links.create_point()?;
    let usage = links.create_link(point, 0)?;
    let nested = links.create_link(usage, usage)?;
    assert_eq!(links.delete(point)?, point);
    assert_eq!(links.count(), 0);
    for index in [point, usage, nested] {
        assert_eq!(links.get_link(index), None);
    }
    Ok(())
}

fn non_null_deletion_resets_contents_before_deleting(
    store: impl Doublets<u32>,
) -> Result<(), Error<u32>> {
    let mut links = NonNullContentsLinkDeletionResolver::new(store);
    let point = links.create_point()?;
    let mut changes = Vec::new();
    assert_eq!(
        links.delete_by_with([point], |before, after| {
            changes.push((before, after));
            Flow::Continue
        })?,
        Flow::Continue
    );
    let reset = Link::new(point, 0, 0);
    assert_eq!(
        changes,
        vec![
            (Link::point(point), reset.clone()),
            (reset, Link::nothing())
        ]
    );
    assert_eq!(links.count(), 0);
    Ok(())
}

fn cascade_uniqueness_resolver_redirects_usages(
    store: impl Doublets<u32>,
) -> Result<(), Error<u32>> {
    let mut links = CascadeUniquenessAndUsagesResolver::new(store);
    let a = links.create_point()?;
    assert_eq!(links.create_link(a, a)?, a);
    let b = links.create_point()?;
    let survivor = links.create_link(a, b)?;
    let redundant = links.create_link(b, a)?;
    let usage = links.create_link(redundant, b)?;
    assert_eq!(links.update(redundant, a, b)?, survivor);
    assert_eq!(links.get_link(redundant), None);
    assert_eq!(links.get_link(usage), Some(Link::new(usage, survivor, b)));
    assert_eq!(links.count(), 4);
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
            }
        )+
    };
}

test_stores!(
    uniqueness_resolver_reuses_links,
    uniqueness_validator_rejects_duplicates,
    usages_validator_protects_referenced_links,
    cascade_usages_resolver_composes_with_deletion,
    non_null_deletion_resets_contents_before_deleting,
    cascade_uniqueness_resolver_redirects_usages,
);
