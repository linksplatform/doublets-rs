use rand::Rng;

use data::{Flow, Hybrid, LinkType};
use doublets::{Doublets, Link};

pub fn test_crud<T: LinkType>(store: &mut impl Doublets<T>) {
    let constants = store.constants().clone();

    assert_eq!(store.count(), T::funty(0));

    let address = store.create().unwrap();
    // TODO: expect
    let mut link: Link<T> = store.get_link(address).unwrap();

    assert_eq!(link.index, address);
    assert_eq!(link.source, constants.null);
    assert_eq!(link.target, constants.null);
    assert_eq!(store.count(), T::funty(0));

    store.update(address, address, address).unwrap();

    // TODO: expect
    link = store.get_link(address).unwrap();
    assert_eq!(link.source, address);
    assert_eq!(link.target, address);

    let updated = store
        .update(address, constants.null, constants.null)
        .unwrap();
    assert_eq!(updated, address);
    // TODO: expect
    link = store.get_link(address).unwrap();
    assert_eq!(link.source, constants.null);
    assert_eq!(link.target, constants.null);

    store.delete(address).unwrap();
    assert_eq!(store.count(), T::funty(0));
}

pub fn test_raw_numbers_crud<T: LinkType>(store: &mut impl Doublets<T>) {
    let links = store;

    let constants = links.constants().clone();

    let n106 = T::try_from(106_usize).unwrap();
    let n107 = T::try_from(char::from_u32(107).unwrap() as usize).unwrap();
    let n108 = T::try_from((-108_i32) as usize).unwrap();

    let h106 = Hybrid::external(n106);
    let h107 = Hybrid::new(n107);
    let h108 = Hybrid::new(n108);

    assert_eq!(h106.abs().as_usize(), 106);
    assert_eq!(h107.abs().as_usize(), 107);
    assert_eq!(h108.abs().as_usize(), 108);

    let address1 = links.create().unwrap();
    links
        .update(address1, h106.as_inner(), h108.as_inner())
        .unwrap();

    let link = links.get_link(address1).unwrap();
    assert_eq!(link.source, h106.as_inner());
    assert_eq!(link.target, h108.as_inner());

    let address2 = links.create().unwrap();
    links.update(address2, address1, h108.as_inner()).unwrap();

    let link = links.get_link(address2).unwrap();
    assert_eq!(link.source, address1);
    assert_eq!(link.target, h108.as_inner());

    let address3 = links.create().unwrap();
    links.update(address3, address1, address2).unwrap();

    let link = links.get_link(address3).unwrap();
    assert_eq!(link.source, address1);
    assert_eq!(link.target, address2);

    let any = constants.any;

    let mut result = None;
    links.each_by([any, h106.as_inner(), h108.as_inner()], |link| {
        result = Some(link.index);
        Flow::Break
    });
    assert_eq!(result, Some(address1));

    let mut result = None;
    links.each_by([any, h106.abs(), h107.abs()], |link| {
        result = Some(link.index);
        Flow::Break
    }); // TODO: !!!
    assert_eq!(result, None);

    let updated = links.update(address3, T::funty(0), T::funty(0)).unwrap();
    assert_eq!(updated, address3);

    let link = links.get_link(updated).unwrap();
    assert_eq!(link.source, T::funty(0));
    assert_eq!(link.target, T::funty(0));
    links.delete(updated).unwrap();

    assert_eq!(links.count(), T::try_from(2).unwrap());

    let _continue = links.constants().r#continue;
    let mut result = None;
    links.each(|link| {
        result = Some(link.index);
        Flow::Continue
    });
    assert_eq!(result, Some(address2));
}

pub fn test_random_creations_and_deletions<T: LinkType>(
    store: &mut impl Doublets<T>,
    per_cycle: usize,
) {
    for n in 1..per_cycle {
        let mut created = 0;
        let mut _deleted = 0;
        for _ in 0..n {
            let count = store.count().as_usize();
            let create_point: bool = rand::random();
            if count >= 2 && create_point {
                let address = 1..=count;
                let source = rand::thread_rng().gen_range(address.clone());
                let target = rand::thread_rng().gen_range(address);
                let result = store
                    .get_or_create(T::try_from(source).unwrap(), T::try_from(target).unwrap())
                    .unwrap()
                    .as_usize();

                if result > count {
                    created += 1;
                }
            } else {
                store.create().unwrap();
                created += 1;
            }
            assert_eq!(created, store.count().as_usize());
        }

        store.delete_all().unwrap();

        assert_eq!(store.count(), T::funty(0));
    }
}
