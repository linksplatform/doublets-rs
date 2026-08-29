//! Duplicate doublets, with and without a uniqueness policy.
//!
//! A bare store happily creates the same `(source, target)` pair over and over. Each
//! copy adds another entry to the trees that index sources and targets, and deleting one
//! of them panics inside `platform-trees` — this is issue #57.
//!
//! Wrapping the same store in [`UniquenessResolver`] makes `create_link` return the
//! address of the link that already holds the pair, which is what a caller reaching for
//! `get_or_create` everywhere actually wants.
//!
//! ```console
//! $ cargo run -p doublets --example uniqueness
//! ```
//!
//! [`UniquenessResolver`]: doublets::decorators::UniquenessResolver

use doublets::{
    decorators::{DecoratorsExt, Resolve},
    mem, unit, Doublets, DoubletsExt, Error,
};

const DUPLICATES: usize = 8;

fn main() -> Result<(), Error<usize>> {
    without_a_policy()?;
    with_uniqueness_resolution()
}

/// What a bare store does: eight distinct links holding the same pair.
fn without_a_policy() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(mem::Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let created: Vec<_> = (0..DUPLICATES)
        .map(|_| store.create_link(a, b))
        .collect::<Result<_, _>>()?;

    println!("bare store:");
    println!("  create_link({a}, {b}) x{DUPLICATES} -> {created:?}");
    println!("  count = {}", store.count());
    println!("  deleting any of them now panics in `platform-trees` (issue #57)");
    println!();

    Ok(())
}

/// The same sequence through `with_uniqueness(Resolve)`.
fn with_uniqueness_resolution() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(mem::Global::new())?.with_uniqueness(Resolve);

    let a = store.create_point()?;
    let b = store.create_point()?;
    let created: Vec<_> = (0..DUPLICATES)
        .map(|_| store.create_link(a, b))
        .collect::<Result<_, _>>()?;

    println!("with_uniqueness(Resolve):");
    println!("  create_link({a}, {b}) x{DUPLICATES} -> {created:?}");
    println!("  count = {}", store.count());

    let ab = created[0];
    store.delete(ab)?;

    println!("  delete({ab}) -> ok, count = {}", store.count());
    println!("  links = {:?}", store.iter().collect::<Vec<_>>());

    Ok(())
}
