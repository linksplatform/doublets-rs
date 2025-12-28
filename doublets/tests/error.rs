use doublets::{Doublet, Error, Link};
use std::{error::Error as StdError, io};

#[test]
fn error_not_exists() {
    let err = Error::<usize>::NotExists(42);
    let display = format!("{}", err);
    assert!(display.contains("42"));
    assert!(display.contains("does not exist"));
}

#[test]
fn error_has_usages() {
    let links = vec![Link::<usize>::new(1, 2, 3), Link::<usize>::new(2, 3, 4)];
    let err = Error::<usize>::HasUsages(links);
    let display = format!("{}", err);
    assert!(display.contains("dependencies"));
}

#[test]
fn error_already_exists() {
    let doublet = Doublet::<usize>::new(1, 2);
    let err = Error::<usize>::AlreadyExists(doublet);
    let display = format!("{}", err);
    assert!(display.contains("already exists"));
    assert!(display.contains("1->2"));
}

#[test]
fn error_limit_reached() {
    let err = Error::<usize>::LimitReached(1000);
    let display = format!("{}", err);
    assert!(display.contains("1000"));
    assert!(display.contains("limit"));
}

#[test]
fn error_from_io_error() {
    let io_err = io::Error::new(io::ErrorKind::OutOfMemory, "out of memory");
    let err: Error<usize> = io_err.into();
    let display = format!("{}", err);
    assert!(display.contains("allocate memory"));
}

#[test]
fn error_other() {
    let other_err: Box<dyn StdError + Send + Sync> = "custom error".into();
    let err = Error::<usize>::Other(other_err);
    let display = format!("{}", err);
    assert!(display.contains("internal error"));
}

#[test]
fn error_debug() {
    let err = Error::<usize>::NotExists(42);
    let debug = format!("{:?}", err);
    assert!(debug.contains("NotExists"));
    assert!(debug.contains("42"));
}

#[test]
fn error_with_different_types() {
    // Test with u8
    let err_u8 = Error::<u8>::NotExists(42);
    assert!(format!("{}", err_u8).contains("42"));

    // Test with u32
    let err_u32 = Error::<u32>::NotExists(42);
    assert!(format!("{}", err_u32).contains("42"));

    // Test with u64
    let err_u64 = Error::<u64>::NotExists(42);
    assert!(format!("{}", err_u64).contains("42"));
}
