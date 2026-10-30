#!/usr/bin/env bash
# Reproduce the full stable migration checks within a small build container.
set -euo pipefail
cd "$(dirname "$0")/../.."
export RUSTUP_TOOLCHAIN=stable
unset RUSTC_BOOTSTRAP
export CARGO_BUILD_JOBS=1 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0
export CARGO_TARGET_DIR="$(pwd -W 2>/dev/null || pwd)/target"
export RUSTFLAGS=-Dwarnings
log_dir=${ISSUE22_LOG_DIR:-target/issue-22-logs}
mkdir -p "$log_dir"
run() {
    local name=$1
    shift
    echo "Running $name (log: $log_dir/$name.log)"
    if ! "$@" > "$log_dir/$name.log" 2>&1; then
        cat "$log_dir/$name.log"
        return 1
    fi
}
run dependency-audit python3 experiments/issue-68/audit_dependencies.py --check
run format cargo fmt --all -- --check
run clippy cargo clippy --workspace --all-targets --all-features --locked
run tests cargo test --workspace --all-features --locked --verbose -- --test-threads=1
run doctests cargo test --workspace --doc --locked --verbose
run no-default-features cargo check -p doublets --no-default-features --locked
run release cargo build --workspace --release --locked --verbose
run documentation cargo doc --workspace --no-deps --all-features --locked
run package cargo package -p doublets --list --allow-dirty --locked
for package in doublets-ffi doublets-ffi/env-decorators doublets-ffi/ffi-attributes; do
    name=${package//\//-}
    run "$name-tests" cargo test --manifest-path "$package/Cargo.toml" --all-features --locked -- --test-threads=1
    run "$name-format" cargo fmt --manifest-path "$package/Cargo.toml" -- --check
    run "$name-clippy" cargo clippy --manifest-path "$package/Cargo.toml" --all-targets --all-features --locked
done
run ffi-release cargo build --manifest-path doublets-ffi/Cargo.toml --release --locked
case "$(uname -s)" in
    Darwin) ffi_library=target/release/libdoublets_ffi.dylib ;;
    MINGW*|MSYS*|CYGWIN*) ffi_library=target/release/doublets_ffi.dll ;;
    *) ffi_library=target/release/libdoublets_ffi.so ;;
esac
run ffi-abi python3 experiments/issue-22/ffi-smoke.py "$ffi_library"
run stable-packages bash experiments/issue-22/check-packages.sh
for check in check-file-size check-readme-badges check-toolchain-docs; do
    run "$check" rust-script "scripts/$check.rs"
done
# Compile embedded dependencies without running release or publishing actions.
for script in scripts/*.rs; do
    if grep -q '^//! \[dependencies\]' "$script"; then
        run "$(basename "$script" .rs)-compile" rust-script --test "$script"
    fi
done
run changelog env GITHUB_BASE_REF=main rust-script scripts/check-changelog-fragment.rs
run version env GITHUB_EVENT_NAME=pull_request GITHUB_HEAD_REF="$(git branch --show-current)" GITHUB_BASE_REF=main rust-script scripts/check-version-modification.rs
run diff git diff --check origin/main
