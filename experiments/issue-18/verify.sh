#!/usr/bin/env bash
# Verify the issue 18 workflow on stable Rust: `cargo add doublets`, resolve
# the newest compatible dependencies, then build and run a small consumer.
#
# Usage: verify.sh [package|crates-io]
#   package    (default) consume `cargo package` output of this checkout, the
#              exact files a crates.io release of this revision would contain.
#   crates-io  consume the latest release published on crates.io.
set -euo pipefail
cd "$(dirname "$0")/../.."
mode=${1:-package}
case "$mode" in
    package | crates-io) ;;
    *) echo "Unknown mode '$mode'; expected 'package' or 'crates-io'" >&2; exit 2 ;;
esac
export RUSTUP_TOOLCHAIN=stable
unset RUSTC_BOOTSTRAP
export CARGO_BUILD_JOBS=1 CARGO_PROFILE_DEV_DEBUG=0
repository_root=$(pwd -W 2>/dev/null || pwd)
export CARGO_TARGET_DIR="$repository_root/target"
probe=target/issue-18-$mode-consumer
log_dir=target/issue-18-logs/$mode
rm -rf "$probe" "$log_dir"
mkdir -p "$probe/src" "$log_dir"

run() {
    local log_file=$1
    shift
    if ! "$@" > "$log_file" 2>&1; then
        cat "$log_file"
        echo "Failed: $* (log: $log_file)" >&2
        exit 1
    fi
}

# A standalone package, as created by `cargo new` in the issue (the explicit
# empty workspace keeps it out of this repository's workspace).
cat > "$probe/Cargo.toml" <<MANIFEST
[package]
name = "issue-18-consumer"
version = "0.0.0"
edition = "2021"
publish = false

[workspace]
MANIFEST
cp experiments/issue-18/consumer.rs "$probe/src/main.rs"
rustc --version
cargo --version

if [[ $mode == package ]]; then
    version=$(sed -n 's/^version = "\(.*\)"$/\1/p' doublets/Cargo.toml | head -n 1)
    run "$log_dir/package.log" cargo package -p doublets --no-verify --allow-dirty
    unpacked=target/issue-18-package
    rm -rf "$unpacked"
    mkdir -p "$unpacked"
    tar -xzf "target/package/doublets-$version.crate" -C "$unpacked"
    run "$log_dir/add.log" cargo add --manifest-path "$probe/Cargo.toml" \
        doublets --path "$unpacked/doublets-$version"
else
    run "$log_dir/add.log" cargo add --manifest-path "$probe/Cargo.toml" doublets
fi
cat "$log_dir/add.log"
printf '\n[features]\nfull = ["doublets/full"]\n' >> "$probe/Cargo.toml"

# Like the issue, resolve without a lockfile so every dependency (including
# `thiserror`) uses its newest semver-compatible release.
run "$log_dir/resolve.log" cargo generate-lockfile --manifest-path "$probe/Cargo.toml"
run "$log_dir/tree.log" cargo tree --manifest-path "$probe/Cargo.toml" --locked \
    --features full --prefix none --no-dedupe
resolved=$(grep -m 1 '^doublets v' "$log_dir/tree.log")
echo "Resolved $resolved"
for historical in 'doublets v0\.1\.' 'platform-data v0\.1\.' 'platform-mem v0\.1\.' \
    'platform-num v0\.1\.' 'platform-treesmethods v'; do
    if grep -q "^$historical" "$log_dir/tree.log"; then
        echo "Historical nightly-only dependency resolved: $historical (see $log_dir/tree.log)" >&2
        exit 1
    fi
done

for features in default full; do
    args=(--locked)
    if [[ $features == full ]]; then args+=(--features full); fi
    log_file="$log_dir/$features.log"
    run "$log_file" cargo run --manifest-path "$probe/Cargo.toml" "${args[@]}"
    cat "$log_file"
done
echo "Issue 18 $mode consumer passed on $(rustc --version)"
