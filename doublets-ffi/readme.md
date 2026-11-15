# Compatibility FFI facade

This package is unpublished and reexports `doublets::ffi`. New consumers should
build the primary package with `cargo build -p doublets --features ffi --release`.
See [the FFI guide](../docs/ffi.md) for native artifact names and migration steps.

Existing source builds can still run
`cargo build --manifest-path doublets-ffi/Cargo.toml --release --locked`
to produce a library named `doublets_ffi`.
