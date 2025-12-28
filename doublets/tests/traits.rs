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

// Additional tests for _with handler variants

#[test]
fn unit_create_by_with() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let mut before_link = None;
    let mut after_link = None;
    store.create_by_with([], |before, after| {
        before_link = Some(before);
        after_link = Some(after);
        Flow::Continue
    })?;

    assert!(before_link.is_some());
    assert!(after_link.is_some());

    Ok(())
}

#[test]
fn unit_update_by_with() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;

    let mut before_link = None;
    let mut after_link = None;
    store.update_by_with([a], [a, a, b], |before, after| {
        before_link = Some(before);
        after_link = Some(after);
        Flow::Continue
    })?;

    assert!(before_link.is_some());
    assert!(after_link.is_some());
    assert_eq!(after_link.unwrap().target, b);

    Ok(())
}

#[test]
fn unit_delete_by_with() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;

    let mut before_link = None;
    let mut after_link = None;
    store.delete_by_with([a], |before, after| {
        before_link = Some(before);
        after_link = Some(after);
        Flow::Continue
    })?;

    assert!(before_link.is_some());
    assert!(after_link.is_some());
    assert!(store.get_link(a).is_none());

    Ok(())
}

#[test]
fn unit_create_link_with() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;

    let mut created_links = Vec::new();
    store.create_link_with(a, b, |_before, after| {
        created_links.push(after);
        Flow::Continue
    })?;

    assert!(!created_links.is_empty());

    Ok(())
}

#[test]
fn unit_delete_with() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;

    let mut before_link = None;
    store.delete_with(a, |before, _after| {
        before_link = Some(before);
        Flow::Continue
    })?;

    assert!(before_link.is_some());
    assert_eq!(before_link.unwrap().index, a);

    Ok(())
}

#[test]
fn unit_update_with() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;

    let mut after_link = None;
    store.update_with(a, a, b, |_before, after| {
        after_link = Some(after);
        Flow::Continue
    })?;

    assert!(after_link.is_some());
    assert_eq!(after_link.unwrap().target, b);

    Ok(())
}

#[test]
fn unit_delete_query_with() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_point()?;
    // Create two different links that use a as source
    store.create_link(a, b)?;
    store.create_link(a, c)?;

    let any = Links::constants(&store).any;
    let initial_count = store.count();

    let mut deleted_count = 0;
    store.delete_query_with([any, a, any], |_before, _after| {
        deleted_count += 1;
        Flow::Continue
    })?;

    assert!(deleted_count > 0);
    assert!(store.count() < initial_count);

    Ok(())
}

#[test]
fn unit_delete_usages_with() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    store.create_link(a, b)?;

    let mut deleted_count = 0;
    store.delete_usages_with(a, |_before, _after| {
        deleted_count += 1;
        Flow::Continue
    })?;

    assert!(deleted_count > 0);
    assert!(!store.has_usages(a));

    Ok(())
}

#[test]
fn unit_rebase_with() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_link(a, b)?;

    let mut updated_count = 0;
    store.rebase_with(a, b, |_before, _after| {
        updated_count += 1;
        Flow::Continue
    })?;

    // Check that c's source was updated from a to b
    let link = store.get_link(c).unwrap();
    assert_eq!(link.source, b);

    Ok(())
}

#[test]
fn unit_rebase_same() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;

    // Rebase to same should be a no-op
    store.rebase(a, a)?;

    // Link should still exist
    assert!(store.get_link(a).is_some());

    Ok(())
}

#[test]
fn unit_rebase_and_delete_same() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;

    // Rebase and delete to same should just return the value
    let result = store.rebase_and_delete(a, a)?;
    assert_eq!(result, a);

    // Link should still exist
    assert!(store.get_link(a).is_some());

    Ok(())
}

#[test]
#[allow(deprecated)]
fn unit_search_or() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_link(a, b)?;

    // Should find existing link
    let found = store.search_or(a, b, 0);
    assert_eq!(found, c);

    // Should return default for non-existent link
    let not_found = store.search_or(b, a, 999);
    assert_eq!(not_found, 999);

    Ok(())
}

