#!/usr/bin/env bash
# Exercise an external consumer of a committed Git revision on stable Rust.
set -euo pipefail
cd "$(dirname "$0")/../.."
export RUSTUP_TOOLCHAIN=stable
unset RUSTC_BOOTSTRAP
export CARGO_BUILD_JOBS=1 CARGO_PROFILE_DEV_DEBUG=0
repository_root=$(pwd -W 2>/dev/null || pwd)
export CARGO_TARGET_DIR="$repository_root/target"
git_url=${DOUBLETS_GIT_URL:-"file:///${repository_root#/}"}
git_revision=${DOUBLETS_GIT_REV:-$(git rev-parse HEAD)}
probe=target/issue-19-consumer
log_dir=target/issue-19-logs
mkdir -p "$probe/src" "$log_dir"
cat > "$probe/Cargo.toml" <<EOF
[package]
name = "issue-19-consumer"
version = "0.0.0"
edition = "2021"
publish = false

[workspace]

[features]
full = ["doublets/full"]

[dependencies]
doublets = { git = "$git_url", rev = "$git_revision" }
EOF
cp experiments/issue-19/consumer.rs "$probe/src/main.rs"
rustc --version
echo "Checking $git_url at $git_revision (bootstrap unset)"
cargo generate-lockfile --manifest-path "$probe/Cargo.toml" > "$log_dir/resolve.log" 2>&1 || {
    cat "$log_dir/resolve.log"
    exit 1
}
for features in default full; do
    args=(--locked)
    if [[ $features == full ]]; then args+=(--features full); fi
    log_file="$log_dir/$features.log"
    if ! cargo run --manifest-path "$probe/Cargo.toml" "${args[@]}" > "$log_file" 2>&1; then
        cat "$log_file"
        exit 1
    fi
    cat "$log_file"
done
