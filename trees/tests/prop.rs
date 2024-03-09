use {
    platform_trees::{inner, BTree, OldStore},
    std::num::NonZeroUsize,
};

use proptest::prelude::*;

trait QuickTree {
    type Output: BTree<Item = usize>;

    fn new(len: usize) -> Self::Output;
}

macro_rules! quick_impl {
    ($($name:ident | $ty:ty => $expr:expr)*) => {$(
        struct $name;

        impl QuickTree for $name {
            type Output = $ty;

            fn new(len: usize) -> Self::Output {
                ($expr)(len + 1)
            }
        })*
    };
}

quick_impl!(
    Old | OldStore<usize> => |len| OldStore::make(len)
    New | inner::New<usize> => |len| inner::New::make(len)
);

const STRATEGY_LEN: usize = 10;

prop_compose! {
    fn seq_strategy()
        (len in 2..STRATEGY_LEN)
        (len in Just(len), set in prop::collection::hash_set(1..len, 0..STRATEGY_LEN))
    -> (Vec<usize>, usize) {
       (set.into_iter().collect(), len)
    }
}

fn inner<Tree: QuickTree>((vec, len): (Vec<usize>, usize)) {
    let mut store = Tree::new(len);
    let mut root = None;
    for item in &vec {
        store._attach(&mut root, *item);
    }

    for item in &vec {
        assert!(store.is_contains(root.unwrap(), *item));
    }

    for item in vec {
        store._detach(&mut root, item);
        if let Some(root) = root {
            assert!(!store.is_contains(root, item));
        }
    }

    assert!(store.is_empty());
}

use {
    platform_trees::new::{NoRecur, Tree},
    proptest::test_runner::FileFailurePersistence,
};

proptest! {
    #![proptest_config(ProptestConfig {
        failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
        ..Default::default()
    })]

    #[test]
    fn prop_old(seq in seq_strategy()) {
        inner::<Old>(seq)
    }

    #[test]
    fn prop_new(seq in seq_strategy()) {
        inner::<New>(seq)
    }

    #[test]
    fn non_zero(seq in seq_strategy()) {
        let (vec, len) = seq;

        let cast = |x: &usize| NonZeroUsize::new(*x).unwrap();

        let mut store = inner::New::new(len);
        let mut root = None;
        for item in &vec {
            root = store.attach(root, cast(item));
        }

        for item in &vec {
            assert!(store.is_contains(root.unwrap(), cast(item)));
        }

        for item in &vec {
            root = store.detach(root, cast(item));
            if let Some(root) = root {
                assert!(!store.is_contains(root, cast(item)));
            }
        }

        assert!(store.iter().all(|node| *node == Default::default()));
    }
}
