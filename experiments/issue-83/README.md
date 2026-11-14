# Issue 83: consolidate decorators in the main library

## Requirements and plan

Issue 83 and all three PR 84 feedback endpoints were read on October 29, 2026.
There are no comments, reviews, or required sub-issues. The requirements are:

1. Verify that the main library contains every feature of the redundant package.
2. Remove that package without losing decorator behavior or test coverage.
3. Update the whole codebase and documentation to use the main library.

The implementation plan is to audit every export and package consumer, add
direct-constructor parity tests before removal, delete the redundant package,
update CI and developer guidance, and run the repository's local and CI checks.
Builds use one Cargo job and reduced debug information within the container's
2.92 GiB memory limit. Build logs go to `target/issue-83-logs/`.

## Feature audit and migration

At baseline commit `bb30603`, `doublets-decorators/src/lib.rs` contains only six
`pub use` exports from `doublets::decorators`. There are no implementation modules,
additional public items, package features, build scripts, examples, or other
tests. Its only dependency is this checkout's `doublets` with `full` enabled.
Those feature flags enable platform re-exports and optional search/iteration
optimizations; the decorator module itself is unconditional in `doublets/src/lib.rs`.
Consumers that relied on the transitive feature selection can enable `full` on
their main-library dependency, as documented in the README migration guidance.

Remove the old dependency and import the following types from
`doublets::decorators`. The former names were aliases for these exact types:

| Former export | Main-library export | Verification in `doublets/tests/decorator_consolidation.rs` |
| --- | --- | --- |
| `UniqueResolver` | `UniquenessResolver` | Duplicate creation returns the surviving address; unwrap preserves the store |
| `UniqueValidator` | `UniquenessValidator` | Duplicate creation returns `AlreadyExists` and preserves the original point |
| `UsagesValidator` | `UsagesValidator` | Referenced links reject deletion and updates |
| `CascadeUsagesResolver` | `CascadeUsagesResolver` | Nested usages are deleted, including composition with the reset layer |
| `NonNullDeletionResolver` | `NonNullContentsLinkDeletionResolver` | Callback records show reset before deletion |
| `CascadeUniqueResolver` | `CascadeUniquenessAndUsagesResolver` | Duplicate creation reuses links; merging redirects usages to the survivor |

Every type retains `new(store)`, `inner()`, `inner_mut()`, `into_inner()`, the
same traits, and the stable `Flow` callback API because these are the original
implementations. The six scenarios run against unit and split stores: 12 tests.
The previous package's complete compatibility scenario is preserved and extended
here. Existing decorator, transaction, cascade, and fusion tests remain in place.
No decorator implementation or main-library public API needs changing.

The expanded split-store coverage exposed an existing storage bug with null
targets. `is_unused` inspected only the target-tree size, which is zero for a
live `(source, null)` link as well as a free-list slot. It now requires both
endpoint-tree sizes to be zero before treating the data as free-list pointers.
This fixes the original compatibility scenario on split stores too.
`parity-before-fix.log` records two failing split scenarios; the independent
two-test `split_null_endpoint.rs` regression also failed before the change
(`null-endpoint-before-fix.log`). It checks the minimal case and free-slot
reuse/tail deletion with null endpoints. All 14 tests pass after the change,
before removing the compatibility package.

## Requirement evidence

| Requirement | Implementation | Verification |
| --- | --- | --- |
| Full feature parity | Export-by-export audit above; main-library constructor tests | Run parity tests before removal and afterward without default features |
| Preserve behavior on both backends | Correct split-store liveness for null targets | Two bare-store regressions plus split decorator validation and cascade tests |
| Remove the redundant package | Delete its manifest, lockfile, library and test; remove workspace exclusion | Discover the five remaining package manifests; compile all with `--locked` |
| Update codebase | CI uses workspace tests for decorators; standalone verification uses remaining FFI packages | Three-OS CI includes the parity suite with all features and without default features |
| Update documentation | README import/migration guidance, contribution layout/checks, stable Rust package inventory, current experiment instructions | Repository-wide reference search; retain only migration guidance and clearly historical evidence |
| Prepare release | Changelog fragment drives automatic versioning | Changelog and version-modification guards |

## Verification

```sh
# Parity suite also runs before deleting the redundant package.
CARGO_BUILD_JOBS=1 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
  cargo test -p doublets --test decorator_consolidation --no-default-features --locked -- --test-threads=1
bash experiments/issue-83/verify.sh
```

`verify.sh` runs formatting, strict Clippy, workspace tests, default doctests,
parity without default features, standalone FFI and macro tests, all maintained
package checks, release/documentation builds, package listing, dependency audit,
and documentation checks. The existing issue 22 verification remains available
for the wider C ABI and embedded release-script dependency checks.

Historical issue 22 compiler logs and the issue 68 dependency snapshot describe
the repository before consolidation. They are retained as evidence, not as a
list of maintained packages. Current commands discover or use only existing
manifests.

## Local results

The full verification completed on Linux with stable Rust 1.99.0:

- 399 workspace tests passed, zero failures, one existing ignored doctest;
  default-feature doctests also passed (four tests and the same ignored example).
- All 14 new tests passed with default features and without default features.
- Strict Clippy, formatting, release and documentation builds, and package listing passed.
- Seven FFI regressions, both procedural macro suites, and the C ABI smoke tests
  for u8/u16/u32/u64 passed.
- All five maintained package manifests and all 13 embedded script manifests passed.
- Dependency inventory, file-size, badge, toolchain, changelog, version, and diff
  checks passed. There were no compiler OOM kills.

The next release is prepared through the patch changelog fragment; the package
version remains under automatic release management.
