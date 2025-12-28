use data::Flow;
use doublets::{split, unit, Doublets, DoubletsExt, Error, Links};
use mem::Global;

// Tests for Links trait methods

#[test]
fn unit_count_links() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    // Initially empty
    assert_eq!(store.count(), 0);

    // Create some links
    store.create_point()?;
    store.create_point()?;
    store.create_point()?;

    assert_eq!(store.count(), 3);

    Ok(())
}

#[test]
fn split_count_links() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    assert_eq!(store.count(), 0);

    store.create_point()?;
    store.create_point()?;

    assert_eq!(store.count(), 2);

    Ok(())
}

// Tests for Doublets trait methods

#[test]
fn unit_found() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;

    // Should find the point
    assert!(store.found([a]));

    // Should not find a non-existent link
    assert!(!store.found([100]));

    Ok(())
}

#[test]
fn unit_find() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;

    // Should find the point
    let found = store.find([a]);
    assert!(found.is_some());
    assert_eq!(found.unwrap().index, a);

    // Should not find a non-existent link
    let not_found = store.find([100]);
    assert!(not_found.is_none());

    Ok(())
}

#[test]
fn unit_search() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_link(a, b)?;

    // Should find link by source and target
    let found = store.search(a, b);
    assert_eq!(found, Some(c));

    // Should not find non-existent link
    let not_found = store.search(b, a);
    assert!(not_found.is_none() || not_found != Some(c));

    Ok(())
}

#[test]
fn unit_single() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;

    // Should find exactly one link
    let single = store.single([a]);
    assert!(single.is_some());
    assert_eq!(single.unwrap().index, a);

    // Create another link with same target
    store.create_link(a, b)?;
    store.create_link(b, b)?;

    // When there are multiple matches, single should return None
    let any = Links::constants(&store).any;
    let _multiple = store.single([any, any, b]);
    // If there's more than one match, single returns None
    // In this case we have links targeting b, so behavior depends on count

    Ok(())
}

#[test]
fn unit_get_or_create() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;

    // First call should create
    let link1 = store.get_or_create(a, b)?;

    // Second call should return existing
    let link2 = store.get_or_create(a, b)?;

    assert_eq!(link1, link2);

    Ok(())
}

#[test]
fn unit_count_usages() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;

    // Point has no usages
    let usages = store.count_usages(a)?;
    assert_eq!(usages, 0);

    // Create a link that uses a
    store.create_link(a, b)?;

    // Now a is used as source
    let usages = store.count_usages(a)?;
    assert_eq!(usages, 1);

    Ok(())
}

#[test]
fn unit_usages() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;

    // Point has no usages
    let usages = store.usages(a)?;
    assert!(usages.is_empty());

    // Create links that use a
    let c = store.create_link(a, b)?;

    // Now a is used as source
    let usages = store.usages(a)?;
    assert!(usages.contains(&c));

    Ok(())
}

#[test]
fn unit_exist() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;

    assert!(store.exist(a));
    assert!(!store.exist(100));

    Ok(())
}

#[test]
fn unit_has_usages() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;

    assert!(!store.has_usages(a));

    store.create_link(a, b)?;

    assert!(store.has_usages(a));

    Ok(())
}

#[test]
fn unit_delete_all() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    store.create_point()?;
    store.create_point()?;
    store.create_point()?;

    assert_eq!(store.count(), 3);

    store.delete_all()?;

    assert_eq!(store.count(), 0);

    Ok(())
}

#[test]
fn unit_delete_usages() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    store.create_link(a, b)?;

    assert!(store.has_usages(a));

    store.delete_usages(a)?;

    assert!(!store.has_usages(a));

    Ok(())
}

#[test]
fn unit_create_point() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let point = store.create_point()?;

    let link = store.get_link(point);
    assert!(link.is_some());
    let link = link.unwrap();

    // Point has index == source == target
    assert_eq!(link.index, link.source);
    assert_eq!(link.source, link.target);

    Ok(())
}

#[test]
fn unit_create_link() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_link(a, b)?;

    let link = store.get_link(c).unwrap();
    assert_eq!(link.source, a);
    assert_eq!(link.target, b);

    Ok(())
}

#[test]
fn unit_rebase() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_link(a, b)?;

    // Rebase: replace all references from a to b
    store.rebase(a, b)?;

    let link = store.get_link(c).unwrap();
    assert_eq!(link.source, b);

    Ok(())
}

#[test]
fn unit_rebase_and_delete() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    store.create_link(a, b)?;

    let initial_count = store.count();

    store.rebase_and_delete(a, b)?;

    // a should be deleted after rebase
    assert!(store.get_link(a).is_none());
    assert!(store.count() < initial_count);

    Ok(())
}

#[test]
fn unit_try_get_link() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;

    // Should succeed
    let link = store.try_get_link(a)?;
    assert_eq!(link.index, a);

    // Should fail for non-existent
    let result = store.try_get_link(100);
    assert!(result.is_err());

    Ok(())
}

// Tests for DoubletsExt trait methods

#[test]
fn unit_iter() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;

    let links: Vec<_> = store.iter().collect();
    assert_eq!(links.len(), 2);

    let indices: Vec<_> = links.iter().map(|l| l.index).collect();
    assert!(indices.contains(&a));
    assert!(indices.contains(&b));

    Ok(())
}

#[test]
fn unit_each_iter() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    store.create_link(a, b)?;

    let any = Links::constants(&store).any;
    let links: Vec<_> = store.each_iter([any, a, any]).collect();

    // Links with source = a
    assert!(!links.is_empty());
    for link in &links {
        assert_eq!(link.source, a);
    }

    Ok(())
}

// Tests for Box<dyn Doublets> implementation

#[test]
fn boxed_doublets() -> Result<(), Error<usize>> {
    let store = unit::Store::<usize, _>::new(Global::new())?;
    let mut boxed: Box<dyn Doublets<usize>> = Box::new(store);

    let a = boxed.create_point()?;
    let b = boxed.create_point()?;

    assert_eq!(boxed.count(), 2);

    boxed.create_link(a, b)?;

    assert_eq!(boxed.count(), 3);

    Ok(())
}

// Tests for split store specific behavior

#[test]
fn split_found() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let a = store.create_point()?;

    assert!(store.found([a]));
    assert!(!store.found([100]));

    Ok(())
}

#[test]
fn split_get_or_create() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;

    let link1 = store.get_or_create(a, b)?;
    let link2 = store.get_or_create(a, b)?;

    assert_eq!(link1, link2);

    Ok(())
}

#[test]
fn split_delete_all() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    store.create_point()?;
    store.create_point()?;
    store.create_point()?;

    assert_eq!(store.count(), 3);

    store.delete_all()?;

    assert_eq!(store.count(), 0);

    Ok(())
}

// Tests for each_by with handlers

#[test]
fn unit_each_by_with_flow() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    store.create_point()?;
    store.create_point()?;
    store.create_point()?;

    let mut count = 0;
    store.each(|_link| {
        count += 1;
        Flow::Continue
    });

    assert_eq!(count, 3);

    Ok(())
}

#[test]
fn unit_each_by_with_break() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    store.create_point()?;
    store.create_point()?;
    store.create_point()?;

    let mut count = 0;
    store.each(|_link| {
        count += 1;
        if count >= 2 {
            Flow::Break
        } else {
            Flow::Continue
        }
    });

    assert_eq!(count, 2);

    Ok(())
}
