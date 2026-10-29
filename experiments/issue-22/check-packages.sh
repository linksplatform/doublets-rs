#!/usr/bin/env bash
# Compile every maintained manifest on stable, including the standalone FFI packages.
set -euo pipefail
cd "$(dirname "$0")/../.."
export RUSTUP_TOOLCHAIN=stable
unset RUSTC_BOOTSTRAP
export CARGO_BUILD_JOBS=1 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0
export CARGO_TARGET_DIR="$(pwd -W 2>/dev/null || pwd)/target"
log_dir=${ISSUE22_LOG_DIR:-target/issue-22-logs}
mkdir -p "$log_dir"
status=0
while IFS= read -r manifest; do
    if ! rg -q '^\[package\]' "$manifest"; then
        continue
    fi
    package=$(dirname "$manifest")
    name=${package//\//-}
    echo "Checking $package on stable (log: $log_dir/$name.log)"
    if ! cargo check --manifest-path "$manifest" --all-targets --all-features --locked > "$log_dir/$name.log" 2>&1; then
        echo "FAILED: $package"
        cat "$log_dir/$name.log"
        status=1
    fi
done < <(rg --files --path-separator / -g Cargo.toml -g '!target/**' -g '!**/target/**' | sort)
exit "$status"
