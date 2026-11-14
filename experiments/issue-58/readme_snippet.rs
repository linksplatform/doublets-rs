//! Compile check for the `### Decorators` snippet in `README.md`.
//!
//! Run with: `cargo build --example readme_snippet` after copying into
//! `doublets/examples/`, or simply keep it in sync with the README by hand.

use doublets::{
    decorators::{DecoratorsExt, Resolve},
    mem, unit, Doublets,
};

fn main() -> Result<(), doublets::Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(mem::Global::new())?
        .with_uniqueness(Resolve)
        .with_usages_validation();

    let a = store.create_point()?;
    let b = store.create_point()?;

    // Creating the same doublet twice returns the existing link instead of a duplicate.
    assert_eq!(store.create_link(a, b)?, store.create_link(a, b)?);
    Ok(())
}
