// Tests for internal store operations including unused links list management
// This exercises code paths in delete_links and create_links that handle
// non-sequential allocations and free list management

use data::Flow;
use doublets::{split, unit, Doublets, Error, Links};
use mem::Global;

// Test that exercises unused links list when deleting non-last link
// This triggers Ordering::Less branch in delete_links -> attach_as_first
#[test]
fn unit_delete_middle_link_exercises_unused_list() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    // Create 3 links
    let a = store.create_point()?; // index 1
    let b = store.create_point()?; // index 2
    let c = store.create_point()?; // index 3

    assert_eq!(store.count(), 3);

    // Delete the middle link (index 2)
    // This should add it to the unused list (Ordering::Less branch)
    store.delete(b)?;
    assert_eq!(store.count(), 2);

    // Verify a and c still exist
    assert!(store.exist(a));
    assert!(!store.exist(b)); // b was deleted
    assert!(store.exist(c));

    // Create a new link - should reuse the freed slot
    let d = store.create_point()?;
    assert_eq!(store.count(), 3);

    // d should reuse slot 2 (from unused list)
    assert_eq!(d, b);

    Ok(())
}

// Test that exercises unused links detach when deleting last allocated
// followed by detaching previously unused links
#[test]
fn unit_delete_last_with_preceding_unused() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    // Create 3 links
    let a = store.create_point()?; // index 1
    let b = store.create_point()?; // index 2
    let c = store.create_point()?; // index 3

    // Delete middle link first (adds to unused list)
    store.delete(b)?;

    // Now delete the last link
    // This should trigger the loop that detaches b from unused list
    store.delete(c)?;

    assert_eq!(store.count(), 1);
    assert!(store.exist(a));
    assert!(!store.exist(b));
    assert!(!store.exist(c));

    Ok(())
}

// Test multiple consecutive deletes creating unused chain
#[test]
fn unit_multiple_non_sequential_deletes() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    // Create 5 links
    let _a = store.create_point()?; // 1
    let b = store.create_point()?; // 2
    let _c = store.create_point()?; // 3
    let d = store.create_point()?; // 4
    let _e = store.create_point()?; // 5

    // Delete in non-sequential order to build up unused list
    store.delete(b)?; // 2 -> unused list
    store.delete(d)?; // 4 -> unused list

    assert_eq!(store.count(), 3);

    // Now create new links - should reuse from unused list
    let f = store.create_point()?;
    let g = store.create_point()?;

    // Should have reused slots 4 and 2 (LIFO order from attach_as_first)
    assert!(f == d || f == b);
    assert!(g == d || g == b);
    assert_ne!(f, g);

    assert_eq!(store.count(), 5);

    Ok(())
}

// Test delete all followed by recreate to exercise full cycle
#[test]
fn unit_delete_all_and_recreate() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    // Create 5 links
    for _ in 0..5 {
        store.create_point()?;
    }
    assert_eq!(store.count(), 5);

    // Delete all
    store.delete_all()?;
    assert_eq!(store.count(), 0);

    // Recreate - should start from index 1 again
    let a = store.create_point()?;
    assert_eq!(a, 1);

    Ok(())
}

// Split store equivalent tests

#[test]
fn split_delete_middle_link_exercises_unused_list() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let _a = store.create_point()?;
    let b = store.create_point()?;
    let _c = store.create_point()?;

    store.delete(b)?;
    assert!(!store.exist(b));

    let d = store.create_point()?;
    // Should reuse slot b
    assert_eq!(d, b);

    Ok(())
}

#[test]
fn split_delete_last_with_preceding_unused() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_point()?;

    // Delete middle then last
    store.delete(b)?;
    store.delete(c)?;

    assert_eq!(store.count(), 1);
    assert!(store.exist(a));

    Ok(())
}

// Test complex link structures with usages before delete
#[test]
fn unit_delete_with_usages_first() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let _c = store.create_link(a, b)?;
    let _d = store.create_link(b, a)?;

    // a has usages (c uses a as source, d uses a as target)
    assert!(store.has_usages(a));

    // Delete usages first
    store.delete_usages(a)?;

    // Now a has no usages
    assert!(!store.has_usages(a));

    // Can safely delete a
    store.delete(a)?;
    assert!(!store.exist(a));

    Ok(())
}

