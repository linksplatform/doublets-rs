#![cfg(all(target_os = "linux", not(miri)))]

use doublets::{split, unit, Doublets, Link};
use mem::FileMapped;
use std::{fs::File, os::unix::fs::MetadataExt, path::Path};

const RESERVE_STEP: u64 = 1 << 20;
const LINKS: u32 = 1_000;

fn assert_sparse(path: &Path, reserved_bytes: u64) {
    let file = File::open(path).unwrap();
    file.sync_all().unwrap();
    let metadata = file.metadata().unwrap();
    let allocated_bytes = metadata.blocks() * 512;
    assert_eq!(metadata.len(), reserved_bytes);
    assert!(
        allocated_bytes < 1024 * 1024,
        "allocated {allocated_bytes} bytes for a {reserved_bytes}-byte reservation"
    );
}

fn write_links(mut store: impl Doublets<u32>) {
    for index in 1..=LINKS {
        assert_eq!(store.create_point().unwrap(), index);
    }
}

fn check_links(store: impl Doublets<u32>) {
    assert_eq!(store.count(), LINKS);
    for index in 1..=LINKS {
        assert_eq!(store.get_link(index), Some(Link::point(index)));
        assert_eq!(store.search(index, index), Some(index));
    }
}

#[test]
fn unit_file_reservation_stays_sparse() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("links");
    let open = || unit::Store::<u32, _>::new(FileMapped::from_path(&path).unwrap()).unwrap();

    write_links(open());
    assert_sparse(&path, RESERVE_STEP * 32);
    check_links(open());
}

#[test]
fn split_file_reservations_stay_sparse() {
    let dir = tempfile::tempdir().unwrap();
    let data = dir.path().join("data");
    let index = dir.path().join("index");
    let open = || {
        split::Store::<u32, _, _>::new(
            FileMapped::from_path(&data).unwrap(),
            FileMapped::from_path(&index).unwrap(),
        )
        .unwrap()
    };

    write_links(open());
    assert_sparse(&data, RESERVE_STEP * 8);
    assert_sparse(&index, RESERVE_STEP * 32);
    check_links(open());
}
