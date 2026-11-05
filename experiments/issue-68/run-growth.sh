#!/usr/bin/env bash
# Linux reproduction: four finite probes, each with a 512 MiB virtual-memory limit.
set -euo pipefail
cd "$(dirname "$0")/../.."
export CARGO_BUILD_JOBS=1
export CARGO_PROFILE_DEV_DEBUG=0
export CARGO_PROFILE_TEST_DEBUG=0
export RUSTFLAGS=-Dwarnings
mkdir -p target/issue-68-logs
cargo build -p doublets --release --example memory_growth --locked \
    > target/issue-68-logs/growth-build.log 2>&1
for backend in unit split unit-file split-file; do
    echo "Checking $backend"
    (
        ulimit -v 524288
        RUST_BACKTRACE=full target/release/examples/memory_growth "$backend"
    ) > "target/issue-68-logs/growth-$backend.log" 2>&1
done
