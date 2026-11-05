//! External Git dependency smoke check for issue 19; run via verify.sh.
use doublets::{mem::Global, split, unit, Doublets, DoubletsExt, Link};

fn verify(name: &str, mut store: impl Doublets<usize>) {
    let first = store.create_point().unwrap();
    let second = store.create_point().unwrap();
    let edge = store.create_link(first, second).unwrap();
    let expected = vec![
        Link::point(first),
        Link::point(second),
        Link::new(edge, first, second),
    ];
    assert_eq!(store.iter().len(), expected.len());
    assert_eq!(store.iter().collect::<Vec<_>>(), expected);
    assert_eq!(store.iter().next_back(), expected.last().cloned());
    let query = [store.constants().any, first, second];
    assert_eq!(
        store.each_iter(query).collect::<Vec<_>>(),
        vec![Link::new(edge, first, second)]
    );
    #[cfg(feature = "full")]
    {
        assert_eq!(store.iter_small().collect::<Vec<_>>(), expected);
        assert_eq!(
            store.each_iter_small(query).collect::<Vec<_>>(),
            vec![Link::new(edge, first, second)]
        );
        // Construct both parallel iterators without starting a thread pool.
        let _ = store.par_iter();
        let _ = store.par_each_iter(query);
    }
    store.update(edge, second, first).unwrap();
    assert_eq!(store.get_link(edge), Some(Link::new(edge, second, first)));
    assert_eq!(store.delete(edge).unwrap(), edge);
    store.delete_all().unwrap();
    assert_eq!(store.count(), 0);
    println!("{name}: Git dependency CRUD and iterators passed");
}

fn main() {
    verify("unit", unit::Store::<usize, _>::new(Global::new()).unwrap());
    verify(
        "split",
        split::Store::<usize, _, _>::new(Global::new(), Global::new()).unwrap(),
    );
}
