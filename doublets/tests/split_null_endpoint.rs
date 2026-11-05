//! Split-store liveness must distinguish null endpoints from deleted slots.

use doublets::{split, Doublets, Error, Link};
use mem::Global;

#[test]
fn a_null_target_does_not_make_a_live_link_unused() -> Result<(), Error<u32>> {
    let mut store = split::Store::<u32, _, _>::new(Global::new(), Global::new())?;
    let point = store.create_point()?;
    let usage = store.create_link(point, 0)?;
    assert!(!store.is_unused(usage));
    assert_eq!(store.get_link(usage), Some(Link::new(usage, point, 0)));
    Ok(())
}

#[test]
fn null_endpoints_remain_live_while_free_slots_are_reused() -> Result<(), Error<u32>> {
    let mut store = split::Store::<u32, _, _>::new(Global::new(), Global::new())?;
    let point = store.create_point()?;
    let hole = store.create_point()?;
    let source_only = store.create_link(point, 0)?;
    let target_only = store.create_link(0, point)?;
    let empty = store.create()?;
    let tail = store.create_point()?;

    store.delete(hole)?;
    assert!(store.is_unused(hole));
    assert_eq!(store.get_link(hole), None);
    store.delete(tail)?;
    store.delete(empty)?;
    assert!(!store.is_unused(source_only));
    assert!(!store.is_unused(target_only));
    assert_eq!(
        store.get_link(source_only),
        Some(Link::new(source_only, point, 0))
    );
    assert_eq!(
        store.get_link(target_only),
        Some(Link::new(target_only, 0, point))
    );
    assert_eq!(store.count(), 3);

    assert_eq!(store.create_point()?, hole);
    assert_eq!(store.delete(target_only)?, target_only);
    assert_eq!(store.delete(source_only)?, source_only);
    assert_eq!(store.count(), 2);
    assert_eq!(store.get_link(point), Some(Link::point(point)));
    assert_eq!(store.get_link(hole), Some(Link::point(hole)));
    Ok(())
}