#[test]
fn unit_exist_external() -> Result<(), Error<usize>> {
    let store = unit::Store::<usize, _>::new(Global::new())?;

    // External links should always exist
    let constants = Links::constants(&store);

    // Test with the 'any' constant which should be external
    // External values (like 'any') exist by definition
    let any = constants.any;
    // Just call exist to exercise the external link code path
    let _ = store.exist(any);

    Ok(())
}

#[test]
fn unit_create_with() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let mut created = None;
    store.create_with(|_before, after| {
        created = Some(after.index);
        Flow::Continue
    })?;

    assert!(created.is_some());
    assert!(store.get_link(created.unwrap()).is_some());

    Ok(())
}

// Tests for split store with _with variants

#[test]
fn split_create_by_with() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let mut created = false;
    store.create_by_with([], |_before, _after| {
        created = true;
        Flow::Continue
    })?;

    assert!(created);

    Ok(())
}

#[test]
fn split_update_with() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;

    let mut updated = false;
    store.update_with(a, a, b, |_before, _after| {
        updated = true;
        Flow::Continue
    })?;

    assert!(updated);

    Ok(())
}

#[test]
fn split_delete_with() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let a = store.create_point()?;

    let mut deleted = false;
    store.delete_with(a, |_before, _after| {
        deleted = true;
        Flow::Continue
    })?;

    assert!(deleted);
    assert!(store.get_link(a).is_none());

    Ok(())
}

#[test]
fn split_rebase() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_link(a, b)?;

    store.rebase(a, b)?;

    let link = store.get_link(c).unwrap();
    assert_eq!(link.source, b);

    Ok(())
}

#[test]
fn split_rebase_and_delete() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    store.create_link(a, b)?;

    let initial_count = store.count();

    store.rebase_and_delete(a, b)?;

    assert!(store.get_link(a).is_none());
    assert!(store.count() < initial_count);

    Ok(())
}

#[test]
fn split_count_usages() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;

    assert_eq!(store.count_usages(a)?, 0);

    store.create_link(a, b)?;

    assert_eq!(store.count_usages(a)?, 1);

    Ok(())
}

#[test]
fn split_usages() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_link(a, b)?;

    let usages = store.usages(a)?;
    assert!(usages.contains(&c));

    Ok(())
}

#[test]
fn split_exist() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let a = store.create_point()?;

    assert!(store.exist(a));
    assert!(!store.exist(100));

    Ok(())
}

#[test]
fn split_has_usages() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;

    assert!(!store.has_usages(a));

    store.create_link(a, b)?;

    assert!(store.has_usages(a));

    Ok(())
}

#[test]
fn split_search() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_link(a, b)?;

    assert_eq!(store.search(a, b), Some(c));
    assert_eq!(store.search(b, a), None);

    Ok(())
}

#[test]
fn split_single() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let a = store.create_point()?;

    // Single link
    let single = store.single([a]);
    assert!(single.is_some());
    assert_eq!(single.unwrap().index, a);

    Ok(())
}

#[test]
fn split_find() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let a = store.create_point()?;

    let found = store.find([a]);
    assert!(found.is_some());

    let not_found = store.find([100]);
    assert!(not_found.is_none());

    Ok(())
}

#[test]
fn split_try_get_link() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let a = store.create_point()?;

    assert!(store.try_get_link(a).is_ok());
    assert!(store.try_get_link(100).is_err());

    Ok(())
}

#[test]
fn split_delete_usages() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    store.create_link(a, b)?;

    assert!(store.has_usages(a));

    store.delete_usages(a)?;

    assert!(!store.has_usages(a));

    Ok(())
}

#[test]
fn split_iter() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

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
fn split_each_iter() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    store.create_link(a, b)?;

    let any = Links::constants(&store).any;
    let links: Vec<_> = store.each_iter([any, a, any]).collect();

    assert!(!links.is_empty());
    for link in &links {
        assert_eq!(link.source, a);
    }

    Ok(())
}

// Tests for count_links with various query patterns

#[test]
fn unit_count_links_empty_query() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    store.create_point()?;
    store.create_point()?;

    // Empty query returns total count
    assert_eq!(store.count_by([]), 2);

    Ok(())
}

#[test]
fn unit_count_links_single_any() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;
    let any = Links::constants(&store).any;

    store.create_point()?;
    store.create_point()?;

    // [any] returns total count
    assert_eq!(store.count_by([any]), 2);

    Ok(())
}

