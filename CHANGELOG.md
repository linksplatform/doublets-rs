# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- changelog-insert-here -->












## [0.12.0] - 2026-10-29

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

### Added
- Add `doublets::sequences` with balanced variants, C# hybrid raw-number converters,
  conversion caching, a default stack and an iterative sequence walker.
- Add reusable target criteria and store-backed UTF-16 symbol/string converters
  compatible with `Platform.Data.Doublets.Sequences` 0.6.5, including raw UTF-16
  methods, checked address ranges, cycle detection and typed conversion errors.
- Add a Unicode string storage example and interoperability/regression tests.

### Added
- Generic `TransactionsDecorator` with commit/rollback handles, rollback on drop, reversible transitions, memory and durable checksummed file journals, and recovery of interrupted transactions.
- Transaction builder, runnable example, optional diagnostics, and tests for both store backends, I/O failures, journal corruption and subprocess crash recovery.

### Fixed
- Classify split-store self references before detaching index nodes, preventing an update from moving a live reference to the external index and later panicking during rollback.

### Fixed
- `Doublets::rebase_with` now updates each usage once and replaces both endpoints together when both reference the old address, including through the automatic uniqueness and usages resolution stack.

### Fixed
- Count and list each usage once when a link references the same value as both source and target, while excluding the referenced link itself.
- Delete overlapping usage and query matches once, avoiding spurious `NotExists` errors and duplicate deletion callbacks.

### Fixed
- Keep split-store source and target references searchable when their IDs are
  created later, including reuse of deleted IDs. Move existing external index
  entries directly into the new ID's internal indexes before creation callbacks.
- Count concrete split-store pairs with absent sources by both endpoints instead
  of counting every usage of the source.

### Fixed
- Return the complete allocation after memory growth so unit and split stores retain their headers, earlier links, and full capacity with heap and file-mapped backends.
- Add regressions for initial capacity, repeated growth, preserved link queries, and backends that return only the grown region.
- Count empty endpoint indexes as zero instead of reading free-list header fields, with a regression for deleted-slot reuse exposed by the updated random test sequence.

### Changed
- Update all Rust dependencies to their latest stable releases, including development tools, embedded release scripts, and excluded legacy crate manifests; refresh the workspace lockfile and update affected APIs.
- Update GitHub Actions and pre-commit hooks to their current stable major releases.

### Fixed
- Report nested resets to `(0, 0)` before cascaded deletions in the automatic uniqueness and usages resolution stack, matching C# callback order while preserving cycle termination.
- Correct decorator ordering documentation and synchronize the README dependency example with the released crate version.

### Fixed
- Delete all links from stores with holes by enumerating existing addresses in reverse order, skipping links already removed by cascading deletion.

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

## [0.11.0] - 2026-10-29

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

### Added
- Add `doublets::sequences` with balanced variants, C# hybrid raw-number converters,
  conversion caching, a default stack and an iterative sequence walker.
- Add reusable target criteria and store-backed UTF-16 symbol/string converters
  compatible with `Platform.Data.Doublets.Sequences` 0.6.5, including raw UTF-16
  methods, checked address ranges, cycle detection and typed conversion errors.
- Add a Unicode string storage example and interoperability/regression tests.

### Added
- Generic `TransactionsDecorator` with commit/rollback handles, rollback on drop, reversible transitions, memory and durable checksummed file journals, and recovery of interrupted transactions.
- Transaction builder, runnable example, optional diagnostics, and tests for both store backends, I/O failures, journal corruption and subprocess crash recovery.

### Fixed
- Classify split-store self references before detaching index nodes, preventing an update from moving a live reference to the external index and later panicking during rollback.

### Fixed
- `Doublets::rebase_with` now updates each usage once and replaces both endpoints together when both reference the old address, including through the automatic uniqueness and usages resolution stack.

### Fixed
- Count and list each usage once when a link references the same value as both source and target, while excluding the referenced link itself.
- Delete overlapping usage and query matches once, avoiding spurious `NotExists` errors and duplicate deletion callbacks.

