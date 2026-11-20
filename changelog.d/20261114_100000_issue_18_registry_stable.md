---
bump: patch
---

### Fixed
- Document the root cause of the `unresolved import thiserror` errors from
  `cargo add doublets` in issue 18 and how to upgrade projects that still
  resolve the 2022 nightly-only pre-release.
- Verify the packaged crate as a registry dependency with freshly resolved
  dependencies on stable Rust across Linux, macOS and Windows, and verify
  `cargo add doublets` from crates.io on Linux.
