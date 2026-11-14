//! Registry consumer check for issue 18; run via verify.sh.
use doublets::{mem::Global, split, unit, Doublets, DoubletsExt, Error, Link};
use std::{error::Error as _, io};

fn verify(name: &str, mut store: impl Doublets<usize>) {
    let first = store.create_point().unwrap();
    let second = store.create_point().unwrap();
    let edge = store.create_link(first, second).unwrap();
    let expected = vec![
        Link::point(first),
        Link::point(second),
        Link::new(edge, first, second),
    ];
    assert_eq!(store.iter().collect::<Vec<_>>(), expected);
    #[cfg(feature = "full")]
    assert_eq!(store.iter_small().collect::<Vec<_>>(), expected);

    // Issue 18 failed while deriving `thiserror::Error`; exercise the derived impls.
    let missing = edge + 100;
    let error = store.delete(missing).unwrap_err();
    assert!(matches!(error, Error::NotExists(index) if index == missing));
    assert_eq!(error.to_string(), format!("link {missing} does not exist."));
    assert!(error.source().is_none());

    store.update(edge, second, first).unwrap();
    assert_eq!(store.get_link(edge), Some(Link::new(edge, second, first)));
    store.delete_all().unwrap();
    assert_eq!(store.count(), 0);
    println!("{name}: registry dependency CRUD, iterators and errors passed");
}

fn main() {
    // The original failure was a `#[from]` I/O source; its source chain must work.
    let alloc: Error<usize> = io::Error::other("probe").into();
    assert!(matches!(alloc, Error::AllocFailed(_)));
    assert!(alloc.to_string().starts_with("unable to allocate memory"));
    assert!(alloc.source().is_some());

    verify("unit", unit::Store::<usize, _>::new(Global::new()).unwrap());
    verify(
        "split",
        split::Store::<usize, _, _>::new(Global::new(), Global::new()).unwrap(),
    );
}
