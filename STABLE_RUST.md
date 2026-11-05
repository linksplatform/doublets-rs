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
