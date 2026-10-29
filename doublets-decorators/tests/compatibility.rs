use doublets::{mem, unit, Doublets, Error};
use doublets_decorators::{
    CascadeUniqueResolver, CascadeUsagesResolver, NonNullDeletionResolver, UniqueResolver,
    UniqueValidator, UsagesValidator,
};

type Store = unit::Store<usize, mem::Global<unit::LinkPart<usize>>>;
fn store() -> Store {
    Store::new(mem::Global::new()).unwrap()
}

#[test]
fn legacy_names_compose_with_current_stores() {
    let mut links = UniqueResolver::new(store());
    let point = links.create_point().unwrap();
    assert_eq!(links.create_link(point, point).unwrap(), point);

    let mut links = UniqueValidator::new(store());
    let point = links.create_point().unwrap();
    assert!(matches!(
        links.create_link(point, point),
        Err(Error::AlreadyExists(_))
    ));

    let mut links = UsagesValidator::new(store());
    let point = links.create_point().unwrap();
    links.create_link(point, 0).unwrap();
    assert!(matches!(links.delete(point), Err(Error::HasUsages(_))));

    let mut links = NonNullDeletionResolver::new(CascadeUsagesResolver::new(store()));
    let point = links.create_point().unwrap();
    links.create_link(point, 0).unwrap();
    links.delete(point).unwrap();
    assert_eq!(links.count(), 0);

    let mut links = CascadeUniqueResolver::new(store());
    let point = links.create_point().unwrap();
    assert_eq!(links.create_link(point, point).unwrap(), point);
}
