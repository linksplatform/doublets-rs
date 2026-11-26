//! Stored sizes of the internal sources lists (issue #94).

use super::*;
use crate::DoubletsExt;
use mem::Global;
use rand::{rngs::StdRng, RngExt, SeedableRng};

/// Checks every live head: its stored list size, if it stores one, matches a
/// walk of its list, and the list holds exactly the links with that source.
/// Free slots keep no list or tree metadata.
fn check_sources_lists<MD, MI>(store: &Store<usize, MD, MI>)
where
    MD: RawMem<Item = DataPart<usize>>,
    MI: RawMem<Item = IndexPart<usize>>,
{
    let allocated = store.get_header().allocated;
    let live: Vec<_> = (1..=allocated).filter(|&link| store.exists(link)).collect();
    for &head in &live {
        let mut members = Vec::new();
        store.sources_list.each_usages(head, &mut |link| {
            members.push(link.index);
            Flow::Continue
        });
        members.sort_unstable();
        let expected: Vec<_> = live
            .iter()
            .copied()
            .filter(|&link| store.get_data_part(link).source == head)
            .collect();
        assert_eq!(members, expected, "list of {head}");
        let walked = store.sources_list.count_by_traversal(head);
        assert_eq!(walked, expected.len(), "walk of {head}");
        if store.list_head_stores_size(head) {
            assert_eq!(
                store.sources_list.count_usages(head),
                walked,
                "size of {head}"
            );
        }
        assert_eq!(store.count_list_usages(head), walked, "count of {head}");
    }
    for link in (1..=allocated).filter(|&link| !store.exists(link)) {
        let index = store.get_index_part(link);
        assert_eq!(
            (
                index.root_as_source,
                index.left_as_source,
                index.right_as_source
            ),
            (0, 0, 0),
            "free slot {link}"
        );
        assert_eq!(index.size_as_source, 0, "free slot {link}");
    }
}

#[test]
fn stored_list_sizes_match_their_lists() {
    for seed in [5, 94, 777] {
        let mut store = Store::<usize, _, _>::new(Global::new(), Global::new()).unwrap();
        let mut random = StdRng::seed_from_u64(seed);
        let mut pairs: Vec<(usize, usize)> = Vec::new();
        // 300 operations on at most 24 links; endpoints up to 40 include
        // null, self references, and absent IDs.
        for _ in 0..300 {
            let live: Vec<_> = store.each_iter([store.constants.any]).collect();
            let operation = random.random_range(0..5);
            if live.is_empty() || (operation == 0 && live.len() < 24) {
                store.create().unwrap();
            } else {
                let link = live[random.random_range(0..live.len())].clone();
                if operation == 1 {
                    store.delete(link.index).unwrap();
                } else {
                    let source = if operation == 4 {
                        link.index
                    } else {
                        random.random_range(0..=40)
                    };
                    let pair = (source, random.random_range(0..=40));
                    pairs.clear();
                    pairs.extend(live.iter().map(|link| (link.source, link.target)));
                    if pair == (0, 0) || !pairs.contains(&pair) {
                        store.update(link.index, pair.0, pair.1).unwrap();
                    }
                }
            }
            check_sources_lists(&store);
        }
    }
}

#[test]
fn a_head_in_the_external_tree_does_not_store_its_list_size() {
    let mut store = Store::<usize, _, _>::new(Global::new(), Global::new()).unwrap();
    let head = store.create().unwrap();
    let member = store.create().unwrap();
    store.update(head, 10, 0).unwrap();
    store.update(member, head, 0).unwrap();
    assert!(!store.list_head_stores_size(head));
    // The external sources tree keeps its subtree size in this field.
    assert_eq!(store.get_index_part(head).size_as_source, 1);
    assert_eq!(store.count_list_usages(head), 1);
    store.update(head, 0, 0).unwrap();
    assert!(store.list_head_stores_size(head));
    assert_eq!(store.sources_list.count_usages(head), 1);
    check_sources_lists(&store);
}
