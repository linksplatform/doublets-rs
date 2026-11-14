#!/usr/bin/env bash
# Run mutation-contract regressions, then the maintained repository verification.
set -euo pipefail
cd "$(dirname "$0")/../.."
export CARGO_BUILD_JOBS=1 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0
export RUSTFLAGS=-Dwarnings RUSTDOCFLAGS=-Dwarnings
export ISSUE22_LOG_DIR=target/issue-80-logs
mkdir -p "$ISSUE22_LOG_DIR"
bash experiments/issue-81/check-test-layout.sh
run() {
    local log_file="$ISSUE22_LOG_DIR/$1.log"
    shift
    echo "Running $* (log: $log_file)"
    if ! "$@" > "$log_file" 2>&1; then
        cat "$log_file"
        exit 1
    fi
}
for features in default no-default; do
    args=()
    if [[ "$features" == no-default ]]; then args+=(--no-default-features); fi
    run "unchecked-mutation-$features" cargo test -p doublets --test unchecked_mutation \
        "${args[@]}" --locked -- --test-threads=1
done
run internal cargo test -p doublets --lib --locked unchecked_mutation_omits_validation_work
run guides cargo test -p doublets --doc --locked Guide
run example cargo run -p doublets --example unchecked_mutation --locked
bash experiments/issue-22/verify.sh
