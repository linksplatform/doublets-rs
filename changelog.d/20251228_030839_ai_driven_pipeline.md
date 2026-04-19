---
bump: minor
---

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
