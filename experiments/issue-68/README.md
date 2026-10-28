# Issue 68: allocation slices and dependency updates

## Root cause and regression

`RawMem::grow_filled` returns the newly added region. `resize_mem` passed that
slice to both stores as their entire allocation. Growing from 8,192 to
1,048,576 records therefore exposed a slice of 1,040,384 records. Its pointer
also started after the allocation's header. A later growth similarly discarded
the earlier region from the store's view.

Before the fix, all seven new memory regressions failed. Both stores and both
backends reported an exposed capacity of `1040384` instead of `1048576`; the
small resize tests returned `[0, 0, 0]` instead of
`[11, 22, 33, 44, 0, 0, 0]`. The original failure log is preserved locally in
`target/issue-68-logs/regression-before.log`.

The helper now performs growth and returns `allocated_mut()`, just as for
shrinking or retaining capacity. A search of all Rust sources found no other
production calls to `grow` or `grow_filled`; every store growth uses this helper.

The seven regressions cover:

- Initial capacity and reserved-header consistency for unit/split stores using
  `Global` and `FileMapped`.
- Three further growth events per store, with every earlier link checked by ID
  and by indexed search after each creation.
- Reopening file-backed unit/split stores and verifying all 50 earlier links.
- Empty, growing, unchanged, shrinking, and regrowing allocations, with contents
  and complete-allocation pointers checked.
- A test backend that deliberately returns only the added region, independently
  of future changes to the real backend's return contract.

Store tests check the real initial allocation, then shrink the empty store to
16 records and grow in steps of 16. They create only 50 links rather than
millions. Production allocation sizes remain unchanged.

Updating rand changed the existing seeded model test's operation sequence and
exposed a second bug: an empty split source index used `get_size(0)`, reading
the free-list head from the shared header. The minimal regression creates four
empty links and deletes IDs 1 and 2; a query for absent source 99 incorrectly
reports two links. `empty_indexes_ignore_free_list_header` failed before the
fix (`target/issue-68-logs/empty-index-before.log`). All four endpoint tree
implementations now use the existing null-aware `get_size_or_zero` helper.
The minimal regression and all ten forward-reference/model tests pass. Optional
`DOUBLETS_TRACE_RANDOM_CHANGES=1` tracing preserves the operation sequence for
future investigations without adding default output.

## Dependency inventory

The final issue comment requires **all dependencies**, not only platform-mem,
to use their latest stable releases. `audit_dependencies.py` queries crates.io,
filters out prereleases and yanked releases, and records the highest stable
version. `dependencies.json` preserves the 2026-10-28 audit, including source
URLs, declared Rust requirements, and the original dependency declarations.
All 60 versioned declarations across all six Cargo manifests and the embedded
rust-script manifests now match that snapshot. The workspace lockfile and
standalone legacy crate lockfiles resolve the updated requirements.

Notable upgrades include platform-mem 0.4.0, rand 0.10.3, criterion 0.8.2,
syn 3.0.9, darling 0.24.1, and ureq 3.5.0. Code using their changed APIs has
been adapted. Procedural macro integration tests cover the generated exports
and default environment configuration. CI compiles the embedded release
scripts as test executables, avoiding release/publish side effects.

The excluded `doublets-decorators` and `doublets-ffi` crates remain excluded as
in the original workspace; their dependency declarations and resolutions are
updated as well. This issue does not restore the legacy APIs currently under
rework. Their two standalone procedural macro packages are tested directly.
The main library retains its Rust 1.85 requirement; current development tools
use stable Rust (criterion requires 1.86; darling requires 1.88).

Development automation is included: checkout v7, cache v6, Codecov v7, and
pre-commit-hooks v6. The other action pins already track their latest stable
release series or the upstream stable tool installer. Historical case-study
snapshots and pinned C# comparison fixtures are retained as historical evidence.
`automation.json` records the upstream stable release tags and URLs checked.

The changelog fragment triggers the repository's automatic patch release.
Its CI prohibits manually changing the package version in a normal PR.

## Verify

```sh
python3 experiments/issue-68/audit_dependencies.py --check
# Optional fresh audit without modifying the saved snapshot:
python3 experiments/issue-68/audit_dependencies.py --check-live
bash experiments/issue-68/verify.sh
# Linux: four real-size probes, each limited to 512 MiB virtual memory:
bash experiments/issue-68/run-growth.sh
```

`verify.sh` runs strict Clippy, all workspace tests/features, default doctests,
release/documentation builds, package listing, standalone macro tests,
repository checks, and compilation of all embedded dependency manifests.
It uses one Cargo job and reduced debug information. Logs are saved under
`target/issue-68-logs/`.

`memory_growth` creates exactly 2,097,153 self-referencing links, crosses the
original failing position and two growth boundaries, and checks earlier
links and searches. The four modes use plain Global/FileMapped backends for
both stores, without a workaround wrapper. Failures propagate as a nonzero
exit status. Build and runtime limits are separated so the compiler is not
subject to the probe's virtual-memory limit.

## Validation results (Linux, 2026-10-28)

Both verification scripts completed successfully:

- 330 workspace unit/integration tests and four all-feature doctests passed;
  one existing doctest remains ignored. Four default-feature doctests passed.
- Strict Clippy for every workspace target/feature, formatting, release build,
  documentation build, and package listing passed.
- Both standalone procedural macro integration tests passed, as did compilation
  of all 13 embedded dependency manifests and the file-size, badge, and toolchain
  documentation checks.
- Every versioned Rust dependency declaration matches the stable release audit.
- Each of the four bounded production-size probes created 2,097,153 links and
  preserved earlier links and indexed searches.

The build/test commands ran sequentially with one Cargo job. The container's
observed total memory peak was 2.22 GiB, below its 2.92 GiB limit. The real-size
runtime probes had their own stricter 512 MiB virtual-memory limit.

The first CI analysis reported CodeFactor B310 at `audit_dependencies.py:45`
on commit `e4bb85d`; its annotation is saved in
`ci-logs/codefactor-e4bb85d.json`. The audit now uses an HTTPS-only connection
to crates.io. A fresh `--check-live` audit exercised that client for every
declared crate and confirmed that all 60 declarations remain current.
