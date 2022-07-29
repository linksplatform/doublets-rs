use doublets::{split, unit, Doublets, Error};
use mem::Global;
use std::time::Instant;

const MILLION: usize = 1_000_000;

#[test]
#[cfg(not(miri))]
fn unit_million() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    for _ in 0..MILLION {
        store.create().unwrap();
    }

    assert_eq!(store.count(), MILLION);

    Ok(())
}

#[test]
#[cfg(not(miri))]
fn split_million() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    for _ in 0..MILLION {
        store.create().unwrap();
    }

    assert_eq!(store.count(), MILLION);

    Ok(())
}

#[test]
#[cfg(not(miri))]
fn unit_million_points() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let instant = Instant::now();
    for _ in 0..MILLION {
        store.create_point().unwrap();
    }
    println!("{:?}", instant.elapsed());

    assert_eq!(store.count(), MILLION);

    Ok(())
}

#[test]
#[cfg(not(miri))]
fn split_million_points() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let instant = Instant::now();
    for _ in 0..MILLION {
        store.create_point().unwrap();
    }
    println!("{:?}", instant.elapsed());

    assert_eq!(store.count(), MILLION);

    Ok(())
}
