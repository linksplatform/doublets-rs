# Issue #74 investigation and verification

Objective: fix split-store forward references in PR #75, including both endpoint
indexes and subsequent updates, deletion, and address reuse. Preserve support for
references to IDs that do not exist yet.

## Work plan

- [x] Read issue #74, all comments/sub-issues, and every PR #75 comment/review.
  There are no comments, reviews, or required sub-issues at the start of work.
- [x] Read contributing guidance, release workflow, and recent related PRs.
- [x] Trace index migration and reproduce the failure with automated tests before
  changing production code.
- [x] Fix the root cause using the existing internal/external tree operations.
- [x] Verify source and target references, multiple usages, callback behavior,
  updates, deletion/recreation, and free-list reuse.
- [x] Add the patch-release changelog fragment (the automatic release workflow
  manages manifest versions).
- [x] Run focused tests, full workspace/all-feature tests, doctests, formatting,
  strict Clippy, release build/package checks, and repository checks. Use one
  Cargo build job and reduced debug information within the 2.92 GiB limit.
- [ ] Review the diff, merge current main if needed, commit each useful atomic
  step, and push only `issue-74-b2cc25ca247c`.
- [ ] Update PR #75 with reproduction, implementation, evidence, and `Fixes #74`.
- [ ] Inspect CI runs by timestamp and SHA, preserve/analyze any failing logs,
  verify passing checks on the final commit, and mark PR #75 ready.
- [ ] Verify the working tree is clean and every requirement has evidence.

Build/test logs are saved under `target/issue-74-logs/`; failed CI logs are saved
under `ci-logs/` if needed. Experiments must use finite inputs; memory-heavy
commands run sequentially with `CARGO_BUILD_JOBS=1`.

## Root cause and regression evidence

Creation increments the allocated ID or removes a free slot before calling the
external-reference resolver. At that point `is_virtual(free)` is false, so its
normal `each_iter` queries select the new, empty internal trees. Existing
references remain in the external trees. Queries subsequently miss them, and
updates can detach nodes from the wrong tree.

The resolver now collects references directly from each external tree before
mutating that tree, then moves each node with the existing detach/attach
operations. Both endpoint indexes are complete before the creation callback.
No data layout or reference restrictions change. Optional stderr tracing is
enabled by `DOUBLETS_TRACE_SPLIT_INDEX_MIGRATION=1`.

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
DOUBLETS_TRACE_SPLIT_INDEX_MIGRATION=1 cargo run -p doublets --example forward_references
bash experiments/issue-74/verify.sh
```

The README dependency snippet and lockfile were stale after main's 0.8.0 release;
both are synchronized to the existing manifest version. The new `bump: patch`
fragment prepares the next release without manually changing that manifest.

## Requirement inventory

| Requirement | Implementation and verification |
| --- | --- |
| Find the issue's `(2: 3 2)` after creating ID 3 | Direct external-to-internal source migration; `split_forward_source_reference` and `forward_references` example |
| Keep forward references supported, as on the unit backend | No new rejection path; source/target/control and two-missing-endpoints tests |
| Keep indexed queries consistent with stored triples | Source/target wildcard enumeration, exact enumeration/count/search checked against a full scan and a plain-map model |
| Handle the same transition when reusing deleted IDs | Tail recreation and free-list reuse regressions, followed by update/deletion |
| Preserve existing store behavior | Creation/read `Flow::Break` tests, existing self-reference regression, and full workspace validation |
| Prepare a release and reviewable PR | Patch fragment, runnable example, repeatable checks, and PR #75 with `Fixes #74` |

The issue offers rejecting unsupported references as an alternative. This change
supports them, matching the requested unit-store behavior. Raw stores continue
to require unique non-null pairs; the fixtures and model respect that invariant.

## Local validation

`bash experiments/issue-74/verify.sh` passed on Rust 1.99.0. The full workspace
run passed 326 tests with one existing ignored doctest; the separate default
doctest run passed four tests with the same ignored example. Formatting, strict
Clippy, release and documentation builds, package listing, the original example,
and all repository checks passed. Builds used one job and disabled debug
information; the observed container memory peak was 1,680,007,168 bytes.
The latest main commit, `02b3c9b` (0.8.0), is already an ancestor of this branch.

Final pushed-commit CI results and review readiness are tracked on
[PR #75](https://github.com/linksplatform/doublets-rs/pull/75).
