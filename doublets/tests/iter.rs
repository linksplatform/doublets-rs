use doublets::{split, unit, Doublets, DoubletsExt, Error, Link, Links};
use mem::Global;
use std::collections::HashSet;

#[test]
fn unit_iter() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    store.create_link(a, b)?;

    assert_eq!(
        store.iter().collect::<HashSet<_>>(),
        vec![Link::new(1, 1, 1), Link::new(2, 2, 2), Link::new(3, 1, 2),]
            .into_iter()
            .collect()
    );

    Ok(())
}

#[test]
fn unit_iter_bug() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_link(a, b)?;
    store.delete(b)?;
    store.update(c, b, b)?;

    assert_eq!(
        store.iter().collect::<HashSet<_>>(),
        vec![Link::new(1, 1, 1), Link::new(3, 2, 2),]
            .into_iter()
            .collect()
    );

    Ok(())
}

#[test]
fn unit_each_iter() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    store.create_link(1, 1)?;
    store.create_link(2, 1)?;
    store.create_link(3, 1)?;

    let any = store.constants().any;
    assert_eq!(
        store.each_iter([any, any, 1]).collect::<HashSet<_>>(),
        vec![Link::new(1, 1, 1), Link::new(2, 2, 1), Link::new(3, 3, 1),]
            .into_iter()
            .collect()
    );

    Ok(())
}

#[test]
fn split_iter() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    store.create_link(a, b)?;

    assert_eq!(
        store.iter().collect::<HashSet<_>>(),
        vec![Link::new(1, 1, 1), Link::new(2, 2, 2), Link::new(3, 1, 2),]
            .into_iter()
            .collect()
    );

    Ok(())
}

#[test]
fn split_iter_bug() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_link(a, b)?;
    store.delete(b)?;
    store.update(c, b, b)?;

    assert_eq!(
        store.iter().collect::<HashSet<_>>(),
        vec![Link::new(1, 1, 1), Link::new(3, 2, 2),]
            .into_iter()
            .collect()
    );

    Ok(())
}

#[test]
fn split_each_iter() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    store.create_link(1, 1)?;
    store.create_link(2, 1)?;
    store.create_link(3, 1)?;

    let any = store.constants().any;
    assert_eq!(
        store.each_iter([any, any, 1]).collect::<HashSet<_>>(),
        vec![Link::new(1, 1, 1), Link::new(2, 2, 1), Link::new(3, 3, 1),]
            .into_iter()
            .collect()
    );

    Ok(())
}
