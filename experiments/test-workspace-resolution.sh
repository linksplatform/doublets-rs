#!/bin/bash
# Test that workspace resolution works correctly for this repository
set -e

cd "$(dirname "$0")/.."

echo "=== Testing workspace resolution ==="
echo ""

# Test rust-paths.rs
echo "--- rust-paths.rs ---"
rust-script scripts/rust-paths.rs 2>&1 || echo "FAILED: rust-paths.rs"
echo ""

# Test get-version.rs (should find version from doublets/Cargo.toml)
echo "--- get-version.rs ---"
rust-script scripts/get-version.rs 2>&1 || echo "FAILED: get-version.rs"
echo ""

# Test check-release-needed.rs (should find name from doublets/Cargo.toml)
echo "--- check-release-needed.rs ---"
HAS_FRAGMENTS=false rust-script scripts/check-release-needed.rs 2>&1 || echo "FAILED: check-release-needed.rs (exit code $?)"
echo ""

# Test bump-version.rs --dry-run
echo "--- bump-version.rs --dry-run ---"
rust-script scripts/bump-version.rs --bump-type patch --dry-run 2>&1 || echo "FAILED: bump-version.rs"
echo ""

echo "=== All tests completed ==="
