//! Reproduction of the index corruption described in issue #57 and referenced by #58.
//!
//! Creating several links with the same `(source, target)` pair inserts duplicate
//! entries into the size-balanced trees that index sources and targets, and the next
//! `delete` panics inside `platform-trees` with `attempt to subtract with overflow`.
//!
//! Run with:
//!
//! ```console
//! $ rustc --edition 2021 -L target/debug/deps --extern doublets=$(ls target/debug/libdoublets-*.rlib | head -1) \
//!       experiments/issue-58/duplicate_corruption.rs -o /tmp/duplicate_corruption && /tmp/duplicate_corruption
//! ```
//!
//! The supported version of this program lives in `doublets/examples/uniqueness.rs`,
//! which shows the same sequence surviving a uniqueness-resolving stack.
use doublets::{mem, unit, Doublets};

fn main() {
    let mut store = unit::Store::<usize, _>::new(mem::Global::new()).unwrap();
    let a = store.create_point().unwrap();
    let b = store.create_point().unwrap();
    let made: Vec<_> = (0..8).map(|_| store.create_link(a, b).unwrap()).collect();
    println!("created: {made:?}");
    for id in made {
        println!("deleting {id}");
        store.delete(id).unwrap();
    }
    println!("done");
}
