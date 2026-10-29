//! Compilation probe for the "decorators fuse away" property of `doublets::decorators`.
//!
//! It exports one pair of `#[no_mangle]` functions per operation: a `bare_*` function
//! that drives a plain `unit::Store`, and a `composed_*` function that drives a deep
//! decorator stack in which **every** layer forwards that operation untouched. Because
//! the decorators are generic newtypes whose forwarding methods are `#[inline]`, an
//! optimising build must emit the same machine code for both.
//!
//! `integration/tests/fusion.rs` builds this binary in release mode and disassembles it.
//! Running it directly just checks that the two stacks agree at runtime:
//!
//! ```console
//! $ cargo run -p integration --release --bin fusion-probe
//! ```

use data::Flow;
use doublets::{
    decorators::{
        CascadeResolve, CascadeUniquenessAndUsagesResolver, CascadeUsagesResolver, DecoratorsExt,
        InnerReferenceExistenceValidator, ItselfConstantToSelfReferenceResolver,
        NonExistentDependenciesCreator, NonNullContentsLinkDeletionResolver,
        NullConstantToSelfReferenceResolver, Resolve, UniquenessResolver, UniquenessValidator,
        UsagesValidator, Validate,
    },
    mem::{parts::LinkPart, Global},
    unit, Doublets, Links,
};

/// A plain store, with no policy attached.
pub type Bare = unit::Store<usize, Global<LinkPart<usize>>>;

fn bare() -> Result<Bare, Box<dyn std::error::Error>> {
    Ok(unit::Store::<usize, _>::new(Global::new())?)
}

/// Nine decorators, none of which touches `create_links` or `count_links`.
pub type ComposedWrite = UniquenessValidator<
    usize,
    UniquenessResolver<
        usize,
        CascadeUniquenessAndUsagesResolver<
            usize,
            UsagesValidator<
                usize,
                CascadeUsagesResolver<
                    usize,
                    InnerReferenceExistenceValidator<
                        usize,
                        ItselfConstantToSelfReferenceResolver<
                            usize,
                            NonExistentDependenciesCreator<
                                usize,
                                NonNullContentsLinkDeletionResolver<usize, Bare>,
                            >,
                        >,
                    >,
                >,
            >,
        >,
    >,
>;

/// Eight decorators, none of which touches `each_links`.
pub type ComposedRead = UniquenessValidator<
    usize,
    UniquenessResolver<
        usize,
        CascadeUniquenessAndUsagesResolver<
            usize,
            UsagesValidator<
                usize,
                CascadeUsagesResolver<
                    usize,
                    NullConstantToSelfReferenceResolver<
                        usize,
                        NonExistentDependenciesCreator<
                            usize,
                            NonNullContentsLinkDeletionResolver<usize, Bare>,
                        >,
                    >,
                >,
            >,
        >,
    >,
>;

/// Builds [`ComposedWrite`]; the annotation asserts that the builder returns exactly the
/// hand-written type above.
fn composed_write(store: Bare) -> ComposedWrite {
    let composed: ComposedWrite = store
        .with_non_null_contents_deletion_resolution()
        .with_non_existent_dependencies_creation()
        .with_itself_constant_resolution()
        .with_inner_reference_existence_validation()
        .with_cascade_usages_resolution()
        .with_usages_validation()
        .with_uniqueness(CascadeResolve)
        .with_uniqueness(Resolve)
        .with_uniqueness(Validate);
    composed
}

/// Builds [`ComposedRead`].
fn composed_read(store: Bare) -> ComposedRead {
    let composed: ComposedRead = store
        .with_non_null_contents_deletion_resolution()
        .with_non_existent_dependencies_creation()
        .with_null_constant_resolution()
        .with_cascade_usages_resolution()
        .with_usages_validation()
        .with_uniqueness(CascadeResolve)
        .with_uniqueness(Resolve)
        .with_uniqueness(Validate);
    composed
}

macro_rules! probe {
    ($bare_fn:ident: $bare_ty:ty, $composed_fn:ident: $composed_ty:ty, |$store:ident| $body:expr) => {
        #[no_mangle]
        #[inline(never)]
        pub extern "C" fn $bare_fn($store: &mut $bare_ty) -> usize {
            $body
        }

        #[no_mangle]
        #[inline(never)]
        pub extern "C" fn $composed_fn($store: &mut $composed_ty) -> usize {
            $body
        }
    };
}

probe!(
    doublets_fusion_bare_create: Bare,
    doublets_fusion_composed_create: ComposedWrite,
    |store| {
        let mut created = 0;
        let flow = store.create_links(&[], &mut |_, after| {
            created = after.index;
            Flow::Continue
        });
        match flow {
            Ok(Flow::Continue) => created,
            Ok(Flow::Break) | Err(_) => usize::MAX,
        }
    }
);

probe!(
    doublets_fusion_bare_count: Bare,
    doublets_fusion_composed_count: ComposedWrite,
    |store| {
        let any = store.constants().any;
        store.count_links(&[any, any, any])
    }
);

probe!(
    doublets_fusion_bare_each: Bare,
    doublets_fusion_composed_each: ComposedRead,
    |store| {
        let any = store.constants().any;
        let mut sum = 0_usize;
        store.each_links(&[any, any, any], &mut |link| {
            sum = sum.wrapping_add(link.index);
            Flow::Continue
        });
        sum
    }
);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut plain = bare()?;
    let mut write = composed_write(bare()?);
    let mut read = composed_read(bare()?);

    for _ in 0..4 {
        let expected = doublets_fusion_bare_create(&mut plain);
        assert_eq!(doublets_fusion_composed_create(&mut write), expected);
        read.create()?;
    }

    assert_eq!(
        doublets_fusion_bare_count(&mut plain),
        doublets_fusion_composed_count(&mut write)
    );
    assert_eq!(
        doublets_fusion_bare_each(&mut plain),
        doublets_fusion_composed_each(&mut read)
    );

    println!("bare and composed agree on {} links", write.count());
    Ok(())
}
