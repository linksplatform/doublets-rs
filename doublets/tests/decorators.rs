//! Behavioural tests for the ported C# decorator layer.
//!
//! Every test exercises one decorator through the public [`DecoratorsExt`]
//! builder, mirroring the scenarios covered by `Platform.Data.Doublets.Tests`.

use data::Flow;
use doublets::{
    decorators::{CascadeResolve, DecoratorsExt, Resolve, Validate},
    mem::Global,
    unit, Doublets, DoubletsExt, Error, Link, Links,
};
use static_assertions::assert_eq_size;

type Store = unit::Store<usize, Global<doublets::mem::parts::LinkPart<usize>>>;

fn store() -> Store {
    unit::Store::<usize, _>::new(Global::new()).expect("in-memory store")
}

// ---------------------------------------------------------------------------
// LinksUniquenessValidator
// ---------------------------------------------------------------------------

#[test]
fn uniqueness_validator_rejects_a_duplicate_doublet() {
    let mut store = store().with_uniqueness(Validate);

    let a = store.create_point().unwrap();
    let b = store.create_point().unwrap();
    let ab = store.create_link(a, b).unwrap();

    let existing = store.create().unwrap();
    let err = store.update(existing, a, b).unwrap_err();

    assert!(
        matches!(err, Error::AlreadyExists(doublet) if doublet.source == a && doublet.target == b)
    );
    assert_eq!(store.search(a, b), Some(ab));
}

#[test]
fn uniqueness_validator_allows_a_unique_doublet() {
    let mut store = store().with_uniqueness(Validate);

    let a = store.create_point().unwrap();
    let b = store.create_point().unwrap();

    assert_eq!(store.create_link(a, b).unwrap(), 3);
    assert_eq!(store.create_link(b, a).unwrap(), 4);
}

// ---------------------------------------------------------------------------
// LinksUniquenessResolver
// ---------------------------------------------------------------------------

#[test]
fn uniqueness_resolver_reuses_the_existing_doublet() {
    let mut store = store().with_uniqueness(Resolve);

    let a = store.create_point().unwrap();
    let b = store.create_point().unwrap();
    let ab = store.create_link(a, b).unwrap();

    assert_eq!(store.create_link(a, b).unwrap(), ab);
    assert_eq!(store.count(), 3);
}

/// Regression test for [#57]: creating the same doublet over and over used to
/// insert duplicate entries into the source/target trees, which then panicked
/// with `attempt to subtract with overflow` on deletion.
///
/// [#57]: https://github.com/linksplatform/doublets-rs/issues/57
#[test]
fn duplicate_creation_does_not_corrupt_the_index() {
    let mut store = store().with_uniqueness(Resolve);

    let a = store.create_point().unwrap();
    let b = store.create_point().unwrap();

    let created: Vec<_> = (0..8).map(|_| store.create_link(a, b).unwrap()).collect();
    let ab = created[0];

    assert!(created.iter().all(|&index| index == ab));
    assert_eq!(store.count(), 3);
    assert_eq!(
        store.iter().collect::<Vec<_>>(),
        vec![Link::point(a), Link::point(b), Link::new(ab, a, b)]
    );

    // The bare store panics here; with the resolver in place there is exactly
    // one entry to remove.
    store.delete(ab).unwrap();
    assert_eq!(store.count(), 2);
    assert_eq!(store.search(a, b), None);
}

// ---------------------------------------------------------------------------
// LinksCascadeUniquenessAndUsagesResolver
// ---------------------------------------------------------------------------

#[test]
fn cascade_uniqueness_resolver_merges_usages_into_the_survivor() {
    let mut store = store().with_uniqueness(CascadeResolve);

    let a = store.create_point().unwrap();
    let b = store.create_point().unwrap();
    let ab = store.create_link(a, b).unwrap();
    let ba = store.create_link(b, a).unwrap();
    let user = store.create_link(ba, b).unwrap();

    // Turning `ba` into `(a, b)` collides with `ab`: `ba` is dropped and every
    // link that referenced it is re-pointed at `ab`.
    assert_eq!(store.update(ba, a, b).unwrap(), ab);
    assert!(!store.exist(ba));
    assert_eq!(store.try_get_link(user).unwrap(), Link::new(user, ab, b));
}

