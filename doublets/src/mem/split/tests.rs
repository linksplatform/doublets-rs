use super::*;
use crate::mem::tests::check_links;
use mem::{FileMapped, Global};

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