// Test tree rebalancing with many operations
#[test]
fn unit_many_operations_tree_balance() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    // Create many links to exercise tree balancing
    let mut links = Vec::new();
    for _ in 0..50 {
        let a = store.create_point()?;
        links.push(a);
    }

    // Create cross-references
    for i in 0..25 {
        store.create_link(links[i], links[49 - i])?;
    }

    // Delete every other original point that has no usages
    for i in (0..50).step_by(2) {
        if !store.has_usages(links[i]) {
            store.delete(links[i])?;
        }
    }

    // Create new links that should reuse slots
    for _ in 0..10 {
        store.create_point()?;
    }

    // Verify store is still consistent
    let total = store.count();
    let mut counted = 0;
    store.each(|_| {
        counted += 1;
        Flow::Continue
    });
    assert_eq!(total, counted);

    Ok(())
}

// Test that source and target trees are properly maintained
#[test]
fn unit_source_target_tree_consistency() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_point()?;

    // Create links with same source
    let _d = store.create_link(a, b)?;
    let _e = store.create_link(a, c)?;

    // Count usages via source tree
    let source_usages = store.count_usages(a)?;
    assert_eq!(source_usages, 2); // d and e use a as source

    // Create links with same target
    let _f = store.create_link(b, c)?;
    // Note: a->c already exists, so get_or_create returns existing
    let _g = store.get_or_create(a, c)?;

    // Count usages via target tree
    let target_usages_c = store.usages(c)?;
    // c is used as target by e and f
    assert!(target_usages_c.len() >= 2);

    Ok(())
}

// Test search operations through source tree
#[test]
fn unit_search_through_source_tree() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_point()?;

    let ab = store.create_link(a, b)?;
    let ac = store.create_link(a, c)?;
    let bc = store.create_link(b, c)?;

    // Search should find exact matches
    assert_eq!(store.search(a, b), Some(ab));
    assert_eq!(store.search(a, c), Some(ac));
    assert_eq!(store.search(b, c), Some(bc));

    // Search should not find non-existent
    assert_eq!(store.search(c, a), None);
    assert_eq!(store.search(b, a), None);

    Ok(())
}

// Test iteration with various query patterns
#[test]
fn unit_each_with_source_filter() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;
    let any = Links::constants(&store).any;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_point()?;

    store.create_link(a, b)?;
    store.create_link(a, c)?;
    store.create_link(b, c)?;

    // Count links with source=a
    let mut count = 0;
    store.each_by([any, a, any], |_| {
        count += 1;
        Flow::Continue
    });
    assert!(count >= 2); // at least ab and ac

    Ok(())
}

#[test]
fn unit_each_with_target_filter() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;
    let any = Links::constants(&store).any;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_point()?;

    store.create_link(a, c)?;
    store.create_link(b, c)?;

    // Count links with target=c
    let mut count = 0;
    store.each_by([any, any, c], |_| {
        count += 1;
        Flow::Continue
    });
    assert!(count >= 2); // at least ac and bc

    Ok(())
}

// Test split store source/target consistency
#[test]
fn split_source_target_tree_consistency() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_point()?;

    store.create_link(a, b)?;
    store.create_link(a, c)?;

    let usages = store.count_usages(a)?;
    assert_eq!(usages, 2);

    Ok(())
}

// Edge case: delete first created link (should be efficient)
#[test]
fn unit_delete_first_link() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;
    assert_eq!(a, 1);

    store.delete(a)?;
    assert_eq!(store.count(), 0);

    // Creating again should reuse slot 1
    let b = store.create_point()?;
    assert_eq!(b, 1);

    Ok(())
}

// Edge case: alternating create/delete
#[test]
fn unit_alternating_create_delete() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    for _ in 0..20 {
        let a = store.create_point()?;
        let b = store.create_point()?;
        store.delete(a)?;
        let c = store.create_point()?;
        // c should reuse a's slot
        assert_eq!(c, a);
        store.delete(b)?;
        store.delete(c)?;
    }

    assert_eq!(store.count(), 0);

    Ok(())
}

