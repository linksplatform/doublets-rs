# Issue #64: deletion helper return values

## Requirements and evidence

| Requirement | Implementation and verification |
| --- | --- |
| `delete` returns the former index | Shared `Doublets::delete_by`; `delete_returns_the_former_index` and `delete_returns_an_uninitialized_links_index` regressions |
| `delete_by` returns the deleted index | Capture `before.index`; `delete_by_returns_the_former_index` regression |
| `rebase_and_delete` returns `new` | Map successful deletion to `new`; regressions with and without usages |
| Automatic decorators behave consistently | All six shared regression fixtures run on bare and automatic unit/split stores; two additional fixtures check cascaded deletion returns the requested root |
| Preserve errors and the same-address shortcut | `deletion_helpers_preserve_errors_and_self_rebase` checks missing addresses and a surviving self-rebased point |
| Compare with Data.Doublets and report an equivalent unreported defect | Source audit below finds that C# already selects the expected return values; no equivalent defect to report |
| Prepare the next release | Patch changelog fragment; the existing release workflow manages the manifest version |

The issue has no required sub-issues. The regression fixtures are finite, using
at most five links per store.

## Reproduction

```sh
CARGO_BUILD_JOBS=1 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
  cargo test -p doublets --test deletion_returns --all-features -- --test-threads=1
```

Before the fix, `delete` and `delete_by` selected `after.index`. Both backends
report `(before, Link::nothing())` after deletion, so the result was always zero.
`rebase_and_delete` forwarded that result. Changing the deletion result alone
would make rebasing return `old`, which is still contrary to its documented
contract; it must explicitly return `new` after successful deletion.

The tests also inspect the remaining graph, updated source/target usages, and
missing-address errors, so a correct return value alone is insufficient to pass.
Existing callback regression suites cover reset/deletion event order and flow.
The pre-fix run reproduced 22 failures returning zero; the four error and
self-rebase cases already passed.

## C# source comparison

Reviewed Data.Doublets `main` at
[`799502ce9275edc80d9fbc1953ff8892fe0eca19`](https://github.com/linksplatform/Data.Doublets/commit/799502ce9275edc80d9fbc1953ff8892fe0eca19)
and the shared Data API at
[`b73d27124d8ba655dc232d0963525f372c6e59c4`](https://github.com/linksplatform/Data/commit/b73d27124d8ba655dc232d0963525f372c6e59c4).
This is a source audit; the C# library was not executed.

- Data.Doublets' [unit deletion](https://github.com/linksplatform/Data.Doublets/blob/799502ce9275edc80d9fbc1953ff8892fe0eca19/csharp/Platform.Data.Doublets/Memory/United/Generic/UnitedMemoryLinksBase.cs#L548-L574)
  and [split deletion](https://github.com/linksplatform/Data.Doublets/blob/799502ce9275edc80d9fbc1953ff8892fe0eca19/csharp/Platform.Data.Doublets/Memory/Split/Generic/SplitMemoryLinksBase.cs#L737-L761)
  both report the former link as `before` and null as `after`.
- The [shared address-returning `Delete` overloads](https://github.com/linksplatform/Data/blob/b73d27124d8ba655dc232d0963525f372c6e59c4/csharp/Platform.Data/ILinksExtensions.cs#L39-L47)
  use `SetFirstFromNonNullFirstListAndReturnTrue`. The [setter implementation](https://github.com/linksplatform/Setters/blob/ccba1fc61a074d280b6d5d9154fb1f8a9c0b2a01/csharp/Platform.Setters/SetterExtensions.cs#L16-L23)
  reads `list1[0]`, i.e. the index from `before`.
- The address-returning [two-argument `MergeAndDelete`](https://github.com/linksplatform/Data.Doublets/blob/799502ce9275edc80d9fbc1953ff8892fe0eca19/csharp/Platform.Data.Doublets/ILinksExtensions.cs#L1232-L1240)
  deletes `oldLinkIndex` then returns `newLinkIndex`, including the same-address
  shortcut. Its callback overload returns flow instead, like Rust's low-level
  write methods.

The Rust fix adopts these existing C# choices. The expected addresses are thus
consistent across the two implementations without changing callback states.
