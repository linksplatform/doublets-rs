#[cfg(feature = "rayon")]
use doublets::{split, unit, Doublets, DoubletsExt, Error, Link, Links};
#[cfg(feature = "rayon")]
use mem::Global;
#[cfg(feature = "rayon")]
use std::collections::HashSet;
#[cfg(feature = "rayon")]
use rayon::prelude::*;

#[cfg(feature = "rayon")]
#[test]
fn unit_par_iter() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    store.create_link(a, b)?;

    let par_results: HashSet<_> = store.par_iter().collect();
    let seq_results: HashSet<_> = store.iter().collect();

    assert_eq!(par_results, seq_results);
    assert_eq!(
        par_results,
        vec![Link::new(1, 1, 1), Link::new(2, 2, 2), Link::new(3, 1, 2),]
            .into_iter()
            .collect()
    );

    Ok(())
}

#[cfg(feature = "rayon")]
#[test]
fn unit_par_each_iter() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    store.create_link(1, 1)?;
    store.create_link(2, 1)?;
    store.create_link(3, 1)?;
    store.create_link(4, 2)?;
    store.create_link(5, 3)?;

    let any = store.constants().any;
    
    // Test parallel vs sequential consistency for target queries
    let par_results: HashSet<_> = store.par_each_iter([any, any, 1]).collect();
    let seq_results: HashSet<_> = store.each_iter([any, any, 1]).collect();
    assert_eq!(par_results, seq_results);
    
    // Test parallel vs sequential consistency for source queries  
    let par_results: HashSet<_> = store.par_each_iter([any, 2, any]).collect();
    let seq_results: HashSet<_> = store.each_iter([any, 2, any]).collect();
    assert_eq!(par_results, seq_results);

    // Test parallel vs sequential consistency for all links
    let par_results: HashSet<_> = store.par_each_iter([any, any, any]).collect();
    let seq_results: HashSet<_> = store.each_iter([any, any, any]).collect();
    assert_eq!(par_results, seq_results);

    Ok(())
}

#[cfg(feature = "rayon")]
#[test]
fn split_par_iter() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    store.create_link(a, b)?;

    let par_results: HashSet<_> = store.par_iter().collect();
    let seq_results: HashSet<_> = store.iter().collect();

    assert_eq!(par_results, seq_results);
    assert_eq!(
        par_results,
        vec![Link::new(1, 1, 1), Link::new(2, 2, 2), Link::new(3, 1, 2),]
            .into_iter()
            .collect()
    );

    Ok(())
}

#[cfg(feature = "rayon")]
#[test]
fn split_par_each_iter() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    store.create_link(1, 1)?;
    store.create_link(2, 1)?;
    store.create_link(3, 1)?;
    store.create_link(4, 2)?;
    store.create_link(5, 3)?;

    let any = store.constants().any;
    
    // Test parallel vs sequential consistency for target queries
    let par_results: HashSet<_> = store.par_each_iter([any, any, 1]).collect();
    let seq_results: HashSet<_> = store.each_iter([any, any, 1]).collect();
    assert_eq!(par_results, seq_results);
    
    // Test parallel vs sequential consistency for source queries  
    let par_results: HashSet<_> = store.par_each_iter([any, 2, any]).collect();
    let seq_results: HashSet<_> = store.each_iter([any, 2, any]).collect();
    assert_eq!(par_results, seq_results);

    // Test parallel vs sequential consistency for all links
    let par_results: HashSet<_> = store.par_each_iter([any, any, any]).collect();
    let seq_results: HashSet<_> = store.each_iter([any, any, any]).collect();
    assert_eq!(par_results, seq_results);

    Ok(())
}

#[cfg(feature = "rayon")]
#[test]  
fn parallel_performance_test() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    // Create a larger dataset for performance testing
    for i in 1..=100 {
        store.create_link(i, i)?;
        if i > 1 {
            store.create_link(i, i - 1)?;
        }
    }

    let any = store.constants().any;
    
    // Just ensure both methods return the same results
    let par_results: HashSet<_> = store.par_each_iter([any, any, any]).collect();
    let seq_results: HashSet<_> = store.each_iter([any, any, any]).collect();
    
    assert_eq!(par_results, seq_results);
    assert!(par_results.len() > 100); // Should have many links

    Ok(())
}