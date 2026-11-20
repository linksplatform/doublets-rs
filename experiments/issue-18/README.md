# Issue 18: `unresolved import thiserror` after `cargo add doublets`

## Requirements and plan

Read issue 18 (no comments, no sub-issues), every PR 29 conversation comment,
inline comment and review on November 14, 2026. The latest PR feedback asks to
resolve conflicts, confirm that the issue is fully eliminated on latest stable
Rust, comment on the issue or update docs as needed, and keep every change
validated, tested, documented and passing CI.

| Requirement | Implementation | Verification |
| --- | --- | --- |
| Resolve conflicts without losing features | Merge main; every conflict takes main's complete stable implementation | Main is an ancestor; source, manifests and lockfiles match main |
| Find the root cause | Reproduce the issue's exact errors with the old release and its 2022 nightly; isolate the dependency drift | `bash experiments/issue-18/reproduce.sh`; [historical-errors.log](historical-errors.log) |
| Eliminate the issue on latest stable | Stable releases (0.2.0 and newer) replace the nightly-only crates; `cargo add doublets` selects them | `bash experiments/issue-18/verify.sh crates-io` |
| Keep it eliminated | CI consumes the packaged crate through `cargo add` with freshly resolved dependencies | `bash experiments/issue-18/verify.sh package` on Linux/macOS/Windows |
| Document the fix and migration | Troubleshooting section, README/build-status/contributor links | [STABLE_RUST.md](../../STABLE_RUST.md#cratesio-releases-and-issue-18); documentation checker |
| Comment on the issue and prepare a release | Issue comment; patch changelog fragment | Issue thread; changelog and version guards |

## Root cause

When the issue was reported, crates.io had no stable `doublets` release, so
`cargo add doublets` selected `0.1.0-pre+beta.15` (shown as `0.1.0-pre`; its
feature list matches the issue output). That release and its
`platform-data 0.1.0-beta.3` dependency are nightly-only and:

1. derive `thiserror::Error` with a `#[from] #[backtrace] io::Error` field, and
2. require only `thiserror = "1.0.30"`/`"1.0.31"`, so a fresh resolution
   selects the newest `thiserror 1.x` (1.0.69).

`thiserror-impl 1.0.69` always generates a `provide` method for a backtrace
field. The method uses `std::error::Request` and the
`thiserror::__private::ThiserrorProvide` trait. `thiserror` exports that trait
only when its build-script probe finds the current `error_generic_member_access`
API. The 2022 nightly predates that API, so the generated code reports the
issue's four errors: E0432, E0412, E0658 and E0599. The issue's error lines
match the reproduction exactly (`links.rs:4:10` and `links.rs:18:13`).

Pinning `thiserror` to 1.0.31 lets `platform-data` compile but exposes a second
drift: `platform-data 0.1.0-beta.3`, published a week after the `doublets`
pre-release, changed bounds that it uses. The historical release builds only on
`nightly-2022-07-23` with `thiserror =1.0.31` and `platform-data =0.1.0-beta.1`.
On stable, it fails with E0554 (`#![feature]` on the stable channel). Published
crates are immutable, so the historical release cannot be repaired.

Main fixed the issue in the stable migration (PRs 48 and 26). Every release
since 0.2.0 declares Rust 1.85, uses `thiserror` 2 without `#[backtrace]`, and
depends on stable `platform-*` releases. The original PR 29 draft instead
deleted the tracked lockfile, ignored committed logs, pinned
`nightly-2024-01-01`, and declared the crate unbuildable. The merge discards
those changes.

## Verification

```bash
bash experiments/issue-18/verify.sh            # packaged checkout (CI, all OSes)
bash experiments/issue-18/verify.sh crates-io  # published release (CI, Linux)
bash experiments/issue-18/reproduce.sh         # historical; installs a 2022 nightly
```

`verify.sh` uses stable with `RUSTC_BOOTSTRAP` unset, one build job, and no
development debug info. It creates a standalone project under ignored
`target/`, adds `doublets` with `cargo add`, and generates a fresh lockfile so
every dependency uses its newest compatible release, as in the issue. In
`package` mode, the dependency is the `cargo package` output of the checkout,
which is exactly what the next release will publish. The script rejects
historical nightly-only crates in `cargo tree`. It then runs
[consumer.rs](consumer.rs) with default and full features: CRUD, iterators, and
the derived `Display`/`source()` impls on both storage layouts. Logs are
written to `target/issue-18-logs/<mode>/`.

`reproduce.sh` uses a private Cargo home under `target/`. Stable Cargo's
MSRV-aware resolver writes a lockfile that the 2022 Cargo can read, still
selecting `thiserror 1.0.69`. The script requires all four errors in
`platform-data-0.1.0-beta.3/src/links.rs`. It then pins the 2022 versions and
requires a successful build, and finally requires E0554 on stable.

Local results (Linux, stable Rust 1.99.0, `nightly-2022-07-23` = rustc
1.64.0-nightly 848090dcd):

- `reproduce.sh`: resolved `doublets 0.1.0-pre+beta.15`,
  `platform-data 0.1.0-beta.3`, `thiserror 1.0.69`; reproduced all four errors;
  the pinned 2022 set built; stable reported E0554.
- `verify.sh package`: `doublets 0.21.0` from the packaged checkout resolved
  `platform-data 2.0.0` and `thiserror 2.0.21`; default and full runs passed on
  both layouts.
- `verify.sh crates-io`: `cargo add doublets` added `doublets v0.21.0` from
  crates.io; default and full runs passed.
- Migration: in a project locked to `=0.1.0-pre`, a plain `cargo add doublets`
  kept the old requirement. `cargo remove doublets` and then `cargo add doublets`
  moved it to `0.21.0` and `thiserror 2.0.21`, and the consumer ran on stable.
