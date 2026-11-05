#!/usr/bin/env bash
# Run the issue 66 regressions and the doc-tested guides that describe them.
set -euo pipefail
cd "$(dirname "$0")/../.."
export CARGO_BUILD_JOBS=1 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0
export RUSTFLAGS=-Dwarnings RUSTDOCFLAGS=-Dwarnings
log_dir=target/issue-66-logs
mkdir -p "$log_dir"
run() {
    local log_file="$log_dir/$1.log"
    shift
    echo "Running $* (log: $log_file)"
    if ! "$@" > "$log_file" 2>&1; then
        cat "$log_file"
        exit 1
    fi
}
run regressions cargo test -p doublets --test uniqueness_missing_update --test decorators --locked
run guides cargo test -p doublets --doc --locked -- Guide CascadeUniquenessAndUsagesResolver
echo 'Issue 66 checks passed.'
