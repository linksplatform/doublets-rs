---
bump: patch
---

### Fixed
- Fixed CI/CD release pipeline failing on Cargo workspace repositories
- All release scripts now correctly resolve the publishable member crate's Cargo.toml instead of reading the workspace root manifest
- Fixed `bump-version.rs` failing on versions with pre-release suffixes (e.g. `0.1.0-pre+beta.15`)
- Fixed `publish-crate.rs` to pass `-p <package>` flag for workspace repos
