use data::ToQuery;
use doublets::Link;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

#[test]
fn link_to_query() {
    let link = Link::<usize>::new(1, 2, 3);
    assert_eq!([1, 2, 3], &link.to_query()[..]);
}

#[test]
fn link_new() {
    let link = Link::<usize>::new(1, 2, 3);
    assert_eq!(link.index, 1);
    assert_eq!(link.source, 2);
    assert_eq!(link.target, 3);
}

#[test]
fn link_nothing() {
    let link = Link::<usize>::nothing();
    assert_eq!(link.index, 0);
    assert_eq!(link.source, 0);
    assert_eq!(link.target, 0);
}

#[test]
fn link_point() {
    let link = Link::<usize>::point(42);
    assert_eq!(link.index, 42);
    assert_eq!(link.source, 42);
    assert_eq!(link.target, 42);
}

#[test]
fn link_from_slice() {
    let slice = [1usize, 2, 3];
    let link = Link::from_slice(&slice);
    assert_eq!(link.index, 1);
    assert_eq!(link.source, 2);
    assert_eq!(link.target, 3);
}

#[test]
#[should_panic]
fn link_from_slice_too_small() {
    let slice = [1usize, 2];
    let _ = Link::from_slice(&slice);
}

#[test]
fn link_is_null() {
    let null_link = Link::<usize>::point(0);
    assert!(null_link.is_null());

    let non_null = Link::<usize>::point(1);
    assert!(!non_null.is_null());

    let partial = Link::<usize>::new(0, 1, 0);
    assert!(!partial.is_null());
}

#[test]
fn link_is_full() {
    // Full link: index == source == target
    let full = Link::<usize>::point(5);
    assert!(full.is_full());

    // Not full: source differs
    let not_full = Link::<usize>::new(5, 3, 5);
    assert!(!not_full.is_full());

    // Not full: target differs
    let not_full2 = Link::<usize>::new(5, 5, 3);
    assert!(!not_full2.is_full());

    // Not full: all different
    let all_different = Link::<usize>::new(1, 2, 3);
    assert!(!all_different.is_full());
}

#[test]
fn link_is_partial() {
    // Partial: index == source
    let partial_source = Link::<usize>::new(5, 5, 3);
    assert!(partial_source.is_partial());

    // Partial: index == target
    let partial_target = Link::<usize>::new(5, 3, 5);
    assert!(partial_target.is_partial());

    // Full is also partial (index == source and index == target)
    let full = Link::<usize>::point(5);
    assert!(full.is_partial());

    // Not partial: all different
    let not_partial = Link::<usize>::new(1, 2, 3);
    assert!(!not_partial.is_partial());
}

#[test]
fn link_as_slice() {
    let link = Link::<usize>::new(1, 2, 3);
    let slice = link.as_slice();
    assert_eq!(slice, &[1, 2, 3]);
}

#[test]
fn link_debug_format() {
    let link = Link::<usize>::new(1, 2, 3);
    let debug_str = format!("{:?}", link);
    assert_eq!(debug_str, "1: 2 3");
}

#[test]
fn link_default() {
    let link = Link::<usize>::default();
    assert_eq!(link.index, 0);
    assert_eq!(link.source, 0);
    assert_eq!(link.target, 0);
}

#[test]
fn link_eq() {
    let link1 = Link::<usize>::new(1, 2, 3);
    let link2 = Link::<usize>::new(1, 2, 3);
    let link3 = Link::<usize>::new(1, 2, 4);

    assert_eq!(link1, link2);
    assert_ne!(link1, link3);
}

#[test]
fn link_clone() {
    let link = Link::<usize>::new(1, 2, 3);
    let cloned = link.clone();
    assert_eq!(link, cloned);
}

#[test]
fn link_hash() {
    let link1 = Link::<usize>::new(1, 2, 3);
    let link2 = Link::<usize>::new(1, 2, 3);
    let link3 = Link::<usize>::new(1, 2, 4);

    let mut hasher1 = DefaultHasher::new();
    let mut hasher2 = DefaultHasher::new();
    let mut hasher3 = DefaultHasher::new();

    link1.hash(&mut hasher1);
    link2.hash(&mut hasher2);
    link3.hash(&mut hasher3);

    assert_eq!(hasher1.finish(), hasher2.finish());
    assert_ne!(hasher1.finish(), hasher3.finish());
}

#[test]
fn link_with_different_types() {
    // Test with u8
    let link_u8 = Link::<u8>::new(1, 2, 3);
    assert_eq!(link_u8.index, 1u8);

    // Test with u32
    let link_u32 = Link::<u32>::new(1, 2, 3);
    assert_eq!(link_u32.index, 1u32);

    // Test with u64
    let link_u64 = Link::<u64>::new(1, 2, 3);
    assert_eq!(link_u64.index, 1u64);
}

#[test]
fn link_large_values() {
    let max = usize::MAX;
    let link = Link::<usize>::new(max, max - 1, max - 2);
    assert_eq!(link.index, max);
    assert_eq!(link.source, max - 1);
    assert_eq!(link.target, max - 2);
}
