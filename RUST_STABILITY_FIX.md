# Build status for issue 23

The old stable-Rust failure in
[issue 23](https://github.com/linksplatform/doublets-rs/issues/23) was valid for
the historical platform dependencies. It has already been resolved on main by
the stable migration in PRs 48 and 26. The maintained library declares Rust 1.85
as its minimum; use latest stable for repository development and FFI builds.

Follow [Stable Rust Support](STABLE_RUST.md) for source and FFI build commands,
supported packages, verification, and troubleshooting old dependency versions.
The [issue 23 investigation](experiments/issue-23/README.md) includes a small
script that reproduces the original stable-channel dependency errors.

The historical Git/nightly failure in
[issue 19](https://github.com/linksplatform/doublets-rs/issues/19) is also
resolved by the stable migration. The
[Git consumer check](experiments/issue-19/README.md) verifies default/full
features and both storage layouts on stable; the
[Git troubleshooting guide](STABLE_RUST.md#git-dependencies-and-issue-19)
explains how to update old revision pins and lockfiles.

The original draft of this document incorrectly claimed a complete fix while
required dependencies were disabled and described edits that were not recorded
in the dependency submodules. PR 25 now preserves main's complete stable
implementation, including memory storage, trees, decorators, and optional
features. No dependency disabling or bootstrap override is required.
