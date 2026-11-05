#!/usr/bin/env bash
# Run the repository CI checks and retain one log per check.
set -euo pipefail
cd "$(dirname "$0")/../.."
log_dir="${1:-/tmp/doublets-issue-61-checks}"
mkdir -p "$log_dir"
cargo fmt --all -- --check > "$log_dir/fmt.log" 2>&1
cargo clippy --workspace --all-targets --all-features -- -D warnings > "$log_dir/clippy.log" 2>&1
cargo test --workspace --all-features --verbose > "$log_dir/tests.log" 2>&1
cargo test --workspace --doc --verbose > "$log_dir/doc-tests.log" 2>&1
cargo build --workspace --release --verbose > "$log_dir/release.log" 2>&1
cargo doc --workspace --no-deps --all-features > "$log_dir/docs.log" 2>&1
cargo package -p doublets --list --allow-dirty > "$log_dir/package.log" 2>&1
cargo run -p doublets --example transactions > "$log_dir/example.log" 2>&1
for check in check-file-size check-readme-badges check-toolchain-docs get-bump-type; do
    rust-script "scripts/$check.rs" > "$log_dir/$check.log" 2>&1
done