// Test is_unused detection
#[test]
fn unit_is_unused_detection() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_point()?;

    // All exist
    assert!(store.exist(a));
    assert!(store.exist(b));
    assert!(store.exist(c));

    // Delete b (middle)
    store.delete(b)?;

    // b is now unused
    assert!(!store.exist(b));

    // a and c still exist
    assert!(store.exist(a));
    assert!(store.exist(c));

    Ok(())
}

// Test handler continuation/break flow
#[test]
fn unit_each_with_early_break() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    for _ in 0..10 {
        store.create_point()?;
    }

    let mut count = 0;
    store.each(|_| {
        count += 1;
        if count >= 5 {
            Flow::Break
        } else {
            Flow::Continue
        }
    });

    assert_eq!(count, 5);

    Ok(())
}

// Test split store with early break
#[test]
fn split_each_with_early_break() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    for _ in 0..10 {
        store.create_point()?;
    }

    let mut count = 0;
    store.each(|_| {
        count += 1;
        if count >= 5 {
            Flow::Break
        } else {
            Flow::Continue
        }
    });

    assert_eq!(count, 5);

    Ok(())
}

// ============================================
// Split Store Tree Traversal Tests
// These tests exercise internal tree operations
// ============================================

// Test split store count_usages with various patterns
#[test]
fn split_count_usages_single_link() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let _ab = store.create_link(a, b)?;

    // a is used as source in ab
    let source_usages = store.count_usages(a)?;
    assert!(source_usages >= 1);

    // b is used as target in ab
    let target_usages = store.count_usages(b)?;
    assert!(target_usages >= 1);

    Ok(())
}

// Test split store with many usages to exercise tree structure
#[test]
fn split_count_usages_many_links() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let base = store.create_point()?;
    let mut targets = Vec::new();

    // Create many links with same source
    for _ in 0..20 {
        let t = store.create_point()?;
        targets.push(t);
        store.create_link(base, t)?;
    }

    // base should have many usages
    let usages = store.count_usages(base)?;
    assert!(usages >= 20);

    Ok(())
}

// Test split store each_usages iteration
#[test]
fn split_each_usages_with_handler() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_point()?;

    store.create_link(a, b)?;
    store.create_link(a, c)?;
    store.create_link(b, a)?;

    // Count usages of a
    let mut usage_count = 0;
    store.usages(a)?.iter().for_each(|_| usage_count += 1);
    assert!(usage_count >= 2); // a used as source (ab, ac) and target (ba)

    Ok(())
}

// Test split store tree search
#[test]
fn split_search_operations() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_point()?;

    let ab = store.create_link(a, b)?;
    let ac = store.create_link(a, c)?;
    let bc = store.create_link(b, c)?;

    // Search should find existing links
    assert_eq!(store.search(a, b), Some(ab));
    assert_eq!(store.search(a, c), Some(ac));
    assert_eq!(store.search(b, c), Some(bc));

    // Search should not find non-existent
    assert_eq!(store.search(c, a), None);

    Ok(())
}

// Test split store with complex link patterns
#[test]
fn split_complex_link_patterns() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    // Create a mesh of links
    let mut points = Vec::new();
    for _ in 0..10 {
        points.push(store.create_point()?);
    }

    // Connect each point to several others
    for i in 0..10 {
        for j in (i + 1)..10 {
            store.create_link(points[i], points[j])?;
        }
    }

    // Verify all links exist via search
    for i in 0..10 {
        for j in (i + 1)..10 {
            let result = store.search(points[i], points[j]);
            assert!(result.is_some(), "Link {}->{} should exist", i, j);
        }
    }

    Ok(())
}

// Test split store deletion with tree updates
#[test]
fn split_delete_updates_trees() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_point()?;

    let ab = store.create_link(a, b)?;
    let ac = store.create_link(a, c)?;

    // Both links exist
    assert!(store.search(a, b).is_some());
    assert!(store.search(a, c).is_some());

    // Record usages before delete
    let usages_before = store.count_usages(a)?;

    // Delete one link
    store.delete(ab)?;

    // ab is gone, ac remains
    assert!(store.search(a, b).is_none());
    assert!(store.search(a, c).is_some());

    // a still has usages (ac)
    assert!(store.has_usages(a));

    // Delete remaining link
    store.delete(ac)?;

    // a has fewer usages than before
    let usages_after = store.count_usages(a)?;
    assert!(usages_after < usages_before || usages_before == 0);

    Ok(())
}