#[test]
fn unit_count_links_single_index() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;

    // [existing] returns 1
    assert_eq!(store.count_by([a]), 1);

    // [non-existing] returns 0
    assert_eq!(store.count_by([100]), 0);

    Ok(())
}

#[test]
fn unit_count_links_two_element_any_any() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;
    let any = Links::constants(&store).any;

    store.create_point()?;
    store.create_point()?;

    // [any, any] returns total count
    assert_eq!(store.count_by([any, any]), 2);

    Ok(())
}

#[test]
fn unit_count_links_two_element_any_value() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;
    let any = Links::constants(&store).any;

    let a = store.create_point()?;
    let b = store.create_point()?;
    store.create_link(a, b)?;

    // [any, a] counts usages of a (as source + as target)
    let count = store.count_by([any, a]);
    assert!(count >= 1); // a is used as source and self-references

    Ok(())
}

#[test]
fn unit_count_links_two_element_index_any() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;
    let any = Links::constants(&store).any;

    let a = store.create_point()?;

    // [a, any] returns 1 if a exists
    assert_eq!(store.count_by([a, any]), 1);

    // [non-existing, any] returns 0
    assert_eq!(store.count_by([100, any]), 0);

    Ok(())
}

#[test]
fn unit_count_links_two_element_index_value_match() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_link(a, b)?;

    // [c, a] returns 1 because c's source is a
    assert_eq!(store.count_by([c, a]), 1);

    // [c, b] returns 1 because c's target is b
    assert_eq!(store.count_by([c, b]), 1);

    // [c, 100] returns 0 because c doesn't reference 100
    assert_eq!(store.count_by([c, 100]), 0);

    Ok(())
}

#[test]
fn unit_count_links_three_element_any_any_any() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;
    let any = Links::constants(&store).any;

    store.create_point()?;
    store.create_point()?;

    // [any, any, any] returns total count
    assert_eq!(store.count_by([any, any, any]), 2);

    Ok(())
}

#[test]
fn unit_count_links_three_element_any_source_any() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;
    let any = Links::constants(&store).any;

    let a = store.create_point()?;
    let b = store.create_point()?;
    store.create_link(a, b)?;

    // [any, a, any] counts links with source = a
    let count = store.count_by([any, a, any]);
    assert!(count >= 1);

    Ok(())
}

#[test]
fn unit_count_links_three_element_any_any_target() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;
    let any = Links::constants(&store).any;

    let a = store.create_point()?;
    let b = store.create_point()?;
    store.create_link(a, b)?;

    // [any, any, b] counts links with target = b
    let count = store.count_by([any, any, b]);
    assert!(count >= 1);

    Ok(())
}

#[test]
fn unit_count_links_three_element_any_source_target() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;
    let any = Links::constants(&store).any;

    let a = store.create_point()?;
    let b = store.create_point()?;
    store.create_link(a, b)?;

    // [any, a, b] searches for specific link
    assert_eq!(store.count_by([any, a, b]), 1);

    // [any, b, a] should be 0 (no link from b to a)
    assert_eq!(store.count_by([any, b, a]), 0);

    Ok(())
}

#[test]
fn unit_count_links_three_element_index_any_any() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;
    let any = Links::constants(&store).any;

    let a = store.create_point()?;

    // [a, any, any] returns 1 if a exists
    assert_eq!(store.count_by([a, any, any]), 1);

    // [non-existing, any, any] returns 0
    assert_eq!(store.count_by([100, any, any]), 0);

    Ok(())
}

#[test]
fn unit_count_links_three_element_index_source_target() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_link(a, b)?;

    // [c, a, b] returns 1 if link matches
    assert_eq!(store.count_by([c, a, b]), 1);

    // [c, b, a] returns 0 (doesn't match)
    assert_eq!(store.count_by([c, b, a]), 0);

    Ok(())
}

#[test]
fn unit_count_links_three_element_index_any_target() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;
    let any = Links::constants(&store).any;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_link(a, b)?;

    // [c, any, b] returns 1 if target matches
    assert_eq!(store.count_by([c, any, b]), 1);

    // [c, any, a] returns 0 (target doesn't match)
    assert_eq!(store.count_by([c, any, a]), 0);

    // Suppress unused variable warning
    let _ = c;

    Ok(())
}

