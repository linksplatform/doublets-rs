//! References to future IDs stay searchable when those IDs are created.
//! Run with `cargo run -p doublets --example forward_references`.

use doublets::{data::Flow, mem::Global, split, unit, Doublets};

fn run(name: &str, mut store: impl Doublets<usize>) {
    store.create_point().unwrap();
    let b = store.create_point().unwrap();
    store.update(b, 3, b).unwrap();
    let c = store.create_point().unwrap();
    let any = store.constants().any;
    let mut found = Vec::new();
    store.each_by([any, c, b], |link| {
        found.push(link.index);
        Flow::Continue
    });
    println!(
        "{name}: each_by([any, {c}, {b}]) -> {found:?}, get_link({b}) -> {:?}",
        store.get_link(b)
    );
    assert_eq!(found, [b]);
}

fn main() {
    run("unit", unit::Store::<usize, _>::new(Global::new()).unwrap());
    run(
        "split",
        split::Store::<usize, _, _>::new(Global::new(), Global::new()).unwrap(),
    );
}
