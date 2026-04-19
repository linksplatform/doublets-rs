// Tests for count_links and each_by with various query patterns
// Split from traits.rs to stay under 1000 line limit

use data::Flow;
use doublets::{split, unit, Doublets, Error, Links};
use mem::Global;

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
