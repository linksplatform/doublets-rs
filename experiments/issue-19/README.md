# Issue 19: stable Rust and external Git dependencies

## Requirements and decisions

Read issue 19, its screenshot and all comments, plus PR 28's conversation,
inline comments and reviews on November 5, 2026. There are no required
sub-issues. The latest feedback asks to check stable Rust, fix any current
failure, otherwise document the current state, update relevant code/docs,
and answer the issue.

| Requirement | Implementation | Verification |
| --- | --- | --- |
| Resolve conflicts and preserve features | Merge main, retaining its complete stable implementation | Merge commit `1e5e992` has exactly the same tree as main at `18d6a81` |
| Check the original Git consumption path on stable | Independent consumer in `verify.sh`, using a Git URL and committed revision | CRUD, default iterators, small and parallel iterators on both stores; default/full builds |
| Keep future Git consumers working | Run the consumer check in the existing CI matrix | Linux, macOS and Windows test jobs |
| Explain the historical errors and migration | README installation, stable guide, contributor guidance | Existing toolchain documentation check and review |
| Answer the issue and finish the PR | Updated PR description and issue comment, retaining `Fixes #19` | Current-commit CI, clean tree and ready status |
| Prepare the release | Patch changelog fragment | Existing changelog and version guards |

## Root cause and current state

The original June 2024 report names Git revision `5522d91c`, nightly Rust 1.81,
and a submodule copy of `platform-data 0.1.0-beta.3`. It reports E0658 for an
opaque associated iterator, E0635 for removed feature gates, and errors for
obsolete const-trait bounds. These are source/dependency compatibility errors.

PR 28's 2025 draft updated some nightly flags and submodule pointers. Main has
since migrated the maintained packages to stable in PRs 48 and 26 and removed
the submodules. Retaining the draft's nightly gates and associated types would
regress stable support; the merge keeps main's implementations and later bug
fixes. This PR adds verification and guidance; it changes no library behavior.

## Reproduction and verification

```bash
bash experiments/issue-19/verify.sh
```

This generates a consumer under ignored `target/issue-19-consumer/`, outside
the main workspace. By default it fetches the committed `HEAD` from a local
`file:///` Git URL, so CI tests its exact checked-out revision without an extra
network fetch. It does not use a Cargo path dependency and does not modify
tracked lockfiles. The consumer asserts CRUD and iterator results for three
links on both stores. The full-feature run additionally exercises the small
iterator types and constructs parallel iterators without spawning a thread
pool. All builds use stable, unset bootstrap, one Cargo job and no debug info.
Logs are in `target/issue-19-logs/`.

To check a remote revision explicitly:

```bash
DOUBLETS_GIT_URL=https://github.com/linksplatform/doublets-rs \
DOUBLETS_GIT_REV=18d6a81 \
  bash experiments/issue-19/verify.sh
```

The historical compiler diagnostics are preserved in the original issue. The
existing `experiments/issue-23/reproduce.sh` independently demonstrates stable
feature-gate failures in the old published platform dependencies; it is not
needed to test the maintained Git consumer.

## Local results

On stable Rust `1.99.0 (b940084d7 2026-09-28)`, the consumer of merge commit
`1e5e9922bb8a2fd9dd871cd13dfa3de52b2b9995` passes in default and full modes,
including all assertions on both storage layouts. This commit's source tree
matches main at `18d6a81`; no library fix is needed for the current report.
The successful check's peak child RSS was 264,324 KiB. Formatting, shell syntax,
the existing toolchain documentation checker, test-layout guard, and
`git diff --check` also pass locally. Existing CI provides the full workspace,
standalone FFI, lint, coverage, and release build checks.
