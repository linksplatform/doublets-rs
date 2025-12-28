use doublets::{
    mem::LinksHeader,
    parts::{DataPart, IndexPart, LinkPart},
    split, unit, Doublets, Error, Links,
};
use mem::Global;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

// Tests for LinksHeader

#[test]
fn links_header_default() {
    let header = LinksHeader::<usize>::default();
    assert_eq!(header.allocated, 0);
    assert_eq!(header.reserved, 0);
    assert_eq!(header.free, 0);
    assert_eq!(header.first_free, 0);
    assert_eq!(header.root_as_source, 0);
    assert_eq!(header.root_as_target, 0);
    assert_eq!(header.last_free, 0);
}

#[test]
fn links_header_eq() {
    let header1 = LinksHeader::<usize>::default();
    let header2 = LinksHeader::<usize>::default();
    assert_eq!(header1, header2);
}

#[test]
fn links_header_clone() {
    let header = LinksHeader::<usize>::default();
    let cloned = header.clone();
    assert_eq!(header, cloned);
}

#[test]
fn links_header_debug() {
    let header = LinksHeader::<usize>::default();
    let debug_str = format!("{:?}", header);
    assert!(debug_str.contains("LinksHeader"));
}

// Tests for LinkPart - only testing publicly accessible traits

#[test]
fn link_part_default_exists() {
    // Just test that Default trait works
    let _part = LinkPart::<usize>::default();
}

#[test]
fn link_part_eq() {
    let part1 = LinkPart::<usize>::default();
    let part2 = LinkPart::<usize>::default();
    assert_eq!(part1, part2);
}

#[test]
fn link_part_clone() {
    let part = LinkPart::<usize>::default();
    let cloned = part.clone();
    assert_eq!(part, cloned);
}

#[test]
fn link_part_hash() {
    let part1 = LinkPart::<usize>::default();
    let part2 = LinkPart::<usize>::default();

    let mut hasher1 = DefaultHasher::new();
    let mut hasher2 = DefaultHasher::new();

    part1.hash(&mut hasher1);
    part2.hash(&mut hasher2);

    assert_eq!(hasher1.finish(), hasher2.finish());
}

#[test]
fn link_part_debug() {
    let part = LinkPart::<usize>::default();
    let debug_str = format!("{:?}", part);
    assert!(debug_str.contains("LinkPart"));
}

// Tests for DataPart - only testing publicly accessible traits

#[test]
fn data_part_default_exists() {
    // Just test that Default trait works
    let _part = DataPart::<usize>::default();
}

#[test]
fn data_part_eq() {
    let part1 = DataPart::<usize>::default();
    let part2 = DataPart::<usize>::default();
    assert_eq!(part1, part2);
}

#[test]
fn data_part_clone() {
    let part = DataPart::<usize>::default();
    let cloned = part.clone();
    assert_eq!(part, cloned);
}

#[test]
fn data_part_hash() {
    let part1 = DataPart::<usize>::default();
    let part2 = DataPart::<usize>::default();

    let mut hasher1 = DefaultHasher::new();
    let mut hasher2 = DefaultHasher::new();

    part1.hash(&mut hasher1);
    part2.hash(&mut hasher2);

    assert_eq!(hasher1.finish(), hasher2.finish());
}

#[test]
fn data_part_debug() {
    let part = DataPart::<usize>::default();
    let debug_str = format!("{:?}", part);
    assert!(debug_str.contains("DataPart"));
}

// Tests for IndexPart - only testing publicly accessible traits

#[test]
fn index_part_default_exists() {
    // Just test that Default trait works
    let _part = IndexPart::<usize>::default();
}

#[test]
fn index_part_eq() {
    let part1 = IndexPart::<usize>::default();
    let part2 = IndexPart::<usize>::default();
    assert_eq!(part1, part2);
}

#[test]
fn index_part_clone() {
    let part = IndexPart::<usize>::default();
    let cloned = part.clone();
    assert_eq!(part, cloned);
}

