# Stable Rust Support

All Rust packages in this repository build on stable Rust, including their
normal, development, optional, build, and procedural macro dependencies.
`rust-toolchain.toml` selects `stable`; no build needs `RUSTC_BOOTSTRAP`.

The `doublets` library declares Rust 1.85 as its minimum supported version.
Use the latest stable toolchain for repository development: the benchmark and
FFI procedural macro dependencies have newer minimum versions (the FFI crate
requires Rust 1.88 or newer).

## Packages and dependencies

- The main workspace contains `doublets` and `integration`. Full storage,
  decorators, sequences, and transactions are available on stable Rust.
- Decorators are part of `doublets::decorators`, available without optional
  features. They provide `new(store)` constructors and the `DecoratorsExt`
  builder; callbacks use the stable `Flow` API.
- `doublets-ffi` and its `env-decorators` and `ffi-attributes` procedural macros
  also build on stable. The C exports retain their names and integer widths.
- Platform dependencies (`platform-num`, `platform-data`, `platform-mem`, and
  `platform-trees`) come from crates.io; no git submodules are required.
- Release scripts declare their dependencies in embedded Cargo manifests.
  CI compiles each manifest without invoking publishing or release operations.

## Verification

```bash
rustup toolchain install stable
rustup component add rustfmt clippy
cargo +stable test --workspace --all-features --locked
cargo +stable test --workspace --doc --locked
bash experiments/issue-22/check-packages.sh
cargo +stable test -p doublets --test decorator_consolidation --no-default-features --locked
cargo +stable test --manifest-path doublets-ffi/Cargo.toml --locked
```

The package check explicitly selects stable and unsets `RUSTC_BOOTSTRAP`. It
compiles all targets and all features of all five package manifests, including
those excluded from the main workspace. The CI test matrix runs it on Linux,
macOS, and Windows along with the decorator and FFI regressions.

## Optional Miri checks

`ci/miri.sh` installs a separate nightly toolchain because Miri requires nightly.
It is an optional diagnostic tool, independent of normal builds, tests, and
releases. The ordinary Travis test entry also uses stable Rust.

The main workspace migration landed in PR 48. PR 26 completed the standalone
package migration. Decorator coverage now lives in the main library; CI also
checks the standalone FFI packages and their dependencies.

## Build from source

Clone the current repository and build the workspace from its root:

```bash
git clone https://github.com/linksplatform/doublets-rs.git
cd doublets-rs
rustup toolchain install stable
cargo +stable build --workspace --locked
```

Git submodule initialization and nightly feature flags are unnecessary. The
tracked lockfile keeps source builds reproducible. To compile optional features:

```bash
cargo +stable build --workspace --all-features --locked
```

FFI is an independent package, excluded from the main workspace. Build it
explicitly with its own tracked lockfile:

```bash
cargo +stable build --manifest-path doublets-ffi/Cargo.toml --release --locked
```

For machines with limited memory, set `CARGO_BUILD_JOBS=1` and
`CARGO_PROFILE_DEV_DEBUG=0` before development builds.

## Troubleshoot historical build errors

[Issue 23](https://github.com/linksplatform/doublets-rs/issues/23) reported a real
stable-channel failure in older versions: `platform-num 0.1.0-aplha.1` and
`platform-data 0.1.0-beta.3` used nightly features such as `step_trait`,
`trait_alias`, and `try_trait_v2`. Those versions can report E0554, unknown
feature errors, or unresolved `std::default::default` imports. Merely removing
their feature attributes leaves unstable APIs in use.

Current source uses stable platform dependencies and retains both storage
layouts and all optional features. If those old dependency versions still
appear in a build log:

1. Check `rustc +stable --version` and `cargo +stable --version`. Rust 1.81,
   used in the report, is below the current library's declared minimum of 1.85.
   Use latest stable for repository development and FFI.
2. Update the source checkout to a current revision, or update your consuming
   project's `doublets` dependency to the version in the
   [installation instructions](README.md#installation). In a consuming project,
   run `cargo update -p doublets` after updating its manifest.
3. Check `cargo +stable tree` for old direct `platform-*` dependencies or local
   path/`[patch.crates-io]` overrides that keep the historical crates in use.
   Update those declarations too; updating a lockfile alone cannot override
   an old manifest's version constraint.
4. Build again using the source commands above, without `RUSTC_BOOTSTRAP`.
   Keep the required `data`, `mem`, and `trees` dependencies enabled.

The issue's `nightly-2022-07-23` suggestion applies to historical revisions.
The current supported build uses stable. For a small reproduction of the
original errors and the investigation record, see
[experiments/issue-23](experiments/issue-23/README.md).

## Git dependencies and issue 19

[Issue 19](https://github.com/linksplatform/doublets-rs/issues/19) used Git
revision `5522d91c` and Rust 1.81 nightly from June 2024. Its submodule copy of
`platform-data 0.1.0-beta.3` used opaque associated types without the required
feature gate, removed features (`const_deref`, `const_result_drop`,
`const_convert`), and obsolete `~const` bounds. Switching toolchains alone
cannot repair that source.

The maintained source builds on stable, including when `doublets` is used as
an external Git dependency. The stable migration in PRs 48 and 26 replaced
those dependencies with stable crates.io releases. Current iterator methods
use stable return-position `impl Iterator`; no `impl_trait_in_assoc_type`
feature gate is needed. Both storage layouts and the optional `full` features
remain available.

For a consuming project using the Git declaration in the
[installation instructions](README.md#installation):

```bash
rustup toolchain install stable
cargo update -p doublets
cargo +stable run
```

If the old errors persist, check `cargo +stable tree` and the `doublets` entry
in `Cargo.lock`. Remove or update a historical `rev`/`tag` pin or local patch
before updating the lockfile; `cargo update` preserves explicit revision pins.
Consumer projects select their own toolchain, so use `+stable` explicitly when
their configuration still selects nightly. Stable Rust 1.85 is the library's
minimum; use latest stable for development and FFI.

CI runs `bash experiments/issue-19/verify.sh` on Linux, macOS, and Windows. It
creates an independent consumer, resolves `doublets` through a Git URL pinned
to the checked-out commit, and runs CRUD and iterator assertions on both layouts
with default and full features, with `RUSTC_BOOTSTRAP` unset. See the
[investigation record](experiments/issue-19/README.md) for the tested revision
and reproduction commands.
