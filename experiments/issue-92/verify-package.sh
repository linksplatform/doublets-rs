#!/usr/bin/env bash
# Verify FFI using only the archive that will be uploaded to crates.io.
set -euo pipefail
cd "$(dirname "$0")/../.."
python3 experiments/issue-92/check-release.py
python3 experiments/issue-92/test-publish.py
cargo package -p doublets --locked --no-verify
version=$(python3 -c 'import tomllib; print(tomllib.load(open("doublets/Cargo.toml", "rb"))["package"]["version"])')
tar -xzf "target/package/doublets-$version.crate" -C target/package
cargo test --manifest-path "target/package/doublets-$version/Cargo.toml" \
    --features ffi --test ffi --locked
