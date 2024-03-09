use {
    platform_trees::{trees::Node, Leaf, NoRecur, Tree},
    std::{
        mem,
        num::NonZeroUsize,
        ops::Range,
        time::{Duration, Instant},
    },
};

struct Store<T>(Vec<Node<T>>);

impl<T> Store<T> {
    pub fn new(len: usize) -> Self {
        Self((0..len).map(|_| Default::default()).collect())
    }

    pub fn reset(&mut self) {
        self.0.fill_with(|| Default::default());
    }
}

impl<T: Leaf> Tree<T> for Store<T> {
    #[inline]
    fn ptr_range(&self) -> Range<*const u8> {
        // Safety: pointer values do not change during a cast
        // it works like box coercions
        unsafe { mem::transmute(self.0.as_ptr_range()) }
    }

    #[inline]
    fn get(&self, idx: T) -> Option<Node<T>> {
        let Node { size, left, right } = self.0.get(idx.addr()).copied()?;
        Some(Node { size: Leaf::addr(size), left, right })
    }

    #[inline]
    fn set(&mut self, idx: T, node: Node<T>) {
        let Node { size, left, right } = &mut self.0[idx.addr()];
        *size = node.size;
        *left = node.left;
        *right = node.right;
    }

    #[inline]
    fn left_mut(&mut self, idx: T) -> Option<&mut T> {
        self.0.get_mut(idx.addr())?.left.as_mut()
    }

    #[inline]
    fn right_mut(&mut self, idx: T) -> Option<&mut T> {
        self.0.get_mut(idx.addr())?.right.as_mut()
    }

    #[inline]
    fn is_left_of(&self, first: T, second: T) -> bool {
        first.addr() < second.addr()
    }

    #[inline]
    fn is_right_of(&self, first: T, second: T) -> bool {
        first.addr() > second.addr()
    }
}

unsafe impl<T: Leaf> NoRecur<T> for Store<T> {}

use criterion::{criterion_group, criterion_main, Bencher, Criterion, Throughput};

pub fn bench(c: &mut Criterion) {
    const MAGIC: usize = 100_000;

    fn inner<T: Leaf>(b: &mut Bencher, from: impl Fn(usize) -> T) {
        let mut place = Store::<T>::new(MAGIC);
        b.iter_custom(|iters| {
            let mut elapsed = Duration::ZERO;

            for _ in 0..iters {
                let instant = Instant::now();
                let mut root = None;
                for i in 1..MAGIC {
                    root = place.attach(root, from(i))
                }
                for i in 1..MAGIC {
                    //root = place.detach(root, from(i));
                }
                elapsed += instant.elapsed();

                place.reset(); // large drop
            }

            elapsed
        });
    }

    c.benchmark_group("trees")
        .throughput(Throughput::Elements(MAGIC as u64))
        //.bench_function("usize", |b| {
        //    inner(b, |x| x);
        //})
        .bench_function("non-zero", |b| unsafe { inner(b, |x| NonZeroUsize::new_unchecked(x)) });
}

criterion_group!(benches, bench);
criterion_main!(benches);
