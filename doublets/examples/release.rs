//! Choose release-only deletion or opt into endpoint reset on either memory layout.

use data::Flow;
use doublets::{
    decorators::{CheckedRelease, DecoratorsExt},
    mem::Release,
    split, unit, Doublets, Error, Link,
};
use mem::Global;

fn demonstrate(mut store: impl Release<usize>) -> Result<(), Error<usize>> {
    let root = store.create_point()?;
    store.update(root, 0, 0)?;
    // No second reset/update: just checked slot release.
    assert_eq!(store.release(root)?, root);

    let root = store.create_point()?;
    let usage = store.create_link(root, 99)?;
    let mut release = CheckedRelease::new(&mut store);
    assert!(matches!(
        release.delete(root),
        Err(Error::NonNullContents(_))
    ));

    // This layer explicitly opts into automatic reset and reports both events.
    let mut resetting = release.with_non_null_contents_deletion_resolution();
    let mut events = Vec::new();
    resetting.delete_with(root, |before, after| {
        events.push((before, after));
        Flow::Continue
    })?;
    assert_eq!(
        events,
        [
            (Link::point(root), Link::new(root, 0, 0)),
            (Link::new(root, 0, 0), Link::nothing()),
        ]
    );
    // Incoming references survive release and free-list reuse on both layouts.
    assert_eq!(resetting.search(root, 99), Some(usage));
    assert_eq!(resetting.create()?, root);
    assert_eq!(resetting.search(root, 99), Some(usage));
    Ok(())
}

fn main() -> Result<(), Error<usize>> {
    demonstrate(unit::Store::<usize, _>::new(Global::new())?)?;
    demonstrate(split::Store::<usize, _, _>::new(
        Global::new(),
        Global::new(),
    )?)
}
