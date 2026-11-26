#!/usr/bin/env bash
# Reproduce issue #94 on a commit before the fix (default: 34e3a23, v0.22.0):
# 1. the default split store orders sources by target, not insertion;
# 2. turning on `USE_LIST` panics with `todo!()` on the first update.
# Logs go to target/issue-94-logs. Usage: reproduce-before.sh [commit]
set -uo pipefail
cd "$(dirname "$0")/../.."
root=$(pwd)
base=${1:-34e3a23}
logs="$root/target/issue-94-logs"
tree="$root/target/issue-94-before"
mkdir -p "$logs"
export CARGO_TARGET_DIR="$root/target/issue-94-before-target" CARGO_BUILD_JOBS=1
git worktree remove --force "$tree" 2>/dev/null
git worktree add --detach "$tree" "$base" >/dev/null
trap 'git worktree remove --force "$tree"' EXIT
cp experiments/issue-94/probe.rs "$tree/doublets/examples/issue_94_probe.rs"

echo "== default store ($base)"
(cd "$tree" && cargo run -p doublets --example issue_94_probe) > "$logs/default.log" 2>&1
grep -E 'left:|right:|panicked|sources of' "$logs/default.log"

echo "== USE_LIST = true ($base)"
sed -i 's/const USE_LIST: bool = false;/const USE_LIST: bool = true;/' \
    "$tree/doublets/src/mem/split/store.rs"
(cd "$tree" && cargo run -p doublets --example issue_94_probe) > "$logs/use-list.log" 2>&1
grep -E "panicked|not yet implemented|sources of" "$logs/use-list.log"
