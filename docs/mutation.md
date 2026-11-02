# Checked and unchecked mutation

The memory stores keep the mutation mechanism separate from contract checks.
The mechanism is two `unsafe` primitives. Each check is an explicit adapter or
decorator that you can select or omit. Safe Rust APIs stay memory-safe. You can
skip a check only by calling an `unsafe` primitive and taking responsibility
for its preconditions.

| Route | Requested address check | Missing address | Other contracts |
| --- | --- | --- | --- |
| `store.update*` / `store.delete*` | One existence lookup (via `CheckedMutation`) | `Error::NotExists`, store unchanged, no callback | Not checked |
| `CheckedMutation::new(&mut store)` | One existence lookup | `Error::NotExists`, store unchanged, no callback | Not checked |
| `.with_inner_reference_existence_validation()` | Request and endpoints | `Error::NotExists` (error policy) | Endpoint references |
| `.with_non_existent_deletion_resolution()` | One lookup before deleting | `Ok(Flow::Continue)`, no callback (ignore policy) | Inner errors propagate |
| `.with_no_exceptions()` | None | Any inner error becomes `Ok(Flow::Break)` | Swallows *all* errors |
| `unsafe UncheckedMutation::update_unchecked` | None | Caller guarantees it never happens | None |
| `unsafe Release::release_unchecked` | None | Caller guarantees it never happens | See [deletion](deletion.md) |

`UncheckedMutation` and `Release` are in `doublets::mem`. `CheckedMutation` and
the decorators are in `doublets::decorators`.

## Minimal preconditions

`unsafe fn update_unchecked(&mut self, index, source, target, handler) -> Flow`
requires only these conditions:

1. `index` is a live allocated address of this store. It is not null, a service
   constant such as `any`, beyond the allocated range, an unused (free-list)
   slot or an already released address.
2. The store's index structures are consistent with its link data, as the store
   API maintains them. Raw link or index memory has not been written directly.

The new endpoints are unconstrained. They may be zero, the link itself, other
live links, or unallocated addresses (the split layout indexes the latter
externally). Uniqueness, reference existence and usages are separate decorator
policies.

The primitive detaches the link's current outgoing source/target index
memberships, writes the endpoints, attaches the new memberships, and calls
`handler(before, after)` once. `Flow::Break` is returned but does not undo the
update.

Deletion is a composition, not a third primitive. If an endpoint is nonzero,
`update_unchecked(index, 0, 0, ..)` resets it. Then
`release_unchecked(index, ..)` frees the slot. Its preconditions are described
in [deletion](deletion.md#caller-guaranteed-release).

Violating these preconditions is outside the safe API contract. An unused slot
shares memory with the free list, so updating one corrupts allocation state.
Never call the primitives to probe an address. The tests pass them only valid
live addresses and cover invalid addresses through the safe routes.

## Checked convenience route

`Links::update_links` and `Links::delete_links` on both memory stores
delegate to `CheckedMutation`. It verifies exactly one contract: the requested
address is a live link. A violation returns `Error::NotExists(index)` before any
write or callback, so the count, the links, the indexes and the free list are
unchanged. Short `query` or `change` slices read missing parts as null instead of
panicking.

A checked deletion does one lookup, resets nonzero endpoints, releases the
slot, and reports a single `(original, Link::nothing())` event.

```rust
use doublets::{decorators::CheckedMutation, unit, Doublets, Error};
use mem::Global;

# fn main() -> Result<(), Error<u32>> {
let mut store = unit::Store::<u32, _>::new(Global::new())?;
let index = store.create_point()?;
assert!(matches!(store.update(index + 1, 0, 0), Err(Error::NotExists(_))));
let mut checked = CheckedMutation::new(&mut store);
assert_eq!(checked.delete(index)?, index);
assert!(matches!(checked.delete(index), Err(Error::NotExists(i)) if i == index));
# Ok(())
# }
```

## Error, ignore and swallow policies are distinct

- **Error:** the checked route and `InnerReferenceExistenceValidator` return
  `Error::NotExists`. The validator also checks endpoint references. That is a
  separate contract, which the store's own check does not include.
- **Ignore missing deletion:** `NonExistentDeletionResolver` (port of C#
  `LinksNonExistentDeletionResolver`) checks only the requested address before
  forwarding. Deleting a missing address returns `Ok(Flow::Continue)` with no
  callback. Deleting a live address is forwarded unchanged, so `HasUsages`,
  `NonNullContents`, I/O and capacity errors from inner layers still propagate.
  Updates and other operations are not affected. It does not validate any other
  contract, so place it outside the reset and cascade resolvers.
- **Swallow:** `NoExceptionsDecorator` converts every inner error into
  `Ok(Flow::Break)`. Select it only when that is what you want. It is not a
  missing-address policy.

```rust
use doublets::{decorators::DecoratorsExt, unit, Doublets, Error};
use mem::Global;

# fn main() -> Result<(), Error<u32>> {
let mut store = unit::Store::<u32, _>::new(Global::new())?
    .with_usages_validation()
    .with_non_existent_deletion_resolution();
let root = store.create()?;
let usage = store.create_link(root, root)?;
assert!(matches!(store.delete(root), Err(Error::HasUsages(_)))); // not swallowed
store.delete(usage)?;
assert_eq!(store.delete(usage)?, 0); // missing: ignored
# Ok(())
# }
```

## Omitting the checks

If your code already guarantees that an address is live, for example because it
just created the address or holds it in a structure that tracks liveness, call
the primitives directly. That omits the lookup and nothing else: the index
maintenance and the callback still happen. Wrap only the calls whose
preconditions you can justify, and write the justification in a `SAFETY`
comment.

```rust
use data::Flow;
use doublets::{
    mem::{Global, Release, UncheckedMutation},
    unit, Doublets,
};

# fn main() -> Result<(), doublets::Error<u32>> {
let mut store = unit::Store::<u32, _>::new(Global::new())?;
let a = store.create_point()?;
let b = store.create()?;
// SAFETY: `b` was just created by this store and is live; the store API
// maintained every index.
unsafe { store.update_unchecked(b, a, b, &mut |_, _| Flow::Continue) };
assert_eq!(store.search(a, b), Some(b));
// SAFETY: `b` is live; the reset detaches its outgoing index memberships, which
// is the remaining precondition of `release_unchecked`.
unsafe {
    store.update_unchecked(b, 0, 0, &mut |_, _| Flow::Continue);
    store.release_unchecked(b, &mut |_, _| Flow::Continue);
}
assert!(!store.exist(b));
# Ok(())
# }
```

Code that is generic over a store can require `impl UncheckedMutation<T>`. The
safe route is then still available via `CheckedMutation::new(&mut store)`, and
you can stack the policy decorators above it.
