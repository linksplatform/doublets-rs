# Deletion and release

The memory stores separate endpoint transformation from slot release. Choose the
operation that matches the guarantees supplied by your caller. All checked routes
remain safe Rust; contract checks can only be skipped through an `unsafe` call.

| Route | Endpoint reset | Address/content checks | Callback |
| --- | --- | --- | --- |
| `store.delete*` | Automatic, only when needed | Checked | One original-state deletion event |
| `Release::release` / `release_with` | Caller resets first | Checked | One reset-state deletion event |
| `CheckedRelease::new(&mut store).delete*` | Caller resets first | Checked | One reset-state deletion event |
| Reset resolver over `CheckedRelease` | Automatic, only when needed | Checked | Reset event if needed, then deletion |
| `Release::release_unchecked` | Caller resets first | Caller guarantees contract | One reset-state deletion event |

`Release` is in `doublets::mem`. `CheckedRelease` and
`NonNullContentsLinkDeletionResolver` are in `doublets::decorators`.

## Already detached live links

Create an empty link, or update an existing live link to `(0, 0)` using the normal
store API. That update detaches the outgoing source/target index memberships.
Checked release then frees the slot without any redundant update:

```rust
use doublets::{mem::{Global, Release}, unit, Doublets};

# fn main() -> Result<(), doublets::Error<u32>> {
let mut store = unit::Store::<u32, _>::new(Global::new())?;
let index = store.create_point()?;
store.update(index, 0, 0)?;
assert_eq!(store.release(index)?, index);
assert!(!store.exist(index));
# Ok(())
# }
```

A missing or already released address returns `Error::NotExists`. A live link
whose source or target is nonzero returns `Error::NonNullContents`, including its
original state. Both errors leave the store unchanged and invoke no callback.
Do not zero raw memory instead of detaching through the API: raw memory writes
do not maintain indexes.

## Explicit automatic reset

For a decorator stack whose deletion operation only releases reset links, use
`CheckedRelease`. Omitting the reset layer then omits endpoint transformation:

```rust
use doublets::{
    decorators::{CheckedRelease, DecoratorsExt},
    mem::Global, unit, Doublets, Error,
};

# fn main() -> Result<(), Error<u32>> {
let mut store = unit::Store::<u32, _>::new(Global::new())?;
let index = store.create_point()?;
let mut release = CheckedRelease::new(&mut store);
assert!(matches!(release.delete(index), Err(Error::NonNullContents(_))));
let mut resetting = release.with_non_null_contents_deletion_resolution();
assert_eq!(resetting.delete(index)?, index);
# Ok(())
# }
```

With a callback, this composition reports `(index, source, target)` to
`(index, 0, 0)`, then `(index, 0, 0)` to `Link::nothing()`. An already reset link
reports only the deletion. `Flow::Break` fuses subsequent callbacks; the reset
and release still complete and the operation returns `Break`.

Existing `store.delete*` calls remain a checked convenience route. Internally
they compose this same reset resolver with checked release, retaining the
original single callback `(original, Link::nothing())` and former-index result.
The release primitive itself contains no endpoint update or reset policy.

## Incoming references and index structure

Release permits incoming references and leaves their endpoint values unchanged.
The unit layout indexes endpoints globally. The split layout stores usages of a
live address in per-address internal trees; it must move those entries into the
external trees before freeing the slot. Reusing or reallocating the address
moves them back. This reclassification is structural bookkeeping, even when no
endpoint changes. Neither release nor compatibility deletion deletes usages.

To reject incoming usages, place `UsagesValidator` above the release view. To
delete usages, compose `CascadeUsagesResolver` beneath the reset resolver (the
existing automatic uniqueness/usages stack is another convenience choice).
A self-reference is outgoing content and must first be reset; it is not an
exception to the release contract. Cyclic/shared usage cascades retain their
visited-set behavior and reset/deletion callback sequence.

## Caller-guaranteed release

`unsafe Release::release_unchecked(index, handler)` skips the address and content
checks. The caller must guarantee all of the following:

1. The address is allocated and live, not null, a service constant or a free slot.
2. Both endpoints are zero, with their outgoing index memberships detached.
3. Incoming index entries are consistent with that live address. They may remain
   present: the primitive preserves/reclassifies them rather than requiring a
   separate usage scan or a no-usages policy.

The primitive only updates the chosen index representation as required, manages
the free list and allocated tail, and invokes the deletion callback. It performs
no reset/update and no explicit contract validation. Violating its preconditions
is outside the safe API contract. Tests call it only with valid live reset links.

For both layouts, callback invocation follows release; returning `Break` does
not undo the mutation. A released interior slot can be reused immediately, and
releasing the allocated tail also collapses preceding free slots. The link index
can therefore identify a newly allocated link after reuse.
