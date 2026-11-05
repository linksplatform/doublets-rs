---
bump: minor
---

### Added

- Add `mem::UncheckedMutation` with the minimal `unsafe fn update_unchecked` endpoint-update primitive for both memory layouts, documenting its preconditions (live address, consistent indexes). Combined with `Release::release_unchecked`, it gives callers who guarantee those preconditions a mutation and deletion route with no contract validation.
- Add `decorators::CheckedMutation`, an explicit adapter that rejects a missing requested address with `Error::NotExists` before any write or callback.
- Add `decorators::NonExistentDeletionResolver` (`with_non_existent_deletion_resolution`), the ignore-missing deletion policy ported from C# `LinksNonExistentDeletionResolver`. It forwards deletions of live links unchanged, so unrelated errors still propagate.
- Add the `docs/mutation.md` guide and the `unchecked_mutation` example.

### Changed

- The stores' safe `update_links`/`delete_links` now delegate their existence check to `CheckedMutation`. A checked deletion performs one lookup instead of four. A safe update with a short query or change slice now treats the missing parts as null instead of panicking on indexing.
