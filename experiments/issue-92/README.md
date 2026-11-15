# Issue 92: one public package with optional FFI

Plan and requirement inventory for [PR 93](https://github.com/linksplatform/doublets-rs/pull/93):

| Requirement | Implementation | Verification |
| --- | --- | --- |
| Unambiguous public name | Keep `doublets`; decision and migration in `docs/ffi.md` and README | `check-release.py` inventories publishable workspace members |
| Move FFI into primary crate | `doublets::ffi`, optional runtime dependencies, declarative export macro | Seven existing FFI regressions moved to `doublets/tests/ffi.rs`; C ABI smoke test at all four widths |
| No separate support releases | All three legacy manifests set `publish = false`; facade reexports primary FFI | Manifest policy regression; no primary path dependencies; archive FFI tests |
| OIDC trusted publishing | Both release jobs request OIDC credentials; publisher explicitly selects `doublets` | Workflow policy regression, mock publisher checks, CI; real authentication requires crates.io owner setup |
| FFI consumer instructions | `docs/ffi.md`, legacy README, stable build guide | Documented primary build and feature selection exercised in CI |
| Primary appears after workflow publication | Verify published version with `cargo search doublets --limit 1` | **Pending external release:** a PR cannot verify its post-merge publication; existing search results do not prove OIDC success |

There are no issue comments, required sub-issues, PR conversation comments,
inline review comments, or reviews at initial investigation. Main was already
an ancestor of the prepared branch. The former FFI facade remains available to
source consumers and preserves its native artifact name. No C exports were
removed; native consumers of the primary package use the new `doublets` artifact
name described in the migration guide.

## Reproduction and focused verification

Before the change, running `check-release.py` against an archive of commit
`a342098` fails because the legacy manifests do not disable publishing. The
primary crate also has no `ffi` feature and cannot provide the C API. After the
change, policy checks pass and the primary FFI regression suite passes all seven
tests (CRUD, callback flow, persistence, constants, invalid paths, logging).

```sh
python3 experiments/issue-92/check-release.py
python3 experiments/issue-92/test-publish.py
CARGO_BUILD_JOBS=1 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 \
  cargo test -p doublets --features ffi --test ffi --locked
bash experiments/issue-92/verify-package.sh
```

`test-publish.py` compiles the small std-only release script and supplies a mock
`cargo`: no registry writes or real token are needed. It checks missing-token
failure, explicit selection of `doublets`, token exclusion from arguments and
output, success output, and propagation of a failed Cargo exit status.
`verify-package.sh` runs these policy checks and then tests FFI from the extracted
crates.io archive, proving the unpublished macros and compatibility facade are
unnecessary to build the public package. CI also retains the facade tests and
checks the C ABI of both native artifact names on Linux.

The changelog fragment requests a minor bump through the existing automated
release mechanism; PR version guards prohibit a manual version bump. Actual
trusted publishing cannot be claimed until a release of this change runs on
`main` with a configured crates.io trusted publisher. The PR must remain
unmerged here; this acceptance criterion remains explicitly unfinished.
