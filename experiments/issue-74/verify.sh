#!/usr/bin/env bash
# Run checks sequentially within the issue solver's memory budget.
set -euo pipefail
cd "$(dirname "$0")/../.."
log_dir="${1:-target/issue-74-logs}"
mkdir -p "$log_dir"
export CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0
export RUSTFLAGS=-Dwarnings RUST_TEST_THREADS=1

cargo fmt --all -- --check > "$log_dir/fmt.log" 2>&1
cargo clippy --workspace --all-targets --all-features -- -D warnings > "$log_dir/clippy.log" 2>&1
cargo test --workspace --all-features --verbose > "$log_dir/tests.log" 2>&1
cargo test --workspace --doc --verbose > "$log_dir/doc-tests.log" 2>&1
cargo build --workspace --release --verbose > "$log_dir/release.log" 2>&1
cargo doc --workspace --no-deps --all-features > "$log_dir/docs.log" 2>&1
cargo package -p doublets --list --allow-dirty > "$log_dir/package.log" 2>&1
cargo run -p doublets --example forward_references > "$log_dir/example.log" 2>&1
for check in check-file-size check-readme-badges check-toolchain-docs get-bump-type; do
    rust-script "scripts/$check.rs" > "$log_dir/$check.log" 2>&1
done
GITHUB_BASE_REF=main rust-script scripts/check-changelog-fragment.rs > "$log_dir/changelog-check.log" 2>&1
GITHUB_EVENT_NAME=pull_request GITHUB_HEAD_REF=issue-74-b2cc25ca247c GITHUB_BASE_REF=main \
    rust-script scripts/check-version-modification.rs > "$log_dir/version-check.log" 2>&1
