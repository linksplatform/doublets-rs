//! Run via: rustc --edition=2021 experiments/issue-61/split_reuse.rs --extern doublets=target/debug/libdoublets.rlib -L dependency=target/debug/deps -o /tmp/issue-61-split-reuse
use doublets::{mem::Global, split, Doublets, DoubletsExt};
fn main() {
    let mut raw = split::Store::<usize, _, _>::new(Global::new(), Global::new()).unwrap();
    let a = raw.create_point().unwrap();
    let b = raw.create_point().unwrap();
    let ab = raw.create_link(a, b).unwrap();
    let gap = raw.create_point().unwrap();
    let last = raw.create_point().unwrap();
    raw.delete(gap).unwrap();
    eprintln!("initial {:?}", raw.iter().collect::<Vec<_>>());
    raw.delete(ab).unwrap();
    eprintln!("deleted ab {:?}", raw.iter().collect::<Vec<_>>());
    raw.delete(last).unwrap();
    eprintln!("deleted last {:?}", raw.iter().collect::<Vec<_>>());
    raw.update(a, b, a).unwrap();
    eprintln!("updated a {:?}", raw.iter().collect::<Vec<_>>());
    let new = raw.create().unwrap();
    eprintln!("created {new} {:?}", raw.get_link(new));
    raw.update(new, new, new).unwrap();
    eprintln!("made point {:?}", raw.iter().collect::<Vec<_>>());
    let new = raw.create_link(a, b).unwrap();
    eprintln!("created pair {new} {:?}", raw.iter().collect::<Vec<_>>());
    for index in 1..=4 {
        eprintln!("index {index}: {:?}", raw.get_index_part(index));
    }
    for link in raw.iter().collect::<Vec<_>>() {
        eprintln!("reset {}", link.index);
        raw.update(link.index, 0, 0).unwrap();
        eprintln!("reset done {:?}", raw.iter().collect::<Vec<_>>());
    }
}