// ---------------------------------------------------------------------------
// LinksUsagesValidator
// ---------------------------------------------------------------------------

#[test]
fn usages_validator_rejects_changes_to_a_used_link() {
    let mut store = store().with_usages_validation();

    let a = store.create_point().unwrap();
    let b = store.create_point().unwrap();
    let ab = store.create_link(a, b).unwrap();

    assert!(matches!(store.delete(a).unwrap_err(), Error::HasUsages(_)));
    assert!(matches!(
        store.update(a, b, b).unwrap_err(),
        Error::HasUsages(_)
    ));

    // Once the only usage is gone the link may be changed again.
    store.delete(ab).unwrap();
    store.delete(a).unwrap();
    assert_eq!(store.count(), 1);
}

// ---------------------------------------------------------------------------
// LinksCascadeUsagesResolver
// ---------------------------------------------------------------------------

#[test]
fn cascade_usages_resolver_deletes_dependents_first() {
    let mut store = store().with_cascade_usages_resolution();

    let a = store.create_point().unwrap();
    let b = store.create_point().unwrap();
    let ab = store.create_link(a, b).unwrap();
    let _nested = store.create_link(ab, ab).unwrap();

    store.delete(a).unwrap();

    assert_eq!(store.iter().collect::<Vec<_>>(), vec![Link::point(b)]);
}

#[test]
fn cascade_usages_resolver_terminates_on_a_reference_cycle() {
    let mut store = store();
    let a = store.create_point().unwrap();
    let b = store.create_point().unwrap();
    store.update(a, a, b).unwrap();
    store.update(b, b, a).unwrap();

    let mut store = store.with_cascade_usages_resolution();
    store.delete(a).unwrap();

    assert_eq!(store.count(), 0);
}

// ---------------------------------------------------------------------------
// LinksInnerReferenceExistenceValidator
// ---------------------------------------------------------------------------

#[test]
fn inner_reference_existence_validator_rejects_dangling_references() {
    let mut store = store().with_inner_reference_existence_validation();

    let a = store.create_point().unwrap();
    let missing = a + 10;

    assert!(matches!(
        store.update(a, missing, a).unwrap_err(),
        Error::NotExists(index) if index == missing
    ));
    assert!(matches!(
        store.delete(missing).unwrap_err(),
        Error::NotExists(index) if index == missing
    ));
}

#[test]
fn inner_reference_existence_validator_reports_each_failures_out_of_band() {
    let store = store().with_inner_reference_existence_validation();
    let any = store.constants().any;
    let missing = 10;

    let mut seen = 0_usize;
    let flow = store.each_links(&[any, missing, any], &mut |_| {
        seen += 1;
        Flow::Continue
    });

    assert_eq!(flow, Flow::Continue);
    assert_eq!(seen, 0);
    assert!(matches!(
        store.try_each_links(&[any, missing, any], &mut |_| Flow::Continue),
        Err(Error::NotExists(index)) if index == missing
    ));
}

// ---------------------------------------------------------------------------
// LinksItselfConstantToSelfReferenceResolver
// ---------------------------------------------------------------------------

#[test]
fn itself_constant_resolves_to_the_updated_link() {
    let mut store = store().with_itself_constant_resolution();
    let itself = store.constants().itself;

    let x = store.create().unwrap();
    store.update(x, itself, itself).unwrap();

    assert_eq!(store.try_get_link(x).unwrap(), Link::point(x));
}

#[test]
fn itself_constant_is_never_matched_by_a_query() {
    let mut store = store().with_itself_constant_resolution();
    let (any, itself) = (store.constants().any, store.constants().itself);

    let x = store.create().unwrap();
    store.update(x, itself, itself).unwrap();

    let mut seen = 0_usize;
    let flow = store.each_links(&[any, itself, any], &mut |_| {
        seen += 1;
        Flow::Continue
    });

    assert_eq!(flow, Flow::Continue);
    assert_eq!(seen, 0);
}

// ---------------------------------------------------------------------------
// LinksNullConstantToSelfReferenceResolver
// ---------------------------------------------------------------------------

#[test]
fn null_constant_creation_yields_a_point() {
    let mut store = store().with_null_constant_resolution();

    let x = store.create().unwrap();

    assert_eq!(store.try_get_link(x).unwrap(), Link::point(x));
}

