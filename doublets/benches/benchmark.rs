use criterion::{black_box, criterion_group, criterion_main, Criterion};
use data::Flow::Continue;
use doublets::{split::Store, Doublets, DoubletsExt, Links};
use mem::Global;

fn iter(c: &mut Criterion) {
    let mut store = Store::<usize, _, _>::new(Global::new(), Global::new()).unwrap();

    for _ in 0..1_000_000 {
        store.create_point().unwrap();
    }
}

criterion_group!(benches, iter);
criterion_main!(benches);