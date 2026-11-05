//! Bounded probe of self-reference counting while changing and deleting links.
//!
//! Compile against the library built by `cargo build -p doublets`:
//! rustc --edition=2021 experiments/issue-63/self-reference-probe.rs \
//!   -L dependency=target/debug/deps \
//!   --extern doublets=target/debug/libdoublets.rlib \
//!   -o /tmp/issue-63-self-reference-probe

use doublets::{mem::Global, split, unit, Doublets, Error, Link};

fn probe(mut store: impl Doublets<u32>) -> Result<(), Error<u32>> {
    let other = store.create_point()?;
    let value = store.create_point()?;
    let usage_target = store.create_point()?;
    let any = store.constants().any;

    for (source, target) in [(value, value), (value, other), (other, value)] {
        if store.get_link(value) != Some(Link::new(value, source, target)) {
            store.update(value, source, target)?;
        }
        println!(
            "value={:?}, source count={}, target count={}, overlap count={}",
            store.get_link(value),
            store.count_by([any, value, any]),
            store.count_by([any, any, value]),
            store.count_by([any, value, value]),
        );
        let usage = store.create_link(value, usage_target)?;
        println!(
            "usage={usage}, count_usages={:?}",
            store.count_usages(value)
        );
        store.delete_usages(value)?;
    }
    Ok(())
}

fn main() -> Result<(), Error<u32>> {
    println!("unit store:");
    probe(unit::Store::<u32, _>::new(Global::new())?)?;
    println!("split store:");
    probe(split::Store::<u32, _, _>::new(
        Global::new(),
        Global::new(),
    )?)
}
