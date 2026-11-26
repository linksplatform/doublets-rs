#!/usr/bin/env bash
# Run the issue #94 regressions: both split modes against a plain map, and the
# stored list sizes checked from inside the crate.
set -euo pipefail
cd "$(dirname "$0")/../.."
export CARGO_BUILD_JOBS=1
cargo test -p doublets --test split_sources_list --test split_forward_reference --locked
cargo test -p doublets --lib --locked sources_list
cp experiments/issue-94/probe.rs doublets/examples/issue_94_probe.rs
trap 'rm -f doublets/examples/issue_94_probe.rs' EXIT
cargo run -p doublets --example issue_94_probe --locked
