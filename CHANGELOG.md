# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- changelog-insert-here -->





## [0.5.0] - 2026-08-29

### Added
- Configured `rust-toolchain.toml` with nightly-2022-08-22 for reproducible builds
- Added comprehensive CONTRIBUTING.md with development guidelines
- Added changelog fragment system with `changelog.d/` directory for conflict-free versioning
- Added utility scripts for version management and release automation
- Added `.pre-commit-config.yaml` for code quality hooks
- Added code coverage reporting with cargo-llvm-cov and Codecov integration
- Added extensive test coverage for data module (Link, Doublet, Error, Handler/Fuse)
- Added comprehensive tests for traits module (Doublets, DoubletsExt, Links)
- Added tests for memory module components (LinksHeader, stores)

### Changed
- Improved CI/CD workflow with template best practices
- Added concurrency control to prevent duplicate CI runs
- Added automatic release workflow based on changelog fragments
- Added manual release dispatch option for maintainers

### Fixed
- Updated README installation guidance to use doublets 0.3.0 and stable Rust 1.85 or newer.
- Added a CI documentation check that prevents stale nightly Rust and crate version guidance from returning.

### Added
- Ported the whole `Platform.Data.Doublets` decorator layer to Rust as the new `doublets::decorators` module: `UniquenessValidator`, `UniquenessResolver`, `CascadeUniquenessAndUsagesResolver`, `UsagesValidator`, `CascadeUsagesResolver`, `InnerReferenceExistenceValidator`, `ItselfConstantToSelfReferenceResolver`, `NullConstantToSelfReferenceResolver`, `NonExistentDependenciesCreator`, `NonNullContentsLinkDeletionResolver`, `LoggingDecorator` and `NoExceptionsDecorator`.
- Added the `DecoratorsExt` builder, which takes a store by value and returns the concrete composed type, and the zero-sized `Validate` / `Resolve` / `CascadeResolve` policy markers selected through the `UniquenessPolicy` and `UsagesPolicy` traits.
- Added `doublets/examples/uniqueness.rs`, showing duplicate `(source, target)` creation with and without a uniqueness policy.
- Added a fusion check (`integration/tests/fusion.rs`) asserting that a nine-layer decorator stack disassembles to exactly the same machine code as the bare store.

### Fixed
- `unit::Store::update_links` now reports the previous link to write handlers; it used to pass the incoming change as both `before` and `after`, unlike `split::Store`.

### Changed
- **BREAKING**: Migrated from nightly Rust to stable Rust (1.85+)
- **BREAKING**: `Fuse` no longer implements `FnMut`; use `.call()` method instead
- **BREAKING**: Removed `Handler` trait; handlers now use `FnMut(Link<T>, Link<T>) -> Flow` directly
- Migrated `platform-mem` dependency from git submodule to crates.io v0.3.0
- Migrated `platform-trees` dependency from git submodule to crates.io v0.3.3
- Replaced JavaScript CI/CD scripts with Rust scripts matching template best practices
- Updated CI/CD pipeline to use stable Rust toolchain with release.yml pattern
- Updated `rustfmt.toml` for stable Rust compatibility

### Fixed
- Fixed CI/CD release pipeline failing on Cargo workspace repositories
- All release scripts now correctly resolve the publishable member crate's Cargo.toml instead of reading the workspace root manifest
- Fixed `bump-version.rs` failing on versions with pre-release suffixes (e.g. `0.1.0-pre+beta.15`)
- Fixed `publish-crate.rs` to pass `-p <package>` flag for workspace repos

### Fixed
- Replaced stale README workflow badges with the active branch-scoped CI/CD Pipeline badge.
- Added a documentation badge check to prevent badges from pointing at deleted or unscoped legacy workflows.

## [0.4.0] - 2026-05-29

### Added
- Configured `rust-toolchain.toml` with nightly-2022-08-22 for reproducible builds
- Added comprehensive CONTRIBUTING.md with development guidelines
- Added changelog fragment system with `changelog.d/` directory for conflict-free versioning
- Added utility scripts for version management and release automation
- Added `.pre-commit-config.yaml` for code quality hooks
- Added code coverage reporting with cargo-llvm-cov and Codecov integration
- Added extensive test coverage for data module (Link, Doublet, Error, Handler/Fuse)
- Added comprehensive tests for traits module (Doublets, DoubletsExt, Links)
- Added tests for memory module components (LinksHeader, stores)

### Changed
- Improved CI/CD workflow with template best practices
- Added concurrency control to prevent duplicate CI runs
- Added automatic release workflow based on changelog fragments
- Added manual release dispatch option for maintainers

### Fixed
- Updated README installation guidance to use doublets 0.3.0 and stable Rust 1.85 or newer.
- Added a CI documentation check that prevents stale nightly Rust and crate version guidance from returning.

### Changed
- **BREAKING**: Migrated from nightly Rust to stable Rust (1.85+)
- **BREAKING**: `Fuse` no longer implements `FnMut`; use `.call()` method instead
- **BREAKING**: Removed `Handler` trait; handlers now use `FnMut(Link<T>, Link<T>) -> Flow` directly
- Migrated `platform-mem` dependency from git submodule to crates.io v0.3.0
- Migrated `platform-trees` dependency from git submodule to crates.io v0.3.3
- Replaced JavaScript CI/CD scripts with Rust scripts matching template best practices
- Updated CI/CD pipeline to use stable Rust toolchain with release.yml pattern
- Updated `rustfmt.toml` for stable Rust compatibility

