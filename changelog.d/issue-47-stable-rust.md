### Changed
- **BREAKING**: Migrated from nightly Rust to stable Rust (1.85+)
- **BREAKING**: `Fuse` no longer implements `FnMut`; use `.call()` method instead
- **BREAKING**: Removed `Handler` trait; handlers now use `FnMut(Link<T>, Link<T>) -> Flow` directly
- Migrated `platform-mem` dependency from git submodule to crates.io v0.3.0
- Migrated `platform-trees` dependency from git submodule to crates.io v0.3.3
- Replaced JavaScript CI/CD scripts with Rust scripts matching template best practices
- Updated CI/CD pipeline to use stable Rust toolchain with release.yml pattern
- Updated `rustfmt.toml` for stable Rust compatibility