#[test]
fn null_constant_resolves_to_the_updated_link() {
    let mut store = store().with_null_constant_resolution();
    let null = store.constants().null;

    let a = store.create().unwrap();
    let b = store.create().unwrap();
    store.update(b, null, a).unwrap();

    assert_eq!(store.try_get_link(b).unwrap(), Link::new(b, b, a));
}

// ---------------------------------------------------------------------------
// LinksNonExistentDependenciesCreator
// ---------------------------------------------------------------------------

#[test]
fn non_existent_dependencies_are_created_on_demand() {
    let mut store = store().with_non_existent_dependencies_creation();

    let a = store.create().unwrap();
    let missing = a + 2;
    store.update(a, missing, missing).unwrap();

    assert!(store.exist(missing));
    assert_eq!(
        store.try_get_link(a).unwrap(),
        Link::new(a, missing, missing)
    );
}

// ---------------------------------------------------------------------------
// NonNullContentsLinkDeletionResolver
// ---------------------------------------------------------------------------

#[test]
fn non_null_contents_are_reset_before_deletion() {
    let mut store = store().with_non_null_contents_deletion_resolution();
    let a = store.create_point().unwrap();

    let mut seen = Vec::new();
    store
        .delete_with(a, |before, after| {
            seen.push((before, after));
            Flow::Continue
        })
        .unwrap();

    assert_eq!(
        seen,
        vec![
            (Link::point(a), Link::new(a, 0, 0)),
            (Link::new(a, 0, 0), Link::nothing()),
        ]
    );
    assert_eq!(store.count(), 0);
}

// ---------------------------------------------------------------------------
// LoggingDecorator
// ---------------------------------------------------------------------------

#[test]
fn logging_decorator_records_every_mutation() {
    let mut store = store().with_logging(Vec::new());

    let a = store.create_point().unwrap();
    store.delete(a).unwrap();

    let (_, log) = store.into_inner();
    let log = String::from_utf8(log).unwrap();

    assert_eq!(
        log.lines().collect::<Vec<_>>(),
        [
            "Create. Before: (0->0). After: (1: 0->0)",
            "Update. Before: (1: 0->0). After: (1: 1->1)",
            "Delete. Before: (1: 1->1). After: (0->0)",
        ]
    );
}

// ---------------------------------------------------------------------------
// NoExceptionsDecorator
// ---------------------------------------------------------------------------

#[test]
fn no_exceptions_decorator_turns_errors_into_a_break() {
    let mut store = store().with_uniqueness(Validate).with_no_exceptions();

    let a = store.create_point().unwrap();
    let b = store.create_point().unwrap();
    let ab = store.create_link(a, b).unwrap();
    let spare = store.create().unwrap();

    let flow = store
        .update_links(&[spare], &[spare, a, b], &mut |_, _| Flow::Continue)
        .expect("the underlying `AlreadyExists` must be swallowed");

    assert_eq!(flow, Flow::Break);
    assert_eq!(store.search(a, b), Some(ab));
}

// ---------------------------------------------------------------------------
// Composition
// ---------------------------------------------------------------------------

#[test]
fn automatic_resolution_stack_behaves_like_get_or_create() {
    let mut store = store().with_automatic_uniqueness_and_usages_resolution();

    let a = store.create_point().unwrap();
    let b = store.create_point().unwrap();
    let ab = store.create_link(a, b).unwrap();

    assert_eq!(store.create_link(a, b).unwrap(), ab);
    assert_eq!(store.count(), 3);

    // Deleting `a` cascades through `ab`.
    store.delete(a).unwrap();
    assert_eq!(store.iter().collect::<Vec<_>>(), vec![Link::point(b)]);
}

#[test]
fn decorators_add_no_state() {
    assert_eq_size!(Store, <Validate as doublets::decorators::UniquenessPolicy>::Decorator<usize, Store>);
    assert_eq_size!(Store, <Resolve as doublets::decorators::UniquenessPolicy>::Decorator<usize, Store>);
    assert_eq_size!(
        Store,
        doublets::decorators::AutomaticUniquenessAndUsagesResolution<usize, Store>
    );
}
