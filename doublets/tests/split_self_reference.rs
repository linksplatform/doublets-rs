//! A self reference must remain internal while its link is being updated.

use doublets::{mem::Global, split, Doublets, Links};

#[test]
fn updating_a_self_reference_keeps_it_in_the_internal_target_index() {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new()).unwrap();
    let a = store.create_point().unwrap();
    let b = store.create_point().unwrap();
    store.update(a, b, a).unwrap();
    let any = store.constants().any;
    assert_eq!(store.count_by([any, any, a]), 1);
    // Misclassification used to make this detach an absent internal tree node.
    store.update(a, 0, 0).unwrap();
    assert_eq!(store.count_by([any, any, a]), 0);
}
