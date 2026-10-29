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
- [ ] Run focused regressions, all workspace and standalone package tests,
      formatting, Clippy, and repository checks with one build job.
- [ ] Review the complete PR diff, include current main, commit/push only the
      prepared branch, update PR #87, and mark it ready.
- [ ] Verify CI against the final commit SHA and timestamp. Preserve and inspect
      logs under `ci-logs/` for any failed runs and fix the actual failures.
- [ ] Verify a clean working tree and map every requirement to final evidence.

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
