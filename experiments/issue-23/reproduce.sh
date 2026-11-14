#!/usr/bin/env bash
# Verify that the exact dependency versions reported in issue 23 need nightly.
set -euo pipefail
cd "$(dirname "$0")/../.."
export RUSTUP_TOOLCHAIN=stable
unset RUSTC_BOOTSTRAP
export CARGO_BUILD_JOBS=1 CARGO_PROFILE_DEV_DEBUG=0
export CARGO_TARGET_DIR="$(pwd)/target/issue-23-historical/build"
log_dir=target/issue-23-logs
mkdir -p "$log_dir"

for dependency in num data; do
    case "$dependency" in
        num) version=0.1.0-aplha.1 ;;
        data) version=0.1.0-beta.3 ;;
    esac
    probe="target/issue-23-historical/$dependency"
    mkdir -p "$probe/src"
    cat > "$probe/Cargo.toml" <<EOF
[package]
name = "issue-23-historical-$dependency"
version = "0.0.0"
edition = "2021"
publish = false

[workspace]

[dependencies]
platform-$dependency = "=$version"
EOF
    echo 'fn main() {}' > "$probe/src/main.rs"
    log_file="$log_dir/historical-$dependency.log"
    if cargo check --manifest-path "$probe/Cargo.toml" > "$log_file" 2>&1; then
        echo "Unexpected success for platform-$dependency $version" >&2
        exit 1
    fi
    if ! grep -q 'error\[E0554\]' "$log_file"; then
        echo "Expected the stable-channel feature-gate error; inspect $log_file" >&2
        exit 1
    fi
    echo "Confirmed E0554 for platform-$dependency $version (log: $log_file)"
done
