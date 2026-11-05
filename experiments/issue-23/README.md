# Issue 23: historical build failure and current guidance

## Requirements and plan

Read issue 23 and its comment, every PR 25 conversation comment, inline comment
and review, and the sub-issue endpoint on November 5, 2026. There are no required
sub-issues. The latest feedback asks whether the report remains valid and asks
for useful build documentation and a comment on the issue if it is resolved.

| Requirement | Implementation | Verification |
| --- | --- | --- |
| Determine whether the report remains valid | Reproduce the exact old platform dependencies; compare with maintained main | `bash experiments/issue-23/reproduce.sh`; stable workspace build |
| Resolve conflicts without losing features | Merge main, retaining its source, dependencies, lockfiles and complete features | Main ancestry, PR diff and existing cross-platform CI |
| Preserve useful build guidance | Replace the inaccurate original report; extend the stable guide and link it from setup instructions | Existing toolchain documentation checker and review of commands |
| Explain current versus historical behavior | Record old versions, current minimum Rust, and migration steps | Issue comment and updated PR description |
| Prepare a release | Patch changelog fragment, leaving versioning to the release pipeline | Changelog and version guards |
| Finish the PR | Commit, push, inspect current-SHA CI and mark ready | Clean tree, PR status and checks |

## Root cause and decision

The report is valid for the old dependencies: `platform-num 0.1.0-aplha.1` and
`platform-data 0.1.0-beta.3` explicitly enabled unstable language/library features.
The script above generates two isolated Cargo projects under ignored `target/`
and checks each on stable with bootstrap unset. It requires E0554, so network or
dependency-resolution failures cannot be mistaken for a reproduction. It uses
one build job and does not modify repository lockfiles.

Main already resolved this through PRs 48 and 26, with documentation follow-up
in PR 56. Its platform dependencies are stable releases from crates.io. The
original PR 25 draft instead disabled required dependencies, left unresolved
unstable constructs, and described uncommitted submodule edits as a solution.
Those changes cannot provide a working library. The merge keeps main's complete
implementation and removes the draft's regressions. The resulting source and
manifests match main; this PR's remaining improvements are documentation,
reproduction evidence, and a release fragment.

Rust 1.81 is below the current library's declared minimum of 1.85. Repository
development and FFI use latest stable, as their development dependencies can
have higher requirements. The 2022 nightly mentioned in the issue is historical
advice for historical code, not the current build procedure.

## Verification

Logs are written to `target/issue-23-logs/`:

```bash
bash experiments/issue-23/reproduce.sh
env -u RUSTC_BOOTSTRAP CARGO_BUILD_JOBS=1 CARGO_PROFILE_DEV_DEBUG=0 \
  cargo +stable build --workspace --locked > target/issue-23-logs/build.log 2>&1
rustc +stable scripts/check-toolchain-docs.rs -o target/issue-23-docs-check
target/issue-23-docs-check
```

The checker is a dependency-free Rust script, so compiling it directly avoids
installing `rust-script` locally. CI runs the same checker with `rust-script`.
Existing CI covers all-feature and default-feature tests, all five maintained
package manifests on stable, standalone FFI tests, and Linux/macOS/Windows.
Full suites are left to that CI to avoid duplicating resource-intensive work.

Local results on stable Rust 1.99.0:

- Both historical probes fail with E0554 as expected. The `num` log points to
  `#![feature(step_trait)]`; the `data` log also reports the unresolved
  `std::default::default` import and unknown feature names from the issue.
- The current workspace builds successfully with bootstrap unset and the
  tracked lockfile, using one Cargo job and no development debug information.
- The existing toolchain documentation checker passes, as do `bash -n` on
  the reproduction script and `git diff --check`.
- Main is an ancestor of the branch, and the source trees, Cargo manifests,
  and lockfiles are identical to main. No runtime behavior is changed.
