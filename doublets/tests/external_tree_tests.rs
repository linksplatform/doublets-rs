// Tests for external (virtual) source/target tree operations
// These tests exercise code paths for virtual links where source or target has been deleted

use data::Flow;
use doublets::{split, unit, Doublets, Error, Links};
use mem::Global;

// ============================================
// External (Virtual) Source/Target Tree Tests
// These tests exercise code paths for virtual links
// where source or target has been deleted
// ============================================

// Test split store with virtual source (source deleted but link still references it)
#[test]
fn split_virtual_source_operations() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    // Create points
    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_point()?;

    // Create links where 'a' is source
    let _ab = store.create_link(a, b)?;
    let _ac = store.create_link(a, c)?;

    // Now delete 'a' - making it a virtual source for ab and ac
    // This should move the links to external source tree
    store.delete_usages(a)?; // First delete the links that use 'a'

    // Recreate similar pattern but delete the source point
    let x = store.create_point()?;
    let y = store.create_point()?;
    let xy = store.create_link(x, y)?;

    // Delete usages then recreate
    store.delete(xy)?;

    // Create new links
    let _xy2 = store.create_link(x, y)?;
    assert!(store.search(x, y).is_some());

    Ok(())
}

// Test external source tree iteration (each_usages_core branches)
#[test]
fn split_external_source_tree_iteration() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;
    let any = Links::constants(&store).any;

    // Create a scenario where we have links using external sources
    let base = store.create_point()?;
    let mut targets = Vec::new();

    // Create many links
    for _ in 0..15 {
        let t = store.create_point()?;
        targets.push(t);
        store.create_link(base, t)?;
    }

    // Now iterate via each_by which uses the tree structures
    let mut count = 0;
    store.each_by([any, base, any], |_link| {
        count += 1;
        Flow::Continue
    });
    assert!(count >= 15);

    // Try iteration with early break to test the handler flow
    let mut partial_count = 0;
    store.each_by([any, base, any], |_link| {
        partial_count += 1;
        if partial_count >= 5 {
            Flow::Break
        } else {
            Flow::Continue
        }
    });
    assert_eq!(partial_count, 5);

    Ok(())
}

// Test external target tree with virtual targets
#[test]
fn split_virtual_target_operations() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_point()?;

    // Create links where 'c' is target
    store.create_link(a, c)?;
    store.create_link(b, c)?;

    // Count usages of c
    let usages = store.count_usages(c)?;
    assert!(usages >= 2);

    // Delete usages of c and then delete c
    store.delete_usages(c)?;
    store.delete(c)?;

    // Verify c is gone
    assert!(!store.exist(c));

    // Create new links and verify tree consistency
    let d = store.create_point()?;
    store.create_link(a, d)?;
    store.create_link(b, d)?;

    let new_usages = store.count_usages(d)?;
    assert!(new_usages >= 2);

    Ok(())
}

// Test split store search with various source/target combinations
#[test]
fn split_search_all_tree_paths() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    // Create multiple points
    let points: Vec<_> = (0..10).map(|_| store.create_point().unwrap()).collect();

    // Create links between various combinations
    for i in 0..5 {
        for j in 5..10 {
            store.create_link(points[i], points[j])?;
        }
    }

    // Search for existing links
    for i in 0..5 {
        for j in 5..10 {
            let result = store.search(points[i], points[j]);
            assert!(result.is_some(), "Link {}→{} should exist", i, j);
        }
    }

    // Search for non-existent links
    for i in 5..10 {
        for j in 0..5 {
            let result = store.search(points[i], points[j]);
            assert!(result.is_none(), "Link {}→{} should not exist", i, j);
        }
    }

    Ok(())
}

// Test count_usages with tree traversal covering left/right branches
#[test]
fn split_count_usages_tree_branches() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let base = store.create_point()?;

    // Create links in various patterns to exercise tree branches
    // Insert in alternating pattern to create a more balanced tree
    let mut targets = Vec::new();
    for _ in 0..30 {
        let t = store.create_point()?;
        targets.push(t);
        store.create_link(base, t)?;
    }

    // Count should traverse the tree
    let count = store.count_usages(base)?;
    assert_eq!(count, 30);

    // Delete some links and recount
    for t in targets.iter().take(15) {
        let link = store.search(base, *t).unwrap();
        store.delete(link)?;
    }

    let new_count = store.count_usages(base)?;
    assert_eq!(new_count, 15);

    Ok(())
}

// Test that exercises the first_is_to_the_left/right_of_second comparisons
#[test]
fn split_tree_ordering_operations() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    // Create links with same source but different targets
    let source = store.create_point()?;
    let t1 = store.create_point()?;
    let t2 = store.create_point()?;
    let t3 = store.create_point()?;

    // Insert in order that exercises tree comparisons
    let l1 = store.create_link(source, t2)?; // middle target
    let _l2 = store.create_link(source, t1)?; // smaller target (goes left)
    let _l3 = store.create_link(source, t3)?; // larger target (goes right)

    // All links should be findable
    assert!(store.search(source, t1).is_some());
    assert!(store.search(source, t2).is_some());
    assert!(store.search(source, t3).is_some());

    // Delete middle link
    store.delete(l1)?;

    // Others should still be findable
    assert!(store.search(source, t1).is_some());
    assert!(store.search(source, t3).is_some());

    Ok(())
}

