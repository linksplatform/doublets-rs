use super::*;
use crate::mem::tests::check_links;
use mem::{FileMapped, Global};

fn check_growth(memory: impl RawMem<Item = LinkPart<usize>>) {
    let mut store = Store::<usize, _>::new(memory).unwrap();
    assert_eq!(store.mem_ptr.len(), store.reserve_step);
    assert_eq!(store.mem_ptr.len(), store.mem.allocated().len());
    assert_eq!(store.get_header().reserved, store.mem_ptr.len() - 1);
    // Shrink the empty store so three subsequent growth events need only 50 links.
    let memory = NonNull::from(crate::mem::resize_mem(&mut store.mem, 16).unwrap());
    store.update_mem(memory);
    store.reserve_step = 16;
    store.mut_header().reserved = 15;
    for index in 1..=50 {
        assert_eq!(store.create_point().unwrap(), index);
        assert_eq!(store.mem_ptr.len(), store.mem.allocated().len());
        check_links(&store, index);
    }
    assert_eq!(store.mem_ptr.len(), 64);
}

#[test]
fn global_initial_capacity_and_repeated_growth_preserve_links() {
    check_growth(Global::new());
}

#[test]
fn file_mapped_initial_capacity_and_repeated_growth_preserve_links() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("links");
    check_growth(FileMapped::from_path(&path).unwrap());
    let store = Store::<usize, _>::new(FileMapped::from_path(&path).unwrap()).unwrap();
    check_links(&store, 50);
}
