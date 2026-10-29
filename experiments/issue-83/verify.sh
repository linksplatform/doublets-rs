#!/usr/bin/env bash
# Verify decorator consolidation and the repository's complete stable checks.
set -euo pipefail
cd "$(dirname "$0")/../.."
export CARGO_BUILD_JOBS=1 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0
export RUSTFLAGS=-Dwarnings RUSTDOCFLAGS=-Dwarnings
export ISSUE22_LOG_DIR=target/issue-83-logs
mkdir -p "$ISSUE22_LOG_DIR"

for features in default no-default; do
    args=()
    if [[ "$features" == no-default ]]; then
        args+=(--no-default-features)
    fi
    log_file="$ISSUE22_LOG_DIR/parity-$features.log"
    echo "Testing decorators with $features features (log: $log_file)"
    if ! cargo test -p doublets --test decorator_consolidation --test split_null_endpoint \
        "${args[@]}" --locked -- --test-threads=1 > "$log_file" 2>&1; then
        cat "$log_file"
        exit 1
    fi
done

# Includes all-feature parity tests, the C ABI, and embedded release-script tests.
bash experiments/issue-22/verify.sh
