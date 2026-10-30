# Issue #86: callback control through logging

Objective: implement and verify every requirement of issue #86 in PR #87.
The issue has no comments or required sub-issues; PR conversation, inline comments,
and reviews were empty at the initial audit. Data.Doublets #519 / PR #544 provide
the original parity audit, rather than additional Rust implementation requirements.

## Plan and requirement inventory

- [x] Read the complete issue, related audit, repository guidance, and PR feedback.
- [x] Trace `create_link_with`, `Fuse`, logging, and uniqueness cleanup.
- [x] Reproduce the reported update-callback break before changing library code.
- [x] Return recorded callback control independently of primitive return values.
- [x] Preserve initialized live links, callback suppression after break, and all
      completed mutation log entries, including uniqueness/cascade cleanup.
- [x] Cover create/update breaks and continuing handlers on unit/split stores,
      plain/automatic stacks, logging composition, primitive breaks, and errors.
- [x] Add a patch changelog fragment for the automatic release workflow.
- [x] Run focused regressions, all workspace and standalone package tests,
      formatting, Clippy, and repository checks with one build job.
- [x] Review the complete PR diff, include current main, commit/push only the
      prepared branch, update PR #87, and mark it ready.
- [x] Verify CI against the implementation commit SHA and timestamp. Preserve and inspect
      logs under `ci-logs/` for any failed runs and fix the actual failures.
- [x] Verify a clean working tree and map every requirement to final evidence.

## Reproduction

```sh
CARGO_BUILD_JOBS=1 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
  cargo test -p doublets --test creation_flow --all-features
```

Build/test logs are retained locally under `target/issue-86-logs/`. The tests use
small finite stores and do not stress memory or stack limits.

## Root cause and evidence

`Fuse` already remembers whether a callback returned `Break`, but the compound
helper returned only `update_links`' result. Logging deliberately returns
`Continue` to the inner store so it can log every completed mutation, and then
also returned that inner result to the caller. Both boundaries now consult the
recorded flow. Inner breaks and errors still propagate; a logging I/O error still
wins over callback control. Logging continues to pass `Continue` inward.

Before library changes, `before.log` records **18 failures and 7 passes** across
25 focused regressions. `logged_creation_flow` reproduces the exact null-endpoint
example for both backends. `creation_preserves_discarded_break` uses a small
forwarding store whose update intentionally discards callback control, independently
testing the helper. `logged_update_preserves_discarded_break` checks the logging
boundary independently of the helper. The remaining cases cover creation/update
breaks, continuing handlers, both logging/automatic stack orders, duplicate-pair
survivors, complete cascade logs after callback suppression, primitive breaks,
and store/writer errors. All **25 pass** after the fix (`after.log`).

Run the complete local checks with `bash experiments/issue-86/verify.sh` (requires
`rust-script`, Clippy, rustfmt, and Python 3). The patch fragment requests the next
release through the existing automation; manual version edits are prohibited by
the repository's version check.

## Final verification

All checks in `verify.sh` passed locally on stable Rust 1.99.0, using one build job
and reduced debug information:

- 424 workspace tests passed, including the 25 new cases; one existing doctest is
  ignored. Separate default-feature doctests: four passed, one existing ignored.
- All five maintained manifests compiled with every target and feature. The seven
  FFI tests and both procedural macro suites passed.
- Core compilation, the 12 direct-constructor decorator tests, and all 25 new
  regressions passed without default features.
- Strict workspace Clippy, formatting, release and documentation builds, package
  listing, and C ABI smoke tests for u8/u16/u32/u64 passed.
- File-size, README badge, toolchain documentation, changelog, and version checks
  passed. There were no memory-limit or OOM events during validation.

The [full CI pipeline](https://github.com/linksplatform/doublets-rs/actions/runs/40974840171)
passed for implementation commit `1c4eff70a8a817f8b6c13bc51d06fe2aec53640e`
(run created 2026-10-29 23:41:28 UTC, after the commit). This includes Linux,
macOS, Windows, coverage, lint, and changelog/version checks. CodeFactor passed.
No failed CI runs required log investigation. The latest commit's checks remain
visible on [PR #87](https://github.com/linksplatform/doublets-rs/pull/87).

Current main (`645dd2b8e5c9bff1e38335da42e3c008b5d2769e`) is already an ancestor
of the prepared branch. The complete PR diff was reviewed; no existing feature
was removed. A final feedback refresh found no issue comments, required
sub-issues, PR conversation/inline comments, or reviews. Every issue requirement
is implemented and verified, with no deferred work.
