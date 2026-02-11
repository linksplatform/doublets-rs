#![feature(box_syntax)]

use doublets::{unit, Doublets, Error};
use mem::Global;

pub mod extensions;

#[test]
fn basic() -> Result<(), Error<usize>> {
    let mut store: Box<dyn Doublets<Item = usize>> = box unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let _ = store.create_link(a, b)?;

    assert_eq!(store.count(), 3);

    Ok(())
}
