//! References to absent IDs must stay searchable as those IDs become live.

use doublets::{data::Flow, mem::Global, split, unit, Doublets, Link, Links};

fn assert_indexes(store: &impl Doublets<usize>) {
    let mut all = Vec::new();
    store.each(|link| {
        all.push(link);
        Flow::Continue
    });
    assert_eq!(store.count(), all.len());
    let any = store.constants().any;
    for link in &all {
        assert_eq!(store.get_link(link.index), Some(link.clone()));
        // Null endpoints of uninitialized links are deliberately not indexed.
        for query in [
            [any, link.source, any],
            [any, any, link.target],
            [any, link.source, link.target],
        ] {
            if query.contains(&0) {
                continue;
            }
            let mut expected: Vec<_> = all
                .iter()
                .filter(|link| {
                    (query[1] == any || link.source == query[1])
                        && (query[2] == any || link.target == query[2])
                })
                .map(|link| link.index)
                .collect();
            let mut found = Vec::new();
            store.each_by(query, |link| {
                found.push(link.index);
                Flow::Continue
            });
            expected.sort_unstable();
            found.sort_unstable();
            assert_eq!(found, expected, "query {query:?}");
            assert_eq!(store.count_by(query), expected.len(), "query {query:?}");
        }
        if link.source != 0 && link.target != 0 {
            assert_eq!(store.search(link.source, link.target), Some(link.index));
        }
    }
}

fn forward_reference(mut store: impl Doublets<usize>, source: bool) {
    store.create_point().unwrap();
    let b = store.create_point().unwrap();
    let pair = if source { (3, b) } else { (b, 3) };
    store.update(b, pair.0, pair.1).unwrap();
    let any = store.constants().any;
    let query = [any, pair.0, pair.1];
    let mut found = Vec::new();
    store.each_by(query, |link| {
        found.push(link.index);
        Flow::Continue
    });
    assert_eq!(found, [b], "before creating the referenced ID");

    assert_eq!(store.create_point().unwrap(), 3);
    assert_eq!(store.get_link(b), Some(Link::new(b, pair.0, pair.1)));
    found.clear();
    store.each_by(query, |link| {
        found.push(link.index);
        Flow::Continue
    });
    assert_eq!(found, [b], "after creating the referenced ID");
    assert_eq!(store.count_by(query), 1);
    assert_indexes(&store);
}

#[test]
fn split_forward_source_reference() {
    forward_reference(
        split::Store::<usize, _, _>::new(Global::new(), Global::new()).unwrap(),
        true,
    );
}

#[test]
fn split_forward_target_reference() {
    forward_reference(
        split::Store::<usize, _, _>::new(Global::new(), Global::new()).unwrap(),
        false,
    );
}

#[test]
fn unit_forward_references_control() {
    for source in [true, false] {
        forward_reference(unit::Store::<usize, _>::new(Global::new()).unwrap(), source);
    }
}

#[test]
fn multiple_forward_usages_migrate_both_indexes() {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new()).unwrap();
    for _ in 0..8 {
        store.create_point().unwrap();
    }
    for (index, source, target) in [
        (1, 9, 2),
        (2, 3, 9),
        (3, 9, 9),
        (4, 9, 100),
        (5, 100, 9),
        (6, 9, 101),
        (7, 102, 9),
    ] {
        store.update(index, source, target).unwrap();
    }
    assert_indexes(&store);
    assert_eq!(store.create().unwrap(), 9);
    assert_indexes(&store);
    store.update(9, 9, 8).unwrap();
    assert_indexes(&store);
    // Detaching a migrated node must use the same tree that now serves reads.
    store.update(1, 2, 9).unwrap();
    store.delete(3).unwrap();
    assert_indexes(&store);
    let any = store.constants().any;
    assert_eq!(store.count_by([any, 100, 999]), 0);
    assert_eq!(store.count_by([any, 100, 9]), 1);
}

