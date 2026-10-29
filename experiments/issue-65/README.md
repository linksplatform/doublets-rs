# Issue #65 investigation

`delete_all` used the live link count as the address to delete. With three points
and address 2 deleted, it immediately returned `NotExists(2)`. With address 1
deleted, it deleted point 2, then returned `NotExists(1)`, leaving point 3.

The regression suite reproduced both failures on the unit and split stores before
the fix. All 12 tests failed against the original implementation. Run it with:

```sh
cargo test -p doublets --test delete_all --all-features -- --test-threads=1
```

The tests cover both reported fixtures, all 32 subsets of five points, empty and
single-link stores, repeated deletion, store reuse, dependent-before-dependency
deletion, cascades that remove queued addresses, and propagation of real deletion
errors. Every fixture uses at most five links.

The fix snapshots the actual indices through the existing iterator and deletes
them in reverse enumeration order, following `delete_query_with`. An existence
check skips queued indices already removed by a cascade. The shared trait change
applies to both backends and decorators without changing their storage code.

## C# comparison requested in the issue comment

Source audit of `linksplatform/Data.Doublets` at commit
[`799502ce9275edc80d9fbc1953ff8892fe0eca19`](https://github.com/linksplatform/Data.Doublets/tree/799502ce9275edc80d9fbc1953ff8892fe0eca19):

- [`ILinksExtensions.DeleteAll`, lines 164–175](https://github.com/linksplatform/Data.Doublets/blob/799502ce9275edc80d9fbc1953ff8892fe0eca19/csharp/Platform.Data.Doublets/ILinksExtensions.cs#L164-L175)
  starts from `links.Count()` and passes that value to `links.Delete(i)`. Thus it
  shares the assumption that the count is an existing address. In the middle-hole
  fixture, its first requested deletion is the missing address 2. This conclusion
  follows from the source; the C# library was not executed during this investigation.
- [`ILinksExtensions.DeleteByQuery`, lines 1081–1090](https://github.com/linksplatform/Data.Doublets/blob/799502ce9275edc80d9fbc1953ff8892fe0eca19/csharp/Platform.Data.Doublets/ILinksExtensions.cs#L1081-L1090)
  collects actual indices with `Each` before deleting them. Rust's existing
  `delete_query_with` already uses this snapshot pattern and reverses the order.
- [`ILinksExtensions.DeleteAllUsages`, lines 1040–1059](https://github.com/linksplatform/Data.Doublets/blob/799502ce9275edc80d9fbc1953ff8892fe0eca19/csharp/Platform.Data.Doublets/ILinksExtensions.cs#L1040-L1059)
  checks existence before deleting collected usages, accounting for links removed
  by earlier cascading operations. `delete_all` retains this behavior with its
  own snapshot.