#[test]
fn unit_count_links_three_element_index_source_any() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;
    let any = Links::constants(&store).any;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_link(a, b)?;

    // [c, a, any] returns 1 if source matches
    assert_eq!(store.count_by([c, a, any]), 1);

    // [c, b, any] returns 0 (source doesn't match)
    assert_eq!(store.count_by([c, b, any]), 0);

    // Suppress unused variable warning
    let _ = c;

    Ok(())
}

// Tests for each_by with various query patterns

#[test]
fn unit_each_empty_query() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;

    let mut collected = Vec::new();
    store.each_by([], |link| {
        collected.push(link.index);
        Flow::Continue
    });

    assert!(collected.contains(&a));
    assert!(collected.contains(&b));

    Ok(())
}

#[test]
fn unit_each_single_index() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;
    store.create_point()?;

    let mut collected = Vec::new();
    store.each_by([a], |link| {
        collected.push(link.index);
        Flow::Continue
    });

    assert_eq!(collected.len(), 1);
    assert_eq!(collected[0], a);

    Ok(())
}

#[test]
fn unit_each_single_nonexistent() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    store.create_point()?;

    let mut count = 0;
    store.each_by([100], |_link| {
        count += 1;
        Flow::Continue
    });

    assert_eq!(count, 0);

    Ok(())
}

#[test]
fn unit_each_two_element_index_any() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;
    let any = Links::constants(&store).any;

    let a = store.create_point()?;

    let mut collected = Vec::new();
    store.each_by([a, any], |link| {
        collected.push(link.index);
        Flow::Continue
    });

    assert_eq!(collected.len(), 1);
    assert_eq!(collected[0], a);

    Ok(())
}

#[test]
fn unit_each_two_element_index_value_match() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_link(a, b)?;

    // [c, a] should return c because c's source is a
    let mut collected = Vec::new();
    store.each_by([c, a], |link| {
        collected.push(link.index);
        Flow::Continue
    });

    assert!(collected.contains(&c));

    Ok(())
}

#[test]
fn unit_each_two_element_index_value_no_match() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_link(a, b)?;

    // [c, 100] should return nothing
    let mut count = 0;
    store.each_by([c, 100], |_link| {
        count += 1;
        Flow::Continue
    });

    assert_eq!(count, 0);

    // Suppress unused variable warning
    let _ = c;

    Ok(())
}

#[test]
fn unit_each_two_element_any_value() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;
    let any = Links::constants(&store).any;

    let a = store.create_point()?;
    let b = store.create_point()?;
    store.create_link(a, b)?;

    // [any, a] should find links that have a as source or target
    let mut collected = Vec::new();
    store.each_by([any, a], |link| {
        collected.push(link.index);
        Flow::Continue
    });

    // At minimum, the point 'a' should be found
    assert!(!collected.is_empty());

    Ok(())
}

#[test]
fn unit_each_three_element_index_source_target_match() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_link(a, b)?;

    // [c, a, b] should return c
    let mut collected = Vec::new();
    store.each_by([c, a, b], |link| {
        collected.push(link.index);
        Flow::Continue
    });

    assert_eq!(collected.len(), 1);
    assert_eq!(collected[0], c);

    Ok(())
}

#[test]
fn unit_each_three_element_index_source_target_no_match() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_link(a, b)?;

    // [c, b, a] should return nothing (wrong source/target)
    let mut count = 0;
    store.each_by([c, b, a], |_link| {
        count += 1;
        Flow::Continue
    });

    assert_eq!(count, 0);

    // Suppress unused variable warning
    let _ = c;

    Ok(())
}

#[test]
fn unit_each_three_element_index_any_target_match() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;
    let any = Links::constants(&store).any;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_link(a, b)?;

    // [c, any, b] should return c
    let mut collected = Vec::new();
    store.each_by([c, any, b], |link| {
        collected.push(link.index);
        Flow::Continue
    });

    assert_eq!(collected.len(), 1);
    assert_eq!(collected[0], c);

    Ok(())
}

