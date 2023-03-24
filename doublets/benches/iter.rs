use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use data::Flow::Continue;
use doublets::{split::Store, Doublets, DoubletsExt, Links};
use mem::Global;

fn iter(c: &mut Criterion) {
    let mut store = Store::<usize, _, _>::new(Global::new(), Global::new()).unwrap();
    let _any = store.constants().any;

    for _ in 0..1_000_000 {
        store.create_point().unwrap();
    }

    (1..=1_000_000).filter(|x| x % 172 == 0).for_each(|x| {
        store.delete(x).unwrap();
    });

    c.bench_function("iter", |b| {
        b.iter(|| {
            store.iter().for_each(|item| {
                black_box(item);
            })
        });
    });
    c.bench_function("each", |b| {
        b.iter(|| {
            store.each(|link| {
                black_box(link);
                Continue
            });
        });
    });
    c.bench_function("each_with_vec", |b| {
        b.iter(|| {
            let mut vec = Vec::with_capacity(store.count());
            store.each(|link| {
                vec.push(black_box(link));
                Continue
            });
            black_box(vec);
        });
    });
}

fn create_point(c: &mut Criterion) {
    let mut store = Store::<usize, _, _>::new(Global::new(), Global::new()).unwrap();

    let n = 1_000_000;

    let mut group = c.benchmark_group("create_point");
    group.throughput(Throughput::Elements(n));

    group.bench_function("create_point", |b| {
        b.iter(|| {
            for _ in 0..n {
                store.create_point().unwrap();
            }
        });
    });
}

criterion_group!(benches, iter, create_point);
criterion_main!(benches);
