# Stable Rust Support

`doublets-rs` builds on stable Rust. The workspace uses `rust-toolchain.toml`
with `channel = "stable"`, and the `doublets` crate declares Rust 1.85 as its
minimum supported Rust version.

## Current Setup

- The crate no longer uses `#![feature(...)]` attributes.
- CI uses `.github/workflows/release.yml` with `dtolnay/rust-toolchain@stable`.
- Platform dependencies are consumed from crates.io:
  `platform-num`, `platform-data`, `platform-mem`, and `platform-trees`.
- The old `dev-deps/` git submodules are no longer required.
- Miri, when used manually through `ci/miri.sh`, still installs a nightly
  toolchain because Miri itself requires nightly Rust. This does not make the
  crate require nightly for normal builds, tests, or releases.

## Usage

```bash
cargo check
cargo test --all-features
```

For local development, install the standard stable components:

```bash
rustup component add rustfmt clippy
```

## Related Work

The full migration was completed through PR 48, which removed the nightly-only
language features, migrated platform dependencies to crates.io releases, and
updated CI/CD to use the stable toolchain. This document keeps issue 22's
stable Rust requirement visible from the repository root.
