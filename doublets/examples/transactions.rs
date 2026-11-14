//! Run with `cargo run -p doublets --example transactions`.

use doublets::{
    decorators::{DecoratorsExt, MemoryTransitionLog, Resolve},
    mem::Global,
    unit, Doublets, DoubletsExt, Error,
};

fn main() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?
        .with_transactions(MemoryTransitionLog::default())?;

    let tx = store.begin_transaction()?;
    let mut policies = tx.with_uniqueness(Resolve);
    let a = policies.create_point()?;
    let b = policies.create_point()?;
    let ab = policies.create_link(a, b)?;
    assert_eq!(policies.create_link(a, b)?, ab);
    policies.into_inner().commit()?;
    println!("committed: {:?}", store.iter().collect::<Vec<_>>());

    {
        let mut tx = store.begin_transaction()?;
        tx.update(ab, b, a)?;
        tx.delete(a)?;
        // Dropping the handle restores all three links, including their addresses.
    }
    println!(
        "after implicit rollback: {:?}",
        store.iter().collect::<Vec<_>>()
    );
    assert_eq!(store.search(a, b), Some(ab));
    Ok(())
}
