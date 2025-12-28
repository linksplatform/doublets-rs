use doublets::Doublet;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

#[test]
fn doublet_new() {
    let doublet = Doublet::<usize>::new(1, 2);
    assert_eq!(doublet.source, 1);
    assert_eq!(doublet.target, 2);
}

#[test]
fn doublet_display() {
    let doublet = Doublet::<usize>::new(1, 2);
    let display_str = format!("{}", doublet);
    assert_eq!(display_str, "1->2");
}

#[test]
fn doublet_display_large_values() {
    let doublet = Doublet::<usize>::new(12345, 67890);
    let display_str = format!("{}", doublet);
    assert_eq!(display_str, "12345->67890");
}

#[test]
fn doublet_debug() {
    let doublet = Doublet::<usize>::new(1, 2);
    let debug_str = format!("{:?}", doublet);
    assert!(debug_str.contains('1'));
    assert!(debug_str.contains('2'));
}

#[test]
fn doublet_eq() {
    let doublet1 = Doublet::<usize>::new(1, 2);
    let doublet2 = Doublet::<usize>::new(1, 2);
    let doublet3 = Doublet::<usize>::new(1, 3);
    let doublet4 = Doublet::<usize>::new(2, 2);

    assert_eq!(doublet1, doublet2);
    assert_ne!(doublet1, doublet3);
    assert_ne!(doublet1, doublet4);
}

#[test]
fn doublet_clone() {
    let doublet = Doublet::<usize>::new(1, 2);
    let cloned = doublet.clone();
    assert_eq!(doublet, cloned);
}

#[test]
fn doublet_hash() {
    let doublet1 = Doublet::<usize>::new(1, 2);
    let doublet2 = Doublet::<usize>::new(1, 2);
    let doublet3 = Doublet::<usize>::new(2, 1);

    let mut hasher1 = DefaultHasher::new();
    let mut hasher2 = DefaultHasher::new();
    let mut hasher3 = DefaultHasher::new();

    doublet1.hash(&mut hasher1);
    doublet2.hash(&mut hasher2);
    doublet3.hash(&mut hasher3);

    assert_eq!(hasher1.finish(), hasher2.finish());
    assert_ne!(hasher1.finish(), hasher3.finish());
}

#[test]
fn doublet_with_different_types() {
    // Test with u8
    let doublet_u8 = Doublet::<u8>::new(1, 2);
    assert_eq!(doublet_u8.source, 1u8);
    assert_eq!(doublet_u8.target, 2u8);

    // Test with u32
    let doublet_u32 = Doublet::<u32>::new(1, 2);
    assert_eq!(doublet_u32.source, 1u32);
    assert_eq!(doublet_u32.target, 2u32);

    // Test with u64
    let doublet_u64 = Doublet::<u64>::new(1, 2);
    assert_eq!(doublet_u64.source, 1u64);
    assert_eq!(doublet_u64.target, 2u64);
}

#[test]
fn doublet_self_reference() {
    // A doublet can reference itself
    let doublet = Doublet::<usize>::new(5, 5);
    assert_eq!(doublet.source, 5);
    assert_eq!(doublet.target, 5);
    assert_eq!(format!("{}", doublet), "5->5");
}

#[test]
fn doublet_large_values() {
    let max = usize::MAX;
    let doublet = Doublet::<usize>::new(max, max - 1);
    assert_eq!(doublet.source, max);
    assert_eq!(doublet.target, max - 1);
}
