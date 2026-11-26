//! Issue #94: `split::Store` indexes links by internal source with a linked
//! list by default, like `useLinkedList: true` of the C# `SplitMemoryLinks`,
//! and `split::TreeStore` keeps the size-balanced sources tree.

use doublets::{data::Flow, mem::Global, split, Doublets, Link};
use rand::{rngs::StdRng, RngExt, SeedableRng};
use std::collections::BTreeMap;

fn list_store() -> impl Doublets<usize> {
    split::Store::<usize, _, _>::new(Global::new(), Global::new()).unwrap()
}

fn tree_store() -> impl Doublets<usize> {
    split::TreeStore::<usize, _, _>::new(Global::new(), Global::new()).unwrap()
}

/// Indexes of the links with `source` in iteration order.
fn ordered_usages(store: &impl Doublets<usize>, source: usize) -> Vec<usize> {
    let any = store.constants().any;
    let mut found = Vec::new();
    store.each_by([any, source, any], |link| {
        found.push(link.index);
        Flow::Continue
    });
    assert_eq!(store.count_by([any, source, any]), found.len());
    found
}

fn sorted(mut indexes: Vec<usize>) -> Vec<usize> {
    indexes.sort_unstable();
    indexes
}

/// Indexes of the links with `source`, sorted.
fn usages_by_source(store: &impl Doublets<usize>, source: usize) -> Vec<usize> {
    sorted(ordered_usages(store, source))
}

/// Links `(head, 3)`, `(head, 2)`, `(head, 4)` created in this order after
/// the points 1 (the head) to 4.
fn links_with_one_source(store: &mut impl Doublets<usize>) -> (usize, [usize; 3]) {
    let head = store.create_point().unwrap();
    for _ in 0..3 {
        store.create_point().unwrap();
    }
    let a = store.create_link(head, 3).unwrap();
    let b = store.create_link(head, 2).unwrap();
    let c = store.create_link(head, 4).unwrap();
    (head, [a, b, c])
}

#[test]
fn default_split_store_lists_sources_in_insertion_order() {
    let mut store = list_store();
    let (head, [a, b, c]) = links_with_one_source(&mut store);
    // The list appends (C# `AttachAsLast`); the head itself is (head, head).
    assert_eq!(ordered_usages(&store, head), [head, a, b, c]);
    store.update(b, head, a).unwrap();
    assert_eq!(ordered_usages(&store, head), [head, a, c, b]);
    store.delete(a).unwrap();
    assert_eq!(ordered_usages(&store, head), [head, c, b]);
}

#[test]
fn tree_store_orders_sources_by_target() {
    let mut store = tree_store();
    let (head, [a, b, c]) = links_with_one_source(&mut store);
    let mut by_target = vec![head, a, b, c];
    by_target.sort_by_key(|&index| store.get_link(index).unwrap().target);
    assert_eq!(ordered_usages(&store, head), by_target);
}

/// `x` has a virtual (absent) source, so it is a node of the external sources
/// tree while it is also the head of the list of the links whose source is `x`.
fn head_with_virtual_source(mut store: impl Doublets<usize>) {
    let x = store.create().unwrap();
    let y = store.create().unwrap();
    let z = store.create().unwrap();
    let absent = 10;
    store.update(x, absent, 0).unwrap();
    store.update(y, x, 0).unwrap();
    store.update(z, x, x).unwrap();
    assert_eq!(usages_by_source(&store, x).len(), 2);
    assert_eq!(usages_by_source(&store, absent), [x]);
    assert_eq!(store.search(x, x), Some(z));

    // Create the absent ID: `x` moves from the external tree to its list.
    while store.count() < absent {
        store.create().unwrap();
    }
    assert!(store.get_link(absent).is_some());
    assert_eq!(usages_by_source(&store, absent), [x]);
    assert_eq!(usages_by_source(&store, x).len(), 2);
    store.delete(y).unwrap();
    assert_eq!(usages_by_source(&store, x), [z]);

    // Release the ID again: `x` returns to the external tree.
    store.delete(absent).unwrap();
    assert_eq!(usages_by_source(&store, absent), [x]);
    let w = store.create().unwrap();
    store.update(w, x, 20).unwrap();
    assert_eq!(usages_by_source(&store, x), sorted(vec![z, w]));
    assert_eq!(store.search(x, 20), Some(w));

    // Point `x` at itself: it leaves the external tree and joins its own list.
    store.update(x, x, 0).unwrap();
    assert_eq!(usages_by_source(&store, x), sorted(vec![z, w, x]));
    assert_eq!(usages_by_source(&store, absent), []);
    store.update(x, 0, 0).unwrap();
    store.update(z, 0, 0).unwrap();
    store.update(w, 0, 0).unwrap();
    assert_eq!(usages_by_source(&store, x), []);
}