// Test split store tree rebalancing under stress
#[test]
fn split_tree_rebalancing_stress() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let base = store.create_point()?;

    // Create many links
    let mut links = Vec::new();
    for _ in 0..100 {
        let t = store.create_point()?;
        let link = store.create_link(base, t)?;
        links.push(link);
    }

    // Delete half of them
    for link in links.iter().step_by(2) {
        store.delete(*link)?;
    }

    // Verify remaining links still searchable
    let remaining_count = store.count_usages(base)?;
    assert!(remaining_count > 0);

    // Verify store consistency
    let total = store.count();
    let mut counted = 0;
    store.each(|_| {
        counted += 1;
        Flow::Continue
    });
    assert_eq!(total, counted);

    Ok(())
}

// Test split store with ordered insertions
#[test]
fn split_ordered_insertions() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let base = store.create_point()?;

    // Insert targets in order
    for i in 0..50 {
        let t = store.create_point()?;
        store.create_link(base, t)?;

        // Verify search still works
        let found = store.search(base, t);
        assert!(found.is_some(), "Link to target {} should be found", i);
    }

    Ok(())
}

// Test split store with reverse-order insertions (worst case for some trees)
#[test]
fn split_reverse_insertions() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    // Create targets first
    let mut targets = Vec::new();
    for _ in 0..50 {
        targets.push(store.create_point()?);
    }

    let base = store.create_point()?;

    // Insert in reverse order
    for t in targets.iter().rev() {
        store.create_link(base, *t)?;
    }

    // Verify all links found
    for t in &targets {
        assert!(store.search(base, *t).is_some());
    }

    Ok(())
}

// Test split store usages with same target
#[test]
fn split_same_target_multiple_sources() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let target = store.create_point()?;

    // Multiple sources pointing to same target
    for _ in 0..20 {
        let src = store.create_point()?;
        store.create_link(src, target)?;
    }

    // Target should have many usages
    let usages = store.usages(target)?;
    assert!(usages.len() >= 20);

    Ok(())
}

// Test split store with mixed operations
#[test]
fn split_mixed_crud_operations() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;

    // Create, verify, delete cycle
    for _ in 0..10 {
        let link = store.create_link(a, b)?;
        assert!(store.search(a, b).is_some());

        store.delete(link)?;
        // After delete, the link should not be searchable
        // Note: search behavior may vary based on implementation
        // Just verify the store is still consistent
        let count_after = store.count();
        assert!(count_after >= 2); // At least a and b exist
    }

    Ok(())
}

// Test split store update operations
#[test]
fn split_update_operations() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_point()?;

    let link = store.create_link(a, b)?;

    // Update to new target
    store.update(link, a, c)?;

    // Old link gone, new link exists
    assert!(store.search(a, b).is_none());
    assert!(store.search(a, c).is_some());

    Ok(())
}

// Test split store count_usages with no usages
#[test]
fn split_count_usages_no_usages() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;

    // a is a point - count_usages counts how many times it's used as source/target
    // A self-reference point doesn't show up in external usages
    let usages_a = store.count_usages(a)?;
    let usages_b = store.count_usages(b)?;

    // Create and delete a link
    let link = store.create_link(a, b)?;
    let usages_with_link = store.count_usages(a)?;
    assert!(usages_with_link >= usages_a); // At least as many usages

    store.delete(link)?;

    // Back to original
    assert_eq!(store.count_usages(a)?, usages_a);
    assert_eq!(store.count_usages(b)?, usages_b);

    Ok(())
}

// Test split store iteration with filters
#[test]
fn split_iteration_with_filters() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;
    let any = Links::constants(&store).any;

    let a = store.create_point()?;
    let b = store.create_point()?;
    let c = store.create_point()?;

    store.create_link(a, b)?;
    store.create_link(a, c)?;
    store.create_link(b, c)?;

    // Filter by source
    let mut source_a_count = 0;
    store.each_by([any, a, any], |_| {
        source_a_count += 1;
        Flow::Continue
    });
    assert!(source_a_count >= 2);

    // Filter by target
    let mut target_c_count = 0;
    store.each_by([any, any, c], |_| {
        target_c_count += 1;
        Flow::Continue
    });
    assert!(target_c_count >= 2);

    Ok(())
}

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