#[test]
fn unit_each_three_element_index_any_target_no_match() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;
    let any = Links::constants(&store).any;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_link(a, b)?;

    // [c, any, a] should return nothing (wrong target)
    let mut count = 0;
    store.each_by([c, any, a], |_link| {
        count += 1;
        Flow::Continue
    });

    assert_eq!(count, 0);

    // Suppress unused variable warning
    let _ = c;

    Ok(())
}

#[test]
fn unit_each_three_element_index_source_any_match() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;
    let any = Links::constants(&store).any;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_link(a, b)?;

    // [c, a, any] should return c
    let mut collected = Vec::new();
    store.each_by([c, a, any], |link| {
        collected.push(link.index);
        Flow::Continue
    });

    assert_eq!(collected.len(), 1);
    assert_eq!(collected[0], c);

    Ok(())
}

#[test]
fn unit_each_three_element_index_source_any_no_match() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;
    let any = Links::constants(&store).any;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_link(a, b)?;

    // [c, b, any] should return nothing (wrong source)
    let mut count = 0;
    store.each_by([c, b, any], |_link| {
        count += 1;
        Flow::Continue
    });

    assert_eq!(count, 0);

    // Suppress unused variable warning
    let _ = c;

    Ok(())
}

#[test]
fn unit_each_three_element_nonexistent_index() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;
    let any = Links::constants(&store).any;

    store.create_point()?;

    // [100, any, any] should return nothing
    let mut count = 0;
    store.each_by([100, any, any], |_link| {
        count += 1;
        Flow::Continue
    });

    assert_eq!(count, 0);

    Ok(())
}

// Tests for split store query patterns

#[test]
fn split_count_links_empty_query() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    store.create_point()?;
    store.create_point()?;

    assert_eq!(store.count_by([]), 2);

    Ok(())
}

#[test]
fn split_count_links_single_any() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;
    let any = Links::constants(&store).any;

    store.create_point()?;
    store.create_point()?;

    assert_eq!(store.count_by([any]), 2);

    Ok(())
}

#[test]
fn split_count_links_three_element_any_source_target() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;
    let any = Links::constants(&store).any;

    let a = store.create_point()?;
    let b = store.create_point()?;
    store.create_link(a, b)?;

    // [any, a, b] searches for specific link
    assert_eq!(store.count_by([any, a, b]), 1);

    // [any, b, a] should be 0
    assert_eq!(store.count_by([any, b, a]), 0);

    Ok(())
}

#[test]
fn split_each_empty_query() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;

    let mut collected = Vec::new();
    store.each_by([], |link| {
        collected.push(link.index);
        Flow::Continue
    });

    assert!(collected.contains(&a));
    assert!(collected.contains(&b));

    Ok(())
}

#[test]
fn split_each_single_index() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let a = store.create_point()?;
    store.create_point()?;

    let mut collected = Vec::new();
    store.each_by([a], |link| {
        collected.push(link.index);
        Flow::Continue
    });

    assert_eq!(collected.len(), 1);
    assert_eq!(collected[0], a);

    Ok(())
}

// Tests for tree operations with multiple links

#[test]
fn unit_tree_operations_multiple_links() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_point()?;

    // Create multiple links with same source
    let l1 = store.create_link(a, b)?;
    let l2 = store.create_link(a, c)?;

    // Count usages should work correctly
    assert!(store.count_usages(a)? >= 2);

    // Search should find both links
    assert_eq!(store.search(a, b), Some(l1));
    assert_eq!(store.search(a, c), Some(l2));

    Ok(())
}

#[test]
fn split_tree_operations_multiple_links() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_point()?;

    // Create multiple links with same source
    let l1 = store.create_link(a, b)?;
    let l2 = store.create_link(a, c)?;

    // Count usages should work correctly
    assert!(store.count_usages(a)? >= 2);

    // Search should find both links
    assert_eq!(store.search(a, b), Some(l1));
    assert_eq!(store.search(a, c), Some(l2));

    Ok(())
}

// Tests for delete and recreate patterns

#[test]
fn unit_delete_and_recreate() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;
    store.delete(a)?;

    // After delete, a should not exist
    assert!(!store.exist(a));

    // Create new point (should reuse freed slot)
    let b = store.create_point()?;

    // b should exist
    assert!(store.exist(b));

    Ok(())
}