### Fixed
- Keep split-store source and target references searchable when their IDs are
  created later, including reuse of deleted IDs. Move existing external index
  entries directly into the new ID's internal indexes before creation callbacks.
- Count concrete split-store pairs with absent sources by both endpoints instead
  of counting every usage of the source.

### Fixed
- Return the complete allocation after memory growth so unit and split stores retain their headers, earlier links, and full capacity with heap and file-mapped backends.
- Add regressions for initial capacity, repeated growth, preserved link queries, and backends that return only the grown region.
- Count empty endpoint indexes as zero instead of reading free-list header fields, with a regression for deleted-slot reuse exposed by the updated random test sequence.

### Changed
- Update all Rust dependencies to their latest stable releases, including development tools, embedded release scripts, and excluded legacy crate manifests; refresh the workspace lockfile and update affected APIs.
- Update GitHub Actions and pre-commit hooks to their current stable major releases.

### Fixed
- Report nested resets to `(0, 0)` before cascaded deletions in the automatic uniqueness and usages resolution stack, matching C# callback order while preserving cycle termination.
- Correct decorator ordering documentation and synchronize the README dependency example with the released crate version.

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

## [0.10.0] - 2026-10-28

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

### Added
- Add `doublets::sequences` with balanced variants, C# hybrid raw-number converters,
  conversion caching, a default stack and an iterative sequence walker.
- Add reusable target criteria and store-backed UTF-16 symbol/string converters
  compatible with `Platform.Data.Doublets.Sequences` 0.6.5, including raw UTF-16
  methods, checked address ranges, cycle detection and typed conversion errors.
- Add a Unicode string storage example and interoperability/regression tests.

### Added
- Generic `TransactionsDecorator` with commit/rollback handles, rollback on drop, reversible transitions, memory and durable checksummed file journals, and recovery of interrupted transactions.
- Transaction builder, runnable example, optional diagnostics, and tests for both store backends, I/O failures, journal corruption and subprocess crash recovery.

### Fixed
- Classify split-store self references before detaching index nodes, preventing an update from moving a live reference to the external index and later panicking during rollback.

### Fixed
- `Doublets::rebase_with` now updates each usage once and replaces both endpoints together when both reference the old address, including through the automatic uniqueness and usages resolution stack.

### Fixed
- Count and list each usage once when a link references the same value as both source and target, while excluding the referenced link itself.
- Delete overlapping usage and query matches once, avoiding spurious `NotExists` errors and duplicate deletion callbacks.

### Fixed
- Keep split-store source and target references searchable when their IDs are
  created later, including reuse of deleted IDs. Move existing external index
  entries directly into the new ID's internal indexes before creation callbacks.
- Count concrete split-store pairs with absent sources by both endpoints instead
  of counting every usage of the source.

### Fixed
- Return the complete allocation after memory growth so unit and split stores retain their headers, earlier links, and full capacity with heap and file-mapped backends.
- Add regressions for initial capacity, repeated growth, preserved link queries, and backends that return only the grown region.
- Count empty endpoint indexes as zero instead of reading free-list header fields, with a regression for deleted-slot reuse exposed by the updated random test sequence.

### Changed
- Update all Rust dependencies to their latest stable releases, including development tools, embedded release scripts, and excluded legacy crate manifests; refresh the workspace lockfile and update affected APIs.
- Update GitHub Actions and pre-commit hooks to their current stable major releases.

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

## [0.9.0] - 2026-10-28

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

### Added
- Add `doublets::sequences` with balanced variants, C# hybrid raw-number converters,
  conversion caching, a default stack and an iterative sequence walker.
- Add reusable target criteria and store-backed UTF-16 symbol/string converters
  compatible with `Platform.Data.Doublets.Sequences` 0.6.5, including raw UTF-16
  methods, checked address ranges, cycle detection and typed conversion errors.
- Add a Unicode string storage example and interoperability/regression tests.

