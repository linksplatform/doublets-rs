# C# comparison for issue #74

Review request: [PR comment 6464125763](https://github.com/linksplatform/doublets-rs/pull/75#issuecomment-6464125763).

The C# implementation returns link 2 for `(3, 2)` both before and after creating
ID 3. Its target-reference variant `(2, 3)` also works. The Rust fix preserves
these results without rejecting references to missing IDs.

## Reproduce the comparison

```sh
bash experiments/issue-74/compare-csharp.sh
```

The script builds the unmodified C# library from `Data.Doublets` revision
[`f8c521e539935e670f4f484cf451857d2bd3e693`](https://github.com/linksplatform/Data.Doublets/tree/f8c521e539935e670f4f484cf451857d2bd3e693)
(current main when reviewed). It uses .NET 8 as the compilation target. This
review used SDK 10.0.112 and runtime roll-forward to .NET 10. The source checkout
and build transcript are under `target/issue-74-review/`; the small result log
is [csharp/results.log](csharp/results.log). Run this optional comparison
separately from the Rust checks; CI does not need a .NET dependency.

The probe runs 12 finite cases: three store configurations, two endpoint
directions, and external-reference support both disabled and enabled. It checks
that ID 3 is initially absent, creates `(2: 3 2)` or `(2: 2 3)`, and compares
exact and wildcard queries and counts against a complete scan. It checks again
after adding another reference to ID 3 and after creating ID 3. An absent pair
must count zero even when its source has other usages. It also checks link 2's
stored endpoints. All 12 cases pass.

The configurations are `UnitedMemoryLinks<ulong>`, `SplitMemoryLinks<ulong>` with
its default source linked list, and split with a source tree (the configuration
closest to Rust). Each store uses a 4,096-byte reservation step, and ID 3 fits
within the reserved memory before it is created. The script uses one build
worker and a 768 MiB managed-heap limit. This verifies the issue's small example;
it does not establish C# support for arbitrary IDs beyond reserved memory or
full equivalence between every C# and Rust operation.

## How C# stores the reference

The established names in C# are **united**, **split**, **internal references**,
and **external references**. Rust exposes the united store as `unit::Store` and
the split store as `split::Store`.

In C#'s united store, an update adds the link to two store-wide trees: one for
source searches and one for target searches. Those trees contain the written
endpoint values, so searching for `3` does not depend on whether link 3 exists.
See
[`UnitedMemoryLinksBase.Update`](https://github.com/linksplatform/Data.Doublets/blob/f8c521e539935e670f4f484cf451857d2bd3e693/csharp/Platform.Data.Doublets/Memory/United/Generic/UnitedMemoryLinksBase.cs#L473)
and
[`Count`](https://github.com/linksplatform/Data.Doublets/blob/f8c521e539935e670f4f484cf451857d2bd3e693/csharp/Platform.Data.Doublets/Memory/United/Generic/UnitedMemoryLinksBase.cs#L208).
Rust's united store follows the same broad design.

In C#'s split store, `ExternalReferencesRange` chooses the index. When the
external range is enabled, an endpoint inside that range uses a store-wide
external tree. An endpoint outside it uses an internal index rooted at that
endpoint's slot. With no external range, endpoints use internal indexes.
Allocation status does not make a value enter or leave the configured range.
For example, 3 uses its internal index even before link 3 is created, provided
the memory for that slot is reserved. Creating link 3 keeps that index available.
See
[`SplitMemoryLinksBase.Update`](https://github.com/linksplatform/Data.Doublets/blob/f8c521e539935e670f4f484cf451857d2bd3e693/csharp/Platform.Data.Doublets/Memory/Split/Generic/SplitMemoryLinksBase.cs#L613),
[`Each`](https://github.com/linksplatform/Data.Doublets/blob/f8c521e539935e670f4f484cf451857d2bd3e693/csharp/Platform.Data.Doublets/Memory/Split/Generic/SplitMemoryLinksBase.cs#L434),
and
[`Create`](https://github.com/linksplatform/Data.Doublets/blob/f8c521e539935e670f4f484cf451857d2bd3e693/csharp/Platform.Data.Doublets/Memory/Split/Generic/SplitMemoryLinksBase.cs#L694).
Its exact-pair
[`Count`](https://github.com/linksplatform/Data.Doublets/blob/f8c521e539935e670f4f484cf451857d2bd3e693/csharp/Platform.Data.Doublets/Memory/Split/Generic/SplitMemoryLinksBase.cs#L298)
also searches for both endpoints; only a wildcard target uses a source-only
count. The Rust count correction follows that rule.

## Why the Rust repair differs internally

Rust's existing `split::Store::is_virtual` classifies an absent ID as external
and a live ID as internal. This differs from C#'s fixed range. A reference to 3
therefore starts in Rust's external index and needs to move to its internal
index when link 3 is created. The previous resolver already intended to do this.
It called the public query path after allocation, which searched the new empty
internal index and missed the old external entries.

The repair uses the existing external tree's `each_usages` operation directly,
collects the referring link IDs, then detaches and attaches those entries with
the existing tree operations. Source and target passes are separate because a
link referring to 3 through both endpoints needs an entry in both indexes.
Collecting before editing keeps traversal from following a tree being changed.
Both passes finish before the creation callback.

This keeps the Rust implementation small: two explicit passes in the existing
resolver, plus a corrected wildcard condition in `count_links`. Changing Rust
to C#'s fixed-range classification would be a broader storage/index redesign;
it is unnecessary to preserve the behavior requested here. No data-format,
ID, endpoint, or reference-validity rule changes. The entries move between
indexes **inside the same split store**. There is no conversion between united
and split stores in this PR. Optional diagnostics use
`DOUBLETS_TRACE_SPLIT_INDEXES=1` and are disabled by default.
