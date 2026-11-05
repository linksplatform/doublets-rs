//! Exercise two growth boundaries with unwrapped heap or file-mapped memory.
//! Run `cargo run -p doublets --release --example memory_growth -- unit`.
//! Backends: unit, split, unit-file, split-file. Each run creates a finite 2^21+1 links.

use doublets::{
    mem::{FileMapped, Global},
    split, unit, Doublets, Link,
};

const LINKS: usize = (1 << 21) + 1;

fn check(mut store: impl Doublets<usize>) {
    for index in 1..=LINKS {
        assert_eq!(store.create_point().unwrap(), index);
    }
    assert_eq!(store.count(), LINKS);
    for index in [1, 1_040_384, 1 << 20, 1 << 21, LINKS] {
        assert_eq!(store.get_link(index), Some(Link::point(index)));
        assert_eq!(store.search(index, index), Some(index));
    }
    println!("Created {LINKS} links; earlier links remain readable and searchable");
}

fn main() {
    let directory = tempfile::tempdir().unwrap();
    match std::env::args().nth(1).as_deref() {
        Some("unit") => check(unit::Store::<usize, _>::new(Global::new()).unwrap()),
        Some("split") => {
            check(split::Store::<usize, _, _>::new(Global::new(), Global::new()).unwrap());
        }
        Some("unit-file") => check(
            unit::Store::<usize, _>::new(
                FileMapped::from_path(directory.path().join("links")).unwrap(),
            )
            .unwrap(),
        ),
        Some("split-file") => check(
            split::Store::<usize, _, _>::new(
                FileMapped::from_path(directory.path().join("data")).unwrap(),
                FileMapped::from_path(directory.path().join("index")).unwrap(),
            )
            .unwrap(),
        ),
        _ => panic!("usage: memory_growth unit|split|unit-file|split-file"),
    }
}
