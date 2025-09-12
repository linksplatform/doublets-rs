use criterion::{criterion_group, criterion_main, Criterion};
use doublets_async::{AsyncDoublets, MemoryDoublets};
use std::time::Duration;

fn bench_async_doublets_operations(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    let mut group = c.benchmark_group("async_doublets");
    group.measurement_time(Duration::from_secs(10));
    
    // Benchmark count operations
    group.bench_function("count_empty", |b| {
        let doublets = MemoryDoublets::<usize>::new();
        b.to_async(&rt).iter(|| async {
            doublets.count(None).await
        })
    });
    
    // Benchmark create operations
    group.bench_function("create_links", |b| {
        let doublets = MemoryDoublets::<usize>::new();
        let mut counter = 0;
        b.to_async(&rt).iter(|| async {
            counter += 1;
            doublets.create(Some(&[counter, counter + 1]), None).await
        })
    });
    
    // Benchmark combined operations (create + count)
    group.bench_function("create_and_count", |b| {
        let doublets = MemoryDoublets::<usize>::new();
        let mut counter = 0;
        b.to_async(&rt).iter(|| async {
            counter += 1;
            let _id = doublets.create(Some(&[counter, counter + 1]), None).await;
            doublets.count(None).await
        })
    });
    
    // Benchmark iteration over many links
    group.bench_function("each_1000_links", |b| {
        let doublets = MemoryDoublets::<usize>::new();
        rt.block_on(async {
            for i in 1..=1000 {
                doublets.create(Some(&[i, i + 1]), None).await;
            }
        });
        
        b.to_async(&rt).iter(|| async {
            let handler = |_: &[usize]| true;
            doublets.each(None, Some(&handler)).await
        })
    });
    
    group.finish();
}

// Benchmark to show the zero-cost abstraction - the futures should be small
fn bench_future_size(c: &mut Criterion) {
    let doublets = MemoryDoublets::<usize>::new();
    
    c.bench_function("future_creation_overhead", |b| {
        b.iter(|| {
            // Just create the future without awaiting to measure creation overhead
            let _future = doublets.count(None);
            // In traditional async-trait, this would allocate a Box<dyn Future>
            // With GATs and TAIT, this should be zero-cost
        })
    });
}

criterion_group!(benches, bench_async_doublets_operations, bench_future_size);
criterion_main!(benches);