#[test]
fn two_forward_endpoints_can_be_created_separately() {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new()).unwrap();
    let link = store.create_link(2, 3).unwrap();
    assert_indexes(&store);
    assert_eq!(store.create_point().unwrap(), 2);
    assert_indexes(&store);
    assert_eq!(store.create_point().unwrap(), 3);
    assert_indexes(&store);
    store.delete(link).unwrap();
    assert_indexes(&store);
}

fn delete_and_recreate(mut store: impl Doublets<usize>, referenced: usize) {
    for _ in 0..6 {
        store.create_point().unwrap();
    }
    // Keep pairs unique when another link uses both referenced endpoints.
    store.update(referenced, referenced, 4).unwrap();
    store.update(1, referenced, 2).unwrap();
    store.update(2, 3, referenced).unwrap();
    store.update(3, referenced, referenced).unwrap();
    assert_indexes(&store);
    store.delete(referenced).unwrap();
    assert!(!store.exist(referenced));
    assert_indexes(&store);
    assert_eq!(store.create().unwrap(), referenced);
    store.update(referenced, referenced, 4).unwrap();
    assert_indexes(&store);
    store.update(1, 2, referenced).unwrap();
    store.delete(2).unwrap();
    assert_indexes(&store);
}

#[test]
fn references_survive_tail_deletion_and_recreation() {
    delete_and_recreate(
        split::Store::<usize, _, _>::new(Global::new(), Global::new()).unwrap(),
        6,
    );
}

#[test]
fn references_survive_free_list_reuse() {
    delete_and_recreate(
        split::Store::<usize, _, _>::new(Global::new(), Global::new()).unwrap(),
        5,
    );
}

#[test]
fn migration_preserves_creation_callback_and_read_break() {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new()).unwrap();
    store.create_link(3, 1).unwrap();
    store.create_link(2, 3).unwrap();
    let mut events = Vec::new();
    assert_eq!(
        store
            .create_with(|before, after| {
                events.push((before, after));
                Flow::Break
            })
            .unwrap(),
        Flow::Break
    );
    assert_eq!(events, [(Link::nothing(), Link::new(3, 0, 0))]);
    assert_eq!(store.get_link(3), Some(Link::new(3, 0, 0)));
    assert_indexes(&store);
    let any = store.constants().any;
    for query in [[any, 3, any], [any, any, 3]] {
        let mut calls = 0;
        assert_eq!(
            store.each_by(query, |_| {
                calls += 1;
                Flow::Break
            }),
            Flow::Break
        );
        assert_eq!(calls, 1);
    }
}

fn random_changes(mut store: impl Doublets<usize>, seed: u64) {
    use rand::{rngs::StdRng, Rng, SeedableRng};
    use std::collections::BTreeMap;

    let mut random = StdRng::seed_from_u64(seed);
    let mut model = BTreeMap::new();
    // Fixed seeds, at most 24 live links, and 256 operations keep this bounded.
    for step in 0..256 {
        let operation = random.gen_range(0..4);
        if model.is_empty() || (operation == 0 && model.len() < 24) {
            let index = store.create().unwrap();
            model.insert(index, (0, 0));
        } else {
            let index = *model.keys().nth(random.gen_range(0..model.len())).unwrap();
            if operation == 1 {
                store.delete(index).unwrap();
                model.remove(&index);
            } else {
                let pair = (random.gen_range(1..=32), random.gen_range(1..=32));
                // Raw stores require unique pairs; uniqueness policy is optional.
                if model.values().any(|&value| value == pair) {
                    continue;
                }
                store.update(index, pair.0, pair.1).unwrap();
                model.insert(index, pair);
            }
        }
        assert_eq!(store.count(), model.len(), "seed {seed}, step {step}");
        for (&index, &(source, target)) in &model {
            assert_eq!(
                store.get_link(index),
                Some(Link::new(index, source, target)),
                "seed {seed}, step {step}"
            );
        }
        assert_indexes(&store);
    }
}

#[test]
fn seeded_changes_match_a_plain_map() {
    for seed in [1, 74, 1024, 65535] {
        random_changes(
            split::Store::<usize, _, _>::new(Global::new(), Global::new()).unwrap(),
            seed,
        );
        random_changes(unit::Store::<usize, _>::new(Global::new()).unwrap(), seed);
    }
}