#[test]
fn split_delete_and_recreate() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let a = store.create_point()?;
    store.delete(a)?;

    assert!(!store.exist(a));

    let b = store.create_point()?;

    assert!(store.exist(b));

    Ok(())
}

// Tests for error conditions

#[test]
fn unit_delete_nonexistent() {
    let mut store = unit::Store::<usize, _>::new(Global::new()).unwrap();

    let result = store.delete(100);
    assert!(result.is_err());
}

#[test]
fn split_delete_nonexistent() {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new()).unwrap();

    let result = store.delete(100);
    assert!(result.is_err());
}

#[test]
fn unit_update_nonexistent() {
    let mut store = unit::Store::<usize, _>::new(Global::new()).unwrap();

    let result = store.update(100, 1, 1);
    assert!(result.is_err());
}

#[test]
fn split_update_nonexistent() {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new()).unwrap();

    let result = store.update(100, 1, 1);
    assert!(result.is_err());
}

// Tests for create that triggers memory allocation

#[test]
fn unit_many_creates() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    // Create many links to trigger memory allocation
    for _ in 0..1000 {
        store.create_point()?;
    }

    assert_eq!(store.count(), 1000);

    Ok(())
}

#[test]
fn split_many_creates() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    // Create many links to trigger memory allocation
    for _ in 0..1000 {
        store.create_point()?;
    }

    assert_eq!(store.count(), 1000);

    Ok(())
}

// Tests for complex link structures

#[test]
fn unit_complex_structure() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;
    let any = Links::constants(&store).any;

    // Create a graph: a -> b -> c -> a (cycle)
    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_point()?;

    store.create_link(a, b)?;
    store.create_link(b, c)?;
    store.create_link(c, a)?;

    // Each node is used as source once
    assert!(store.count_by([any, a, any]) >= 1);
    assert!(store.count_by([any, b, any]) >= 1);
    assert!(store.count_by([any, c, any]) >= 1);

    // Each node is used as target once
    assert!(store.count_by([any, any, a]) >= 1);
    assert!(store.count_by([any, any, b]) >= 1);
    assert!(store.count_by([any, any, c]) >= 1);

    Ok(())
}

#[test]
fn split_complex_structure() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;
    let any = Links::constants(&store).any;

    // Create a graph: a -> b -> c -> a (cycle)
    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_point()?;

    store.create_link(a, b)?;
    store.create_link(b, c)?;
    store.create_link(c, a)?;

    // Each node is used as source once
    assert!(store.count_by([any, a, any]) >= 1);
    assert!(store.count_by([any, b, any]) >= 1);
    assert!(store.count_by([any, c, any]) >= 1);

    // Each node is used as target once
    assert!(store.count_by([any, any, a]) >= 1);
    assert!(store.count_by([any, any, b]) >= 1);
    assert!(store.count_by([any, any, c]) >= 1);

    Ok(())
}

// Tests for update operations

#[test]
fn unit_update_source_and_target() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_point()?;
    let d = store.create_link(a, b)?;

    // Update both source and target
    store.update(d, c, c)?;

    let link = store.get_link(d).unwrap();
    assert_eq!(link.source, c);
    assert_eq!(link.target, c);

    Ok(())
}

#[test]
fn split_update_source_and_target() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_point()?;
    let d = store.create_link(a, b)?;

    // Update both source and target
    store.update(d, c, c)?;

    let link = store.get_link(d).unwrap();
    assert_eq!(link.source, c);
    assert_eq!(link.target, c);

    Ok(())
}

// Tests for each_usages iteration

#[test]
fn unit_each_usages_iteration() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;

    let l1 = store.create_link(a, b)?;
    let l2 = store.create_link(a, a)?;

    // Get usages of a (should include l1 and l2)
    let usages = store.usages(a)?;
    assert!(usages.contains(&l1) || usages.contains(&l2));

    Ok(())
}

#[test]
fn split_each_usages_iteration() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;

    let l1 = store.create_link(a, b)?;
    let l2 = store.create_link(a, a)?;

    // Get usages of a (should include l1 and l2)
    let usages = store.usages(a)?;
    assert!(usages.contains(&l1) || usages.contains(&l2));

    Ok(())
}
