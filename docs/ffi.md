# FFI from the primary package

The public crates.io package remains **`doublets`**. `doublets-rs` is the
repository name, not a second package. Keeping the existing package name
preserves Rust dependency declarations and the published release history.

Enable FFI in a Rust consumer:

```toml
[dependencies]
doublets = { version = "0.22", features = ["ffi"] }
```

The consolidated FFI API ships in the next release after 0.21.0. Until that
release is published, use the Git dependency with `features = ["ffi"]`.
Rust callers can import `doublets::ffi::*`. The default features keep FFI off;
`ffi` also works with `default-features = false`. It enables the platform
reexports needed by the bindings and their optional logging dependencies.

Build a native library from a source checkout:

```sh
cargo build -p doublets --features ffi --release --locked
```

Outputs are in `target/release`: `libdoublets.so` / `libdoublets.a` on Linux,
`libdoublets.dylib` / `libdoublets.a` on macOS, and `doublets.dll` /
`doublets.lib` on MSVC Windows. Set your loader or P/Invoke library name to
`doublets` (or rename the binary to match your existing loader configuration).
The exported `ByteLinks_*`, `UInt16Links_*`, `UInt32Links_*`, `UInt64Links_*`,
`setup_shared_logger`, and `init_fmt_logger` names and C layouts are unchanged.
Pointers, queries and callbacks must remain valid for each call; drop each
opaque handle exactly once with the same integer specialization that created it.

The unpublished `doublets-ffi` source package remains as a compatibility
facade for existing source builds, including its `libdoublets_ffi` artifact
name. New consumers should use the primary package. `ffi-attributes` and
`env-decorators` remain unpublished development tools. The primary crate uses
ordinary declarative macros and has no dependency on either tool, so the
crates.io archive can build FFI independently of this repository.

## Trusted publishing and verification

Both automatic and manual instant releases in `.github/workflows/release.yml`
use `rust-lang/crates-io-auth-action@v1` and job-scoped `id-token: write`.
Only `cargo publish --package doublets` runs. No long-lived Cargo secret is used.
See the [official authentication action](https://github.com/rust-lang/crates-io-auth-action)
and [crates.io trusted publishing setup](https://crates.io/docs/trusted-publishing).

A crates.io owner must configure the trusted publisher for the existing
`doublets` package: GitHub owner `linksplatform`, repository `doublets-rs`,
workflow filename `release.yml`. The workflow uses no GitHub environment.
Release jobs run after checks on `main`; PR checks never request a publishing
token. If this external configuration is missing, the release fails explicitly
at authentication; configure it on crates.io and rerun the release workflow.

A changelog fragment triggers the automatic version bump after merge. Versions
are managed by the release scripts rather than manually changed in PRs.
CI verifies the package archive and compiles its FFI feature without local
path dependencies, tests all integer widths, and loads the native library via
`experiments/issue-22/ffi-smoke.py` on Linux. To repeat focused checks:

```sh
python3 experiments/issue-92/check-release.py
cargo test -p doublets --features ffi --test ffi --locked
cargo package -p doublets --locked
```

After the release job succeeds, verify `cargo search doublets --limit 1` and
compare the reported version with that job's published version. An existing
search result alone does not prove the new OIDC workflow has published a release.
