#!/usr/bin/env bash
# Run the repository checks sequentially within the issue solver's memory budget.
set -euo pipefail
cd "$(dirname "$0")/../.."
export CARGO_BUILD_JOBS=1 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0
export CARGO_TARGET_DIR="$(pwd -W 2>/dev/null || pwd)/target"
export RUST_BACKTRACE=1
log_dir=target/issue-86-logs
mkdir -p "$log_dir"

run() {
    local name=$1
    shift
    echo "Running $name (log: $log_dir/$name.log)"
    if ! "$@" > "$log_dir/$name.log" 2>&1; then
        cat "$log_dir/$name.log"
        return 1
    fi
}

run format cargo fmt --all -- --check
run clippy cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
run workspace-tests cargo test --workspace --all-features --locked --verbose
run doctests cargo test --workspace --doc --locked --verbose
run packages bash experiments/issue-22/check-packages.sh
run ffi-tests cargo test --manifest-path doublets-ffi/Cargo.toml --all-features --locked
run ffi-attributes-tests cargo test --manifest-path doublets-ffi/ffi-attributes/Cargo.toml --locked
run env-decorators-tests cargo test --manifest-path doublets-ffi/env-decorators/Cargo.toml --locked
run no-default-core cargo check -p doublets --no-default-features --locked
run no-default-decorators cargo test -p doublets --test decorator_consolidation --no-default-features --locked
run no-default-creation cargo test -p doublets --test creation_flow --no-default-features --locked
run release cargo build --release --locked
run ffi-release cargo build --manifest-path doublets-ffi/Cargo.toml --release --locked
run ffi-abi python3 experiments/issue-22/ffi-smoke.py target/release/libdoublets_ffi.so
run package-list cargo package -p doublets --list --allow-dirty --locked
run docs cargo doc --workspace --no-deps --all-features --locked
run file-size rust-script scripts/check-file-size.rs
run readme-badges rust-script scripts/check-readme-badges.rs
run toolchain-docs rust-script scripts/check-toolchain-docs.rs
run changelog rust-script scripts/check-changelog-fragment.rs
run version env GITHUB_EVENT_NAME=pull_request GITHUB_HEAD_REF=issue-86-1b53ed1f880d \
    GITHUB_BASE_REF=main rust-script scripts/check-version-modification.rs