#[test]
fn a_head_with_a_virtual_source_keeps_its_list() {
    head_with_virtual_source(list_store());
    head_with_virtual_source(tree_store());
}

fn internal_source_and_virtual_target(mut store: impl Doublets<usize>) {
    let source = store.create_point().unwrap();
    let link = store.create_link(source, 100).unwrap();
    // Before the fix, list mode searched this pair in the external sources tree.
    assert_eq!(store.search(source, 100), Some(link));
    assert_eq!(store.count_by([store.constants().any, source, 100]), 1);
    store.update(link, source, 0).unwrap();
    assert_eq!(store.search(source, 100), None);
    assert_eq!(usages_by_source(&store, source), [source, link]);
}

#[test]
fn an_internal_source_with_a_virtual_target_is_searchable() {
    internal_source_and_virtual_target(list_store());
    internal_source_and_virtual_target(tree_store());
}

fn null_and_self_endpoints(mut store: impl Doublets<usize>) {
    let a = store.create().unwrap();
    let b = store.create().unwrap();
    // A live (source, null) link keeps a zero size in list mode; it must not
    // look like a free slot.
    store.update(b, a, 0).unwrap();
    assert_eq!(store.get_link(b), Some(Link::new(b, a, 0)));
    assert_eq!(store.count(), 2);
    let c = store.create().unwrap();
    assert_eq!(c, 3, "a live link must not be reused");
    store.update(a, a, a).unwrap();
    store.update(a, a, b).unwrap();
    assert_eq!(usages_by_source(&store, a), [a, b]);
    store.delete(c).unwrap();
    store.update(b, 0, 0).unwrap();
    assert_eq!(usages_by_source(&store, a), [a]);
    assert_eq!(store.get_link(b), Some(Link::new(b, 0, 0)));
}

#[test]
fn null_and_self_endpoints_stay_live() {
    null_and_self_endpoints(list_store());
    null_and_self_endpoints(tree_store());
}

fn check_queries(store: &impl Doublets<usize>, model: &BTreeMap<usize, (usize, usize)>, at: &str) {
    let any = store.constants().any;
    assert_eq!(store.count(), model.len(), "{at}");
    for value in 1..=40 {
        for query in [[any, value, any], [any, any, value]] {
            let mut expected: Vec<_> = model
                .iter()
                .filter(|(_, &(source, target))| {
                    (query[1] == any || source == query[1])
                        && (query[2] == any || target == query[2])
                })
                .map(|(&index, _)| index)
                .collect();
            let mut found = Vec::new();
            store.each_by(query, |link| {
                found.push(link.index);
                Flow::Continue
            });
            found.sort_unstable();
            expected.sort_unstable();
            assert_eq!(found, expected, "{at}: query {query:?}");
            assert_eq!(
                store.count_by(query),
                expected.len(),
                "{at}: query {query:?}"
            );
        }
    }
    for (&index, &(source, target)) in model {
        assert_eq!(
            store.get_link(index),
            Some(Link::new(index, source, target)),
            "{at}"
        );
        if source != 0 && target != 0 {
            assert_eq!(store.search(source, target), Some(index), "{at}");
        }
    }
}

fn random_changes(mut store: impl Doublets<usize>, seed: u64) -> Vec<Link<usize>> {
    let mut random = StdRng::seed_from_u64(seed);
    let mut model = BTreeMap::new();
    // At most 24 live links and 300 operations keep this bounded; endpoints
    // include null, self references, and absent (virtual) IDs up to 40.
    for step in 0..300 {
        let operation = random.random_range(0..5);
        if model.is_empty() || (operation == 0 && model.len() < 24) {
            let index = store.create().unwrap();
            model.insert(index, (0, 0));
        } else {
            let index = *model
                .keys()
                .nth(random.random_range(0..model.len()))
                .unwrap();
            if operation == 1 {
                store.delete(index).unwrap();
                model.remove(&index);
            } else {
                let pair = (random.random_range(0..=40), random.random_range(0..=40));
                let pair = if operation == 4 {
                    (index, pair.1)
                } else {
                    pair
                };
                // Raw stores require unique pairs; uniqueness policy is optional.
                if pair != (0, 0) && model.values().any(|&value| value == pair) {
                    continue;
                }
                store.update(index, pair.0, pair.1).unwrap();
                model.insert(index, pair);
            }
        }
        check_queries(&store, &model, &format!("seed {seed}, step {step}"));
    }
    let mut links = Vec::new();
    store.each(|link| {
        links.push(link);
        Flow::Continue
    });
    links
}

#[test]
fn list_and_tree_stores_agree_with_a_plain_map() {
    for seed in [3, 94, 2026, 4096] {
        let list = random_changes(list_store(), seed);
        let tree = random_changes(tree_store(), seed);
        assert_eq!(list, tree, "seed {seed}");
    }
}
