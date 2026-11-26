//! Issue #94 probe, copied into `doublets/examples/` of a checkout.
//! Links with one internal source must come back in insertion order (the C#
//! default `useLinkedList: true`), and updates must not panic.

use doublets::{data::Flow, mem::Global, split, Doublets, Links};

fn main() {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new()).unwrap();
    let head = store.create_point().unwrap();
    for _ in 0..3 {
        store.create_point().unwrap();
    }
    let a = store.create_link(head, 3).unwrap();
    let b = store.create_link(head, 2).unwrap();
    let c = store.create_link(head, 4).unwrap();
    let any = store.constants().any;
    let mut found = Vec::new();
    store.each_by([any, head, any], |link| {
        found.push(link.index);
        Flow::Continue
    });
    assert_eq!(found, [head, a, b, c]);
    println!("sources of {head} in insertion order: {found:?}");
}
