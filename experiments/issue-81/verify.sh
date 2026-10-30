#!/usr/bin/env bash
# Run release-policy regressions, then the maintained repository verification.
set -euo pipefail
cd "$(dirname "$0")/../.."
export CARGO_BUILD_JOBS=1 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0
export RUSTFLAGS=-Dwarnings RUSTDOCFLAGS=-Dwarnings
export ISSUE22_LOG_DIR=target/issue-81-logs
mkdir -p "$ISSUE22_LOG_DIR"
bash experiments/issue-81/verify-test-layout.sh
bash experiments/issue-81/check-test-layout.sh
for features in default no-default; do
    args=()
    if [[ "$features" == no-default ]]; then args+=(--no-default-features); fi
    log_file="$ISSUE22_LOG_DIR/release-$features.log"
    echo "Testing release with $features features (log: $log_file)"
    if ! cargo test -p doublets --test release "${args[@]}" --locked \
        -- --test-threads=1 > "$log_file" 2>&1; then
        cat "$log_file"
        exit 1
    fi
done
cargo run -p doublets --example release --locked > "$ISSUE22_LOG_DIR/example.log" 2>&1
bash experiments/issue-22/verify.sh