#[test]
fn index_part_hash() {
    let part1 = IndexPart::<usize>::default();
    let part2 = IndexPart::<usize>::default();

    let mut hasher1 = DefaultHasher::new();
    let mut hasher2 = DefaultHasher::new();

    part1.hash(&mut hasher1);
    part2.hash(&mut hasher2);

    assert_eq!(hasher1.finish(), hasher2.finish());
}

#[test]
fn index_part_debug() {
    let part = IndexPart::<usize>::default();
    let debug_str = format!("{:?}", part);
    assert!(debug_str.contains("IndexPart"));
}

// Tests for Unit Store

#[test]
fn unit_store_new() -> Result<(), Error<usize>> {
    let store = unit::Store::<usize, _>::new(Global::new())?;
    assert_eq!(store.count(), 0);
    Ok(())
}

#[test]
fn unit_store_crud() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    // Create
    let a = store.create()?;
    assert!(a > 0);

    // Read
    let link = store.get_link(a);
    assert!(link.is_some());

    // Update
    let b = store.create()?;
    store.update(a, a, b)?;
    let link = store.get_link(a).unwrap();
    assert_eq!(link.target, b);

    // Delete
    store.delete(a)?;
    assert!(store.get_link(a).is_none());

    Ok(())
}

#[test]
fn unit_store_constants() -> Result<(), Error<usize>> {
    let store = unit::Store::<usize, _>::new(Global::new())?;
    let constants = Links::constants(&store);

    // Check that any is set properly
    assert!(constants.any > 0);

    Ok(())
}

// Tests for Split Store

#[test]
fn split_store_new() -> Result<(), Error<usize>> {
    let store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;
    assert_eq!(store.count(), 0);
    Ok(())
}

#[test]
fn split_store_crud() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    // Create
    let a = store.create()?;
    assert!(a > 0);

    // Read
    let link = store.get_link(a);
    assert!(link.is_some());

    // Update
    let b = store.create()?;
    store.update(a, a, b)?;
    let link = store.get_link(a).unwrap();
    assert_eq!(link.target, b);

    // Delete
    store.delete(a)?;
    assert!(store.get_link(a).is_none());

    Ok(())
}

#[test]
fn split_store_constants() -> Result<(), Error<usize>> {
    let store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;
    let constants = Links::constants(&store);

    // Check that any is set properly
    assert!(constants.any > 0);

    Ok(())
}

// Tests comparing unit and split store behavior

#[test]
fn unit_split_equivalence() -> Result<(), Error<usize>> {
    let mut unit_store = unit::Store::<usize, _>::new(Global::new())?;
    let mut split_store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    // Create same structure in both
    let u_a = unit_store.create_point()?;
    let u_b = unit_store.create_point()?;
    let u_c = unit_store.create_link(u_a, u_b)?;

    let s_a = split_store.create_point()?;
    let s_b = split_store.create_point()?;
    let s_c = split_store.create_link(s_a, s_b)?;

    // Should have same count
    assert_eq!(unit_store.count(), split_store.count());

    // Links should have same structure
    let u_link = unit_store.get_link(u_c).unwrap();
    let s_link = split_store.get_link(s_c).unwrap();

    assert_eq!(u_link.source, s_link.source);
    assert_eq!(u_link.target, s_link.target);

    Ok(())
}

// Tests for different link types
// Note: u8 is not tested because the store requires minimum allocation size
// that may not fit in u8 range on all platforms

#[test]
fn unit_store_u32() -> Result<(), Error<u32>> {
    let mut store = unit::Store::<u32, _>::new(Global::new())?;
    let a = store.create_point()?;
    assert_eq!(store.count(), 1);
    assert!(a > 0);
    Ok(())
}

#[test]
fn unit_store_u64() -> Result<(), Error<u64>> {
    let mut store = unit::Store::<u64, _>::new(Global::new())?;
    let a = store.create_point()?;
    assert_eq!(store.count(), 1);
    assert!(a > 0);
    Ok(())
}

// Tests for large number of links

#[test]
fn unit_store_many_links() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    for _ in 0..100 {
        store.create_point()?;
    }

    assert_eq!(store.count(), 100);

    Ok(())
}

#[test]
fn split_store_many_links() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    for _ in 0..100 {
        store.create_point()?;
    }

    assert_eq!(store.count(), 100);

    Ok(())
}
