#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
export CARGO_BUILD_JOBS=1
export CARGO_PROFILE_DEV_DEBUG=0
export CARGO_PROFILE_TEST_DEBUG=0
export RUSTFLAGS=-Dwarnings
log_dir=target/issue-68-logs
mkdir -p "$log_dir"
run() {
    local name=$1
    shift
    echo "Running $name (log: $log_dir/$name.log)"
    "$@" > "$log_dir/$name.log" 2>&1
}
run dependency-audit python3 experiments/issue-68/audit_dependencies.py --check
run format cargo fmt --all -- --check
run clippy cargo clippy --workspace --all-targets --all-features --locked
run tests cargo test --workspace --all-features --locked --verbose -- --test-threads=1
run doctests cargo test --workspace --doc --locked --verbose
run release cargo build --workspace --release --locked --verbose
run documentation cargo doc --workspace --no-deps --all-features --locked
run package cargo package -p doublets --list --allow-dirty --locked
for package in ffi-attributes env-decorators; do
    run "$package-tests" cargo test --manifest-path "doublets-ffi/$package/Cargo.toml" --locked
    run "$package-format" cargo fmt --manifest-path "doublets-ffi/$package/Cargo.toml" -- --check
done
if command -v rust-script > /dev/null; then
    for check in check-file-size check-readme-badges check-toolchain-docs; do
        run "$check" rust-script "scripts/$check.rs"
    done
    # Compile all embedded manifests without invoking release/publish side effects.
    for script in scripts/*.rs; do
        if rg -q '^//! \[dependencies\]' "$script"; then
            run "$(basename "$script" .rs)-compile" rust-script --test "$script"
        fi
    done
else
    echo 'rust-script is required for repository and embedded dependency checks' >&2
    exit 1
fi
