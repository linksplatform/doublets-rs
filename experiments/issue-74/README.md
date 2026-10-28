# Issue #74 investigation and verification

Objective: fix split-store forward references in PR #75, including both endpoint
indexes and subsequent updates, deletion, and address reuse. Preserve support for
references to IDs that do not exist yet.

## Investigation checklist

- [x] Read issue #74, all comments, and every PR #75 conversation, inline comment,
  and review. There are no required sub-issues. The follow-up feedback is
  [comment 6464125763](https://github.com/linksplatform/doublets-rs/pull/75#issuecomment-6464125763).
- [x] Read contributing guidance, release workflow, and related merged PRs.
- [x] Reproduce the failure in automated tests before the original fix.
- [x] Repair index selection using existing tree operations, preserving missing
  references, updates, deletion/recreation, reuse, and callback behavior.
- [x] Add the release fragment, example, regressions, and verification script.
- [x] Compare the current C# sources and run all 12 bounded comparison cases.
- [x] Explain united/split storage and internal/external indexes consistently.
- [x] Run focused regressions and all local checks sequentially with one build job.

Commit-specific CI results, the expanded explanation, final diff review, and
review readiness are tracked on [PR #75](https://github.com/linksplatform/doublets-rs/pull/75).

Build/test logs are saved under `target/issue-74-logs/`; failed CI logs are saved
under `ci-logs/` if needed. Experiments must use finite inputs; memory-heavy
commands run sequentially with `CARGO_BUILD_JOBS=1`.

## Root cause and regression evidence

A link is a small record with an ID and two numbers called source and target.
`(2: 3 2)` means "link 2 has source 3 and target 2". It is valid even when link 3
does not exist. An index is a lookup table that helps find records by those
numbers. Reading link 2 by ID reads its record; querying `(3, 2)` uses an index.
The bug left the record intact but searched an index that lacked its entry.

The **united** store (`unit::Store` in Rust, `UnitedMemoryLinks` in C#) keeps data
and index metadata together in one memory region. Its store-wide source and
target trees index the written numbers regardless of whether those IDs exist.
Creating link 3 does not change how it finds `(2: 3 2)`.

The **split** store keeps data and index metadata in two memory regions.
Within Rust's split store, external trees index references to absent IDs, while
internal indexes belong to live referenced IDs. Before link 3 exists, link 2's
source entry is external. After creation, source searches for 3 use its internal
index. Existing entries must follow that change. These are indexes inside one
split store; the PR does not convert between united and split stores.

Creation increments the allocated ID or removes a free slot before calling the
external-reference resolver. At that point `is_virtual(free)` is false, so its
normal `each_iter` queries select the new, empty internal trees. Existing
references remain in the external trees. Queries subsequently miss them, and
updates can detach nodes from the wrong tree.

The resolver now collects references directly from each external tree before
mutating that tree, then moves each node with the existing detach/attach
operations. Both endpoint indexes are complete before the creation callback.
No data layout or reference restrictions change. Optional stderr tracing is
enabled by `DOUBLETS_TRACE_SPLIT_INDEXES=1`.

The source and target passes are intentionally separate: a link using the new
ID at both endpoints needs both entries moved. Each pass first collects link
IDs and then edits the tree, so traversal never follows a tree being changed.
The repair uses the existing resolver and tree operations without changing
stored endpoint values, IDs, API, memory layout, or reference-validity rules.

The original regression run had seven failures and a passing unit-store control.
The source reproduction returned `[]` instead of `[2]` after creating ID 3. The
target variant returned only `[3]` instead of `[2, 3]` for the target wildcard.
The multiple-usage case also exposed a concrete-pair count of `4` instead of `1`:
`count_links` selected a source-only count whenever the source was virtual, even
when the target was concrete. That branch now applies only to wildcard targets.

`doublets/tests/split_forward_reference.rs` now passes all nine tests. Coverage
includes the issue's exact fixture, target references, multiple usages on both
axes, both endpoints missing, later updates/deletion, tail deletion/recreation,
free-list reuse, creation callbacks returning `Break`, and read-handler breaks.
A bounded plain-map differential test runs four seeds of 256 operations on each
backend, with at most 24 live links and references in `1..=32`.

Run the regression suite or the original example:

```sh
cargo test -p doublets --test split_forward_reference --all-features
DOUBLETS_TRACE_SPLIT_INDEXES=1 cargo run -p doublets --example forward_references
bash experiments/issue-74/verify.sh
```

The README dependency snippet and lockfile were stale after main's 0.8.0 release;
both are synchronized to the existing manifest version. The new `bump: patch`
fragment prepares the next release without manually changing that manifest.

## Requirement inventory

| Requirement | Implementation and verification |
| --- | --- |
| Find the issue's `(2: 3 2)` after creating ID 3 | Move its source entry from the external to the internal index; `split_forward_source_reference` and `forward_references` example |
| Keep forward references supported, as on the united backend | No new rejection path; source/target/control and two-missing-endpoints tests |
| Keep indexed queries consistent with stored triples | Source/target wildcard enumeration, exact enumeration/count/search checked against a full scan and a plain-map model |
| Handle the same transition when reusing deleted IDs | Tail recreation and free-list reuse regressions, followed by update/deletion |
| Preserve existing store behavior | Creation/read `Flow::Break` tests, existing self-reference regression, and full workspace validation |
| Prepare a release and reviewable PR | Patch fragment, runnable example, repeatable checks, and PR #75 with `Fixes #74` |
| Compare behavior with C# | [Pinned source comparison and reproducible probe](csharp-comparison.md); all 12 united/split configurations pass |
| Keep code simple and concise | Existing resolver with two explicit tree passes; corrected wildcard condition for pair counts |
| Use established terms and explain before/after behavior | United/split and internal/external definitions above, C# comparison, and expanded PR description |

The issue offers rejecting unsupported references as an alternative. This change
supports them, matching the requested united-store behavior. Raw stores continue
to require unique non-null pairs; the fixtures and model respect that invariant.

## C# review

[The comparison](csharp-comparison.md) records the exact C# revision, source
links, executable probe, and results. C# selects internal versus external
indexes from a configured address range, which stays the same when an ID is
created. Rust selects them from whether the ID is live. The small reproduction
works in C#'s united and split stores, with external-reference support on or off
and with either split source-index implementation. Rust now matches those
observable results while keeping its own existing index design.

## Local validation

In the initial session, `bash experiments/issue-74/verify.sh` passed on Rust
1.99.0. The full workspace
run passed 326 tests with one existing ignored doctest; the separate default
doctest run passed four tests with the same ignored example. Formatting, strict
Clippy, release and documentation builds, package listing, the original example,
and all repository checks passed. Builds used one job and disabled debug
information; the observed container memory peak was 1,680,007,168 bytes.
The latest main commit, `02b3c9b` (0.8.0), is already an ancestor of this branch.

The 2026-10-28 follow-up repeated all of these checks successfully on Rust
1.99.0: 326 workspace tests passed, the separate default doctests passed four
tests, and one pre-existing doctest was ignored in each run. All nine focused
regressions and 12 C# comparison cases passed. Formatting, strict Clippy, release
and documentation builds, package listing, the example, and every repository
check passed. `DOUBLETS_TRACE_SPLIT_INDEXES=1` also reports the source entry
moving inside the split store while both example queries return `[2]`.

Final pushed-commit CI results and review readiness are tracked on
[PR #75](https://github.com/linksplatform/doublets-rs/pull/75).
