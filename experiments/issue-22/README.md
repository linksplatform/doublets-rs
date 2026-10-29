# Issue 22: complete stable Rust migration

Requirements were read from issue 22 (empty description and one historical
comment) and all PR 26 conversation comments, review comments, and reviews on
October 29, 2026. There are no required sub-issues. The latest maintainer comment
requires conflict resolution and a stable Rust audit across all dependencies.

The findings and counts below describe the completed migration at that time.
Issue 83 subsequently consolidated all decorators in `doublets::decorators` and
moved their compatibility scenarios to `doublets/tests/decorator_consolidation.rs`.
The verification scripts now use the five remaining package manifests and run
decorator tests as part of the workspace suite. The saved `before/` logs remain
historical evidence.

## Reproduction and root causes

Before implementation, run `bash experiments/issue-22/check-packages.sh` on the
pre-migration baseline. The workspace and procedural macros built, but both excluded
packages failed. Saved compiler output is in `before/`:

- `decorators.log`: E0554 (nightly feature gates), E0635 (`default_free_fn`),
  unstable `Try` use, and outdated `Doublets`/`LinkType` APIs. The package also
  resolved crates.io doublets 0.9.0 instead of this checkout.
- `ffi.log`: removed `box_syntax`, E0554, and the removed `LinkType` API.
  The FFI lockfile initially also failed `--locked` because its local doublets
  version was stale. Removing only the gates would leave its erased trait-object
  pointer incompatible with the current API and its creation callback inverted.
- `byte-store.log`: the minimal Rust regression
  `cargo test -p doublets --lib byte_store_can_allocate_its_full_address_range`
  fails at the capacity conversion with `TryFromIntError(PosOverflow)`. The
  million-record default capacity cannot be represented in u8/u16. Once FFI
  compilation was restored, this panic aborted `ByteLinks_New` as well.

The initial allocation now fits the index type; the regression fills the complete
u8 address range and checks `LimitReached`. FFI handles use matching
`Box::into_raw`/`Box::from_raw` operations on a concrete store. FFI regressions
exercise both constructors, CRUD, callback continuation/breaks, disposal, and file reopening for
u8/u16/u32/u64, plus constants conversion, invalid paths, and the existing logger.
The compatibility package re-exports all six original names from the tested
current decorator implementations, removing the obsolete duplicate modules.

## Requirement evidence

| Requirement | Implementation | Verification |
| --- | --- | --- |
| Resolve conflicts and preserve current features | Merge main without rewriting branch history; keep its current docs/version and all storage/decorator/sequence/transaction code | Main is an ancestor; review PR diff; full workspace tests |
| Build on stable across dependencies | Remove the remaining nightly features; use current local decorators and FFI APIs | Discover and compile all six package manifests, all targets and features, explicitly on stable without bootstrap |
| Preserve useful original PR changes | Keep stable toolchain documentation and crates.io dependency guidance; remove stale submodule instructions | Toolchain documentation checker and README badge checker |
| Validate standalone compatibility behavior | Six original decorator names and every integer C export remain available | Decorator integration test, seven FFI tests, and Python C ABI smoke test |
| Keep release checks reproducible | Synchronize installation snippets and local package versions in tracked lockfiles when releasing | Release script unit tests preserve other snippets, registry versions, and lockfile format |
| Keep CI/CD passing | Add package checks and compatibility tests to the existing three-OS matrix; retain embedded manifest checks | Fresh PR checks on final head SHA |
| Prepare the next release | User-facing changelog fragment; automatic versioning remains in control | Changelog and version-modification checks |

## Verification commands and logs

Run `bash experiments/issue-22/verify.sh` for the complete local checks, using
one Cargo job, reduced debug information, serial tests, and `-Dwarnings`.
Detailed logs are saved in `target/issue-22-logs/`. All builds and runtime probes
have finite inputs; no stress test is needed for this migration.

Local validation on stable Rust passed: 385 workspace tests, default doctests
(four passing and one existing ignored example), seven FFI regressions, the
decorator compatibility test, both procedural macro integration suites, all
six package graphs, and all 13 embedded script manifests. The two new release
bookkeeping tests also passed, as did strict Clippy, formatting, release and
documentation builds, package listing, and repository documentation/release checks.

After building the FFI release library, run:

```sh
python3 experiments/issue-22/ffi-smoke.py target/release/libdoublets_ffi.so
```

The script also supports `.dylib` and `.dll` paths. CI builds and runs the Rust
compatibility tests on Linux, macOS, and Windows, and runs the Python C ABI
smoke test against the release library on Linux. Release script manifests are
compiled with `rust-script --test`, avoiding release and publish side effects.

Nightly remains only in optional Miri diagnostics and historical documentation.
Neither the package build graph nor normal tests/releases select nightly. Miri
is a separate tool and is not required for any supported package to build.

The last pre-session PR run passed on bdf6b3d in April 2026, before current main.
The three older failed runs listed by GitHub are from September 2025; attempts
to download them returned HTTP 410 (logs expired). Fresh CI provides current
verification rather than relying on those stale runs.