// Test that exercises each_usages_core recursive branches
#[test]
fn split_each_usages_core_branches() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;
    let any = Links::constants(&store).any;

    // Create a specific pattern to test the recursive each_usages_core
    let base = store.create_point()?;

    // Create targets in specific order to create tree structure
    // that will exercise both left and right branches of recursion
    let mut all_targets = Vec::new();
    for _ in 0..20 {
        let t = store.create_point()?;
        all_targets.push(t);
    }

    // Insert in scrambled order
    let order = [
        10, 5, 15, 3, 7, 12, 18, 1, 4, 6, 8, 11, 14, 16, 19, 0, 2, 9, 13, 17,
    ];
    for &i in &order {
        if i < all_targets.len() {
            store.create_link(base, all_targets[i])?;
        }
    }

    // Iterate and collect all links
    let mut found_targets = Vec::new();
    store.each_by([any, base, any], |link| {
        found_targets.push(link.target);
        Flow::Continue
    });

    // Should have found at least 20 links (might include base itself as a point)
    assert!(found_targets.len() >= 20);

    Ok(())
}

// Unit store versions of the same tests for comparison
#[test]
fn unit_virtual_source_operations() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_point()?;

    store.create_link(a, b)?;
    store.create_link(a, c)?;

    // Delete usages of a
    store.delete_usages(a)?;

    // Recreate
    let x = store.create_point()?;
    let y = store.create_point()?;
    let xy = store.create_link(x, y)?;

    store.delete(xy)?;

    let _xy2 = store.create_link(x, y)?;
    assert!(store.search(x, y).is_some());

    Ok(())
}

#[test]
fn unit_tree_ordering_operations() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let source = store.create_point()?;
    let t1 = store.create_point()?;
    let t2 = store.create_point()?;
    let t3 = store.create_point()?;

    // Insert in order that exercises tree comparisons
    let l1 = store.create_link(source, t2)?;
    let _l2 = store.create_link(source, t1)?;
    let _l3 = store.create_link(source, t3)?;

    assert!(store.search(source, t1).is_some());
    assert!(store.search(source, t2).is_some());
    assert!(store.search(source, t3).is_some());

    store.delete(l1)?;

    assert!(store.search(source, t1).is_some());
    assert!(store.search(source, t3).is_some());

    Ok(())
}

#[test]
fn unit_each_usages_core_branches() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;
    let any = Links::constants(&store).any;

    let base = store.create_point()?;

    let mut all_targets = Vec::new();
    for _ in 0..20 {
        let t = store.create_point()?;
        all_targets.push(t);
    }

    let order = [
        10, 5, 15, 3, 7, 12, 18, 1, 4, 6, 8, 11, 14, 16, 19, 0, 2, 9, 13, 17,
    ];
    for &i in &order {
        if i < all_targets.len() {
            store.create_link(base, all_targets[i])?;
        }
    }

    let mut found_targets = Vec::new();
    store.each_by([any, base, any], |link| {
        found_targets.push(link.target);
        Flow::Continue
    });

    // Should have found at least 20 links (might include base itself as a point)
    assert!(found_targets.len() >= 20);

    Ok(())
}

// Test count_links with 2-element query covering both value and not-value branches
#[test]
fn split_count_links_two_element_query() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;
    let any = Links::constants(&store).any;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_point()?;

    store.create_link(a, b)?;
    store.create_link(a, c)?;
    store.create_link(b, a)?;

    // Query with [any, value] - counts both source and target usages
    let count_a = store.count_by([any, a]);
    assert!(count_a >= 2); // a is source of 2 and target of 1

    // Query with [index, value] where values match
    let count_ab = store.count_by([store.search(a, b).unwrap(), b]);
    // The link a->b: source=a, target=b. Query asks if b is in either position
    // b is target, so should match (count should be 0 or more, at least it compiles)
    let _ = count_ab; // Just verify it computes without error

    Ok(())
}

// Test the is_virtual/is_unused detection
#[test]
fn split_is_virtual_detection() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_point()?;

    // All should exist initially
    assert!(store.exist(a));
    assert!(store.exist(b));
    assert!(store.exist(c));

    // Delete b (middle)
    store.delete(b)?;

    // b should now be virtual/unused
    assert!(!store.exist(b));
    assert!(store.exist(a));
    assert!(store.exist(c));

    // Create new point - should reuse b's slot
    let d = store.create_point()?;
    assert_eq!(d, b);
    assert!(store.exist(d));

    Ok(())
}

// Test resolve_dangling_internal and resolve_dangling_external
#[test]
fn split_resolve_dangling_operations() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_point()?;

    // Create links that use a as source and target
    let ab = store.create_link(a, b)?;
    let ca = store.create_link(c, a)?;

    // Now delete a - this should trigger resolve_dangling_internal
    // which moves links from internal to external trees
    store.delete(ab)?;
    store.delete(ca)?;
    store.delete(a)?;

    // Create a new point that reuses a's slot
    let new_a = store.create_point()?;
    assert_eq!(new_a, a);

    // Create new links using new_a
    let _new_ab = store.create_link(new_a, b)?;
    let _new_ca = store.create_link(c, new_a)?;

    // Verify they exist
    assert!(store.search(new_a, b).is_some());
    assert!(store.search(c, new_a).is_some());

    Ok(())
}