### Fixed
- Fixed CI/CD release pipeline failing on Cargo workspace repositories
- All release scripts now correctly resolve the publishable member crate's Cargo.toml instead of reading the workspace root manifest
- Fixed `bump-version.rs` failing on versions with pre-release suffixes (e.g. `0.1.0-pre+beta.15`)
- Fixed `publish-crate.rs` to pass `-p <package>` flag for workspace repos

### Fixed
- Replaced stale README workflow badges with the active branch-scoped CI/CD Pipeline badge.
- Added a documentation badge check to prevent badges from pointing at deleted or unscoped legacy workflows.

## [0.3.0] - 2026-04-18

### Added
- Configured `rust-toolchain.toml` with nightly-2022-08-22 for reproducible builds
- Added comprehensive CONTRIBUTING.md with development guidelines
- Added changelog fragment system with `changelog.d/` directory for conflict-free versioning
- Added utility scripts for version management and release automation
- Added `.pre-commit-config.yaml` for code quality hooks
- Added code coverage reporting with cargo-llvm-cov and Codecov integration
- Added extensive test coverage for data module (Link, Doublet, Error, Handler/Fuse)
- Added comprehensive tests for traits module (Doublets, DoubletsExt, Links)
- Added tests for memory module components (LinksHeader, stores)

### Changed
- Improved CI/CD workflow with template best practices
- Added concurrency control to prevent duplicate CI runs
- Added automatic release workflow based on changelog fragments
- Added manual release dispatch option for maintainers

### Changed
- **BREAKING**: Migrated from nightly Rust to stable Rust (1.85+)
- **BREAKING**: `Fuse` no longer implements `FnMut`; use `.call()` method instead
- **BREAKING**: Removed `Handler` trait; handlers now use `FnMut(Link<T>, Link<T>) -> Flow` directly
- Migrated `platform-mem` dependency from git submodule to crates.io v0.3.0
- Migrated `platform-trees` dependency from git submodule to crates.io v0.3.3
- Replaced JavaScript CI/CD scripts with Rust scripts matching template best practices
- Updated CI/CD pipeline to use stable Rust toolchain with release.yml pattern
- Updated `rustfmt.toml` for stable Rust compatibility

### Fixed
- Fixed CI/CD release pipeline failing on Cargo workspace repositories
- All release scripts now correctly resolve the publishable member crate's Cargo.toml instead of reading the workspace root manifest
- Fixed `bump-version.rs` failing on versions with pre-release suffixes (e.g. `0.1.0-pre+beta.15`)
- Fixed `publish-crate.rs` to pass `-p <package>` flag for workspace repos

### Fixed
- Replaced stale README workflow badges with the active branch-scoped CI/CD Pipeline badge.
- Added a documentation badge check to prevent badges from pointing at deleted or unscoped legacy workflows.

## [0.2.0] - 2026-04-14

### Added
- Configured `rust-toolchain.toml` with nightly-2022-08-22 for reproducible builds
- Added comprehensive CONTRIBUTING.md with development guidelines
- Added changelog fragment system with `changelog.d/` directory for conflict-free versioning
- Added utility scripts for version management and release automation
- Added `.pre-commit-config.yaml` for code quality hooks
- Added code coverage reporting with cargo-llvm-cov and Codecov integration
- Added extensive test coverage for data module (Link, Doublet, Error, Handler/Fuse)
- Added comprehensive tests for traits module (Doublets, DoubletsExt, Links)
- Added tests for memory module components (LinksHeader, stores)

### Changed
- Improved CI/CD workflow with template best practices
- Added concurrency control to prevent duplicate CI runs
- Added automatic release workflow based on changelog fragments
- Added manual release dispatch option for maintainers

### Changed
- **BREAKING**: Migrated from nightly Rust to stable Rust (1.85+)
- **BREAKING**: `Fuse` no longer implements `FnMut`; use `.call()` method instead
- **BREAKING**: Removed `Handler` trait; handlers now use `FnMut(Link<T>, Link<T>) -> Flow` directly
- Migrated `platform-mem` dependency from git submodule to crates.io v0.3.0
- Migrated `platform-trees` dependency from git submodule to crates.io v0.3.3
- Replaced JavaScript CI/CD scripts with Rust scripts matching template best practices
- Updated CI/CD pipeline to use stable Rust toolchain with release.yml pattern
- Updated `rustfmt.toml` for stable Rust compatibility

### Fixed
- Fixed CI/CD release pipeline failing on Cargo workspace repositories
- All release scripts now correctly resolve the publishable member crate's Cargo.toml instead of reading the workspace root manifest
- Fixed `bump-version.rs` failing on versions with pre-release suffixes (e.g. `0.1.0-pre+beta.15`)
- Fixed `publish-crate.rs` to pass `-p <package>` flag for workspace repos

## [0.1.0-pre+beta.15] - Initial Release

### Added
- Initial implementation of doublets library
- Memory-efficient doublets storage
- Support for split and unit storage modes
- FFI bindings for cross-language support
- Comprehensive data structures for link management