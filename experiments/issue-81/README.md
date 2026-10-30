# Issue 81: explicit endpoint reset and minimal release

## Research and checklist

Read issue 81, its complete comments and sub-issue inventory (both empty), all
three PR 85 feedback endpoints (empty), the linked Data.Doublets PR 527 review,
and the related issue 58/67 and PR 77/84 work on October 29, 2026. All feedback
endpoints were read again on October 30, including the request to move Rust
tests out of source directories; there are still no required sub-issues.

- [x] Identify the unconditional backend update and reproduce redundant reset.
- [x] Extract and document a minimal unchecked release on both layouts.
- [x] Offer checked release without automatic endpoint transformation.
- [x] Use the existing reset resolver as the optional transformation layer.
- [x] Preserve existing safe deletion behavior through a convenience adapter.
- [x] Preserve structural indexes, incoming usages, reuse and callbacks.
- [x] Test invalid inputs only through safe APIs; test unchecked release only
      with a live address whose endpoints have already been reset.
- [x] Verify self references, incoming source/target/overlapping usages, tail
      release, holes, reuse, callback states/results and fused Break behavior.
- [x] Run focused tests and all maintained local CI checks.

Fresh PR CI results and final readiness are recorded on
[PR 85](https://github.com/linksplatform/doublets-rs/pull/85).

The baseline unit and split `delete_links` both call `self.update(index, 0, 0)`
even after the caller has already performed that update. Test-only counters in
the actual stores measure update invocations; they add no production fields or
runtime work. The reproducer is
`deletion_of_reset_link_does_not_update_again` in
[`tests/internal/unit.rs`](../../doublets/tests/internal/unit.rs) and
[`tests/internal/split.rs`](../../doublets/tests/internal/split.rs).
Logs are kept in `target/issue-81-logs/`.

## Architecture decisions

The minimal operation releases a live slot with zero endpoints and no endpoint
index memberships. It performs no contract checks or endpoint updates. Incoming
usages are permitted and preserved: the unit layout has global endpoint trees;
the split layout must reclassify entries to the external trees. This structural
work cannot be switched off. Incoming references become internal again on reuse.
Rejecting or cascading incoming usages remains a separate policy.

Safe callers get a checked release method and a `CheckedRelease` adapter, which
reject nonzero contents instead of resetting. The existing `delete*` methods
retain their behavior through a convenience composition of the reset resolver
and checked release. This avoids changing existing deletion helpers, callback
contents, FFI behavior, or cascade behavior. The explicit resolver over checked
release reports reset and deletion events, while the compatibility convenience
adapter retains the original single deletion callback.

Release preparation uses a changelog fragment, as required by the automatic
release workflow; the version guard forbids manual manifest version bumps.

## Requirement-to-evidence map

| Requirement | Implementation | Automated verification |
| --- | --- | --- |
| Minimal release, no reset or contract validation | `Release::release_unchecked` implementations in both stores | `tests/internal/{unit,split}.rs::minimal_release_and_resolver_do_only_required_updates`; valid unchecked calls in `tests/release.rs` |
| Explicit live/detached/incoming contract | `mem::Release` safety documentation; `docs/deletion.md` | Safe rejection of null/service/out-of-range/freed addresses and nonzero endpoints; no invalid unchecked tests |
| Optional resetting layer | Existing resolver composed with `CheckedRelease` | `reset_is_an_explicit_layer`, update counts (zero direct, exactly one resolver reset), already-reset skip |
| Preserve safe convenience deletion | `decorators::delete_with_reset` compatibility composition | Existing deletion return, handler, query, delete-all, cascade, decorator, transaction and FFI suites |
| Preserve required structural bookkeeping | Split incoming-index reclassification remains in the primitive | Source/target/both-endpoint search and counts before/after release and reuse on both layouts |
| Incoming usage policies remain configurable | Validation/cascade decorators over checked release | `usage_policy_composes_with_release`; existing shared/cyclic cascade regressions |
| Correct callbacks, flow and results | Reset-state primitive event; original-state compatibility event | `release_callbacks_and_reuse`, `resetting_layer_fuses_callbacks`, existing callback/return regressions |
| Both layouts, free lists, tail collapse, reuse | Both primitive implementations | All new scenarios run on unit and split, including holes and allocated-tail release |
| Document supported deletion routes | API docs, deletion guide, README, `examples/release.rs` | Compiled doctests and executable example; no-default release tests on all three CI platforms |
| Prepare next automated release | Minor changelog fragment | Existing changelog/version guards; no manual version bump |
| Rust tests live under `tests/` | Private-state suites in `doublets/tests/internal/{mem,unit,split}.rs`; FFI integration suite in `doublets-ffi/tests/ffi.rs` | `check-test-layout.sh` runs locally and in CI; before/after inventories preserve all 12 library tests and seven FFI tests |

## Reproduction and verification

The baseline regression fails on both layouts with **3 updates observed versus
2 expected**; see [`before-fix.log`](before-fix.log). The two extra update-count
tests also verify checked and unchecked direct release and resetting-layer
behavior against the actual stores, without adding production instrumentation.

```sh
CARGO_BUILD_JOBS=1 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
  cargo test -p doublets --lib deletion_of_reset_link_does_not_update_again --locked -- --test-threads=1
bash experiments/issue-81/verify.sh
```

The verification script saves logs in `target/issue-81-logs/`. It runs the new
suite with default features and without defaults, the example, and all maintained
repository checks: strict Clippy, workspace tests/doctests, release and docs,
standalone FFI/macro tests, C ABI smoke tests, package inventory, embedded script
builds and documentation/release guards, plus the source test layout check.
Builds use one Cargo job and reduced
debug information. Cases are finite; no invalid unchecked memory is exercised.

`DOUBLETS_TRACE_SPLIT_INDEXES=1 cargo run -p doublets --example release` logs
incoming-entry moves in both directions; the existing tracing option remains
disabled by default.

Local verification on October 29, 2026 passed: 421 workspace tests, zero failures
and one existing ignored doctest; six default-feature doctests; all 16 release
integration tests with defaults and without defaults; the executable example;
strict formatting/Clippy, release/docs/package checks; standalone FFI and macro
checks; C ABI callbacks, disposal and reopening for all four address widths;
all five package checks; all 13 embedded release-script builds; and repository
documentation/release guards. The trace example confirmed incoming-index moves
on both release and reuse. Builds stayed within the container's memory limit.

## October 30 review: test organization

[PR feedback](https://github.com/linksplatform/doublets-rs/pull/85#issuecomment-6523873049)
requires Rust test implementations to live under `tests/`. Before relocation,
`bash experiments/issue-81/check-test-layout.sh` failed with four offending files;
[`test-layout-before.log`](test-layout-before.log) preserves the evidence.

The three private-state suites moved unchanged to `doublets/tests/internal/`.
Test-only path declarations retain their existing module scope, private field
access, test names and update counters without exposing production internals.
The FFI tests now import the public bindings from `doublets-ffi/tests/ffi.rs`;
an additional `rlib` output supports Rust integration tests alongside the existing
C library outputs. No test implementation remains under `src/`, and the flat
test files avoid creating single-file module directories. Documentation examples
remain with their APIs and are still checked as doctests.

For PR CI, list the most recent runs with timestamps and SHAs, confirm each run
matches the latest push, download any failing run into `ci-logs/`, inspect actual
errors and line numbers, fix them, and verify a fresh run. Before marking ready,
review the GitHub PR diff and all feedback endpoints, confirm latest main is an
ancestor and the working tree is clean, and preserve `Closes #81` in the PR body.