### Added
- Generic `TransactionsDecorator` with commit/rollback handles, rollback on drop, reversible transitions, memory and durable checksummed file journals, and recovery of interrupted transactions.
- Transaction builder, runnable example, optional diagnostics, and tests for both store backends, I/O failures, journal corruption and subprocess crash recovery.

### Fixed
- Classify split-store self references before detaching index nodes, preventing an update from moving a live reference to the external index and later panicking during rollback.

### Fixed
- `Doublets::rebase_with` now updates each usage once and replaces both endpoints together when both reference the old address, including through the automatic uniqueness and usages resolution stack.

### Fixed
- Count and list each usage once when a link references the same value as both source and target, while excluding the referenced link itself.
- Delete overlapping usage and query matches once, avoiding spurious `NotExists` errors and duplicate deletion callbacks.

### Fixed
- Keep split-store source and target references searchable when their IDs are
  created later, including reuse of deleted IDs. Move existing external index
  entries directly into the new ID's internal indexes before creation callbacks.
- Count concrete split-store pairs with absent sources by both endpoints instead
  of counting every usage of the source.

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

## [0.8.0] - 2026-10-08

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

### Added
- Add `doublets::sequences` with balanced variants, C# hybrid raw-number converters,
  conversion caching, a default stack and an iterative sequence walker.
- Add reusable target criteria and store-backed UTF-16 symbol/string converters
  compatible with `Platform.Data.Doublets.Sequences` 0.6.5, including raw UTF-16
  methods, checked address ranges, cycle detection and typed conversion errors.
- Add a Unicode string storage example and interoperability/regression tests.

### Added
- Generic `TransactionsDecorator` with commit/rollback handles, rollback on drop, reversible transitions, memory and durable checksummed file journals, and recovery of interrupted transactions.
- Transaction builder, runnable example, optional diagnostics, and tests for both store backends, I/O failures, journal corruption and subprocess crash recovery.

### Fixed
- Classify split-store self references before detaching index nodes, preventing an update from moving a live reference to the external index and later panicking during rollback.

### Fixed
- `Doublets::rebase_with` now updates each usage once and replaces both endpoints together when both reference the old address, including through the automatic uniqueness and usages resolution stack.

### Fixed
- Count and list each usage once when a link references the same value as both source and target, while excluding the referenced link itself.
- Delete overlapping usage and query matches once, avoiding spurious `NotExists` errors and duplicate deletion callbacks.

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

## [0.7.0] - 2026-10-08

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

### Added
- Add `doublets::sequences` with balanced variants, C# hybrid raw-number converters,
  conversion caching, a default stack and an iterative sequence walker.
- Add reusable target criteria and store-backed UTF-16 symbol/string converters
  compatible with `Platform.Data.Doublets.Sequences` 0.6.5, including raw UTF-16
  methods, checked address ranges, cycle detection and typed conversion errors.
- Add a Unicode string storage example and interoperability/regression tests.

### Added
- Generic `TransactionsDecorator` with commit/rollback handles, rollback on drop, reversible transitions, memory and durable checksummed file journals, and recovery of interrupted transactions.
- Transaction builder, runnable example, optional diagnostics, and tests for both store backends, I/O failures, journal corruption and subprocess crash recovery.

### Fixed
- Classify split-store self references before detaching index nodes, preventing an update from moving a live reference to the external index and later panicking during rollback.

### Fixed
- `Doublets::rebase_with` now updates each usage once and replaces both endpoints together when both reference the old address, including through the automatic uniqueness and usages resolution stack.

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

## [0.6.0] - 2026-10-08

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

### Added
- Add `doublets::sequences` with balanced variants, C# hybrid raw-number converters,
  conversion caching, a default stack and an iterative sequence walker.
- Add reusable target criteria and store-backed UTF-16 symbol/string converters
  compatible with `Platform.Data.Doublets.Sequences` 0.6.5, including raw UTF-16
  methods, checked address ranges, cycle detection and typed conversion errors.
- Add a Unicode string storage example and interoperability/regression tests.

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
