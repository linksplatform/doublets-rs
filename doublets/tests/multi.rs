use doublets::{split, unit, Error};
use mem::Global;
use std::time::Instant;

mod extensions;

#[test]
#[cfg(not(miri))]
fn random_crud_unit() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let instant = Instant::now();
    extensions::test_random_creations_and_deletions(&mut store, 1000);
    println!("{:?}", instant.elapsed());

    Ok(())
}

#[test]
#[cfg(not(miri))]
fn random_crud_split() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let instant = Instant::now();
    extensions::test_random_creations_and_deletions(&mut store, 1000);
    println!("{:?}", instant.elapsed());

    Ok(())
}
