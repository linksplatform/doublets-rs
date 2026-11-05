use super::*;
use crate::mem::tests::check_links;
use mem::{FileMapped, Global};

#[test]
fn deletion_of_reset_link_does_not_update_again() {
    let mut store = Store::<usize, _, _>::new(Global::new(), Global::new()).unwrap();
    let index = store.create_point().unwrap();
    store.update(index, 0, 0).unwrap();
    let updates = store.update_calls;
    assert_eq!(store.delete(index).unwrap(), index);
    assert_eq!(store.update_calls, updates, "release must not reset again");
}

fn check_growth(
    data: impl RawMem<Item = DataPart<usize>>,
    index: impl RawMem<Item = IndexPart<usize>>,
) {
    let mut store = Store::<usize, _, _>::new(data, index).unwrap();
    assert_eq!(store.data_ptr.len(), store.data_step);
    assert_eq!(store.index_ptr.len(), store.data_ptr.len());
    assert_eq!(store.data_ptr.len(), store.data_mem.allocated().len());
    assert_eq!(store.index_ptr.len(), store.index_mem.allocated().len());
    assert_eq!(store.get_header().reserved, store.data_ptr.len() - 1);
    // Keep the growth regression small without changing production capacities.
    let data = NonNull::from(crate::mem::resize_mem(&mut store.data_mem, 16).unwrap());
    let index = NonNull::from(crate::mem::resize_mem(&mut store.index_mem, 16).unwrap());
    store.update_mem(data, index);
    store.data_step = 16;
    store.index_step = 16;
    store.mut_header().reserved = 15;
    for index in 1..=50 {
        assert_eq!(store.create_point().unwrap(), index);
        assert_eq!(store.data_ptr.len(), store.data_mem.allocated().len());
        assert_eq!(store.index_ptr.len(), store.index_mem.allocated().len());
        check_links(&store, index);
    }
    assert_eq!(store.data_ptr.len(), 64);
    assert_eq!(store.index_ptr.len(), 64);
}

#[test]
fn global_initial_capacity_and_repeated_growth_preserve_links() {
    check_growth(Global::new(), Global::new());
}

#[test]
fn file_mapped_initial_capacity_and_repeated_growth_preserve_links() {
    let directory = tempfile::tempdir().unwrap();
    let data_path = directory.path().join("data");
    let index_path = directory.path().join("index");
    check_growth(
        FileMapped::from_path(&data_path).unwrap(),
        FileMapped::from_path(&index_path).unwrap(),
    );
    let store = Store::<usize, _, _>::new(
        FileMapped::from_path(&data_path).unwrap(),
        FileMapped::from_path(&index_path).unwrap(),
    )
    .unwrap();
    check_links(&store, 50);
}

#[test]
fn minimal_release_and_resolver_do_only_required_updates() {
    use crate::{
        decorators::{CheckedRelease, DecoratorsExt},
        mem::Release,
    };

    let mut store = Store::<usize, _, _>::new(Global::new(), Global::new()).unwrap();
    let index = store.create_point().unwrap();
    let updates = store.update_calls;
    assert!(matches!(
        store.release(index),
        Err(LinksError::NonNullContents(_))
    ));
    assert_eq!(store.update_calls, updates);
    store.update(index, 0, 0).unwrap();
    let updates = store.update_calls;
    store.release(index).unwrap();
    assert_eq!(store.update_calls, updates);

    let index = store.create().unwrap();
    // SAFETY: newly created live links have zero endpoints and consistent indexes.
    unsafe { store.release_unchecked(index, &mut |_, _| Flow::Continue) };
    assert_eq!(store.update_calls, updates);

    let index = store.create_point().unwrap();
    let updates = store.update_calls;
    CheckedRelease::new(&mut store)
        .with_non_null_contents_deletion_resolution()
        .delete(index)
        .unwrap();
    assert_eq!(store.update_calls, updates + 1);

    let index = store.create().unwrap();
    let updates = store.update_calls;
    CheckedRelease::new(&mut store)
        .with_non_null_contents_deletion_resolution()
        .delete(index)
        .unwrap();
    assert_eq!(store.update_calls, updates);
}

#[test]
fn unchecked_mutation_omits_validation_work() {
    use crate::mem::{Release, UncheckedMutation};
    use std::sync::atomic::Ordering::Relaxed;

    let mut store = Store::<usize, _, _>::new(Global::new(), Global::new()).unwrap();
    let a = store.create_point().unwrap();
    let b = store.create().unwrap();
    let c = store.create().unwrap();
    let noop = &mut |_, _| Flow::Continue;

    // The safe route validates the address exactly once.
    let (lookups, updates) = (store.link_lookups.load(Relaxed), store.update_calls);
    store.update(b, a, c).unwrap();
    assert_eq!(store.link_lookups.load(Relaxed), lookups + 1);
    assert_eq!(store.update_calls, updates + 1);

    // A rejected safe call validates once and does not reach the primitive.
    let (lookups, updates) = (store.link_lookups.load(Relaxed), store.update_calls);
    assert!(matches!(
        store.update(99, a, a),
        Err(LinksError::NotExists(99))
    ));
    assert!(matches!(store.delete(99), Err(LinksError::NotExists(99))));
    assert_eq!(store.link_lookups.load(Relaxed), lookups + 2);
    assert_eq!(store.update_calls, updates);

    // Omitting validation omits its work, but not the mutation itself.
    let (lookups, updates) = (store.link_lookups.load(Relaxed), store.update_calls);
    // SAFETY: `b` is live and the indexes were maintained by the store API.
    unsafe { store.update_unchecked(b, a, b, noop) };
    assert_eq!(store.link_lookups.load(Relaxed), lookups);
    assert_eq!(store.update_calls, updates + 1);
    assert_eq!(store.search(a, b), Some(b));

    // Checked deletion: one lookup, one reset of the non-null endpoints.
    let (lookups, updates) = (store.link_lookups.load(Relaxed), store.update_calls);
    assert_eq!(store.delete(b).unwrap(), b);
    assert_eq!(store.link_lookups.load(Relaxed), lookups + 1);
    assert_eq!(store.update_calls, updates + 1);

    // Unchecked deletion composed from the primitives does no lookups.
    store.update(c, c, a).unwrap();
    let (lookups, updates) = (store.link_lookups.load(Relaxed), store.update_calls);
    // SAFETY: `c` is live; the reset detaches its outgoing index memberships.
    unsafe {
        store.update_unchecked(c, 0, 0, noop);
        store.release_unchecked(c, noop);
    }
    assert_eq!(store.link_lookups.load(Relaxed), lookups);
    assert_eq!(store.update_calls, updates + 1);
    assert!(!store.exist(c));
    check_links(&store, 1);
}
