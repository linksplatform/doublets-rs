#!/usr/bin/env bash
# Reproduce the historical issue 18 failure: `doublets 0.1.0-pre` from
# crates.io on the nightly it was written for, with the newest semver-compatible
# `thiserror 1.x`. A second build pins the 2022 dependency versions to confirm
# the root cause. Not run in CI: it installs an obsolete nightly toolchain.
set -euo pipefail
cd "$(dirname "$0")/../.."
nightly=nightly-2022-07-23
work="$(pwd)/target/issue-18-historical"
probe="$work/probe"
log_dir=target/issue-18-logs/historical
unset RUSTC_BOOTSTRAP
export CARGO_BUILD_JOBS=1
mkdir -p "$probe/src" "$log_dir"
rustup toolchain install "$nightly" --profile minimal > "$log_dir/toolchain.log" 2>&1 || {
    cat "$log_dir/toolchain.log"
    exit 1
}
# A private Cargo home keeps old-format caches away from the user's registry.
export CARGO_HOME="$work/cargo-home" CARGO_TARGET_DIR="$work/build"
mkdir -p "$CARGO_HOME"

# `rust-version` makes stable Cargo's MSRV-aware resolver skip releases the
# 2022 Cargo cannot parse (edition 2024) and write a version 3 lockfile. The
# newest `thiserror 1.x` still qualifies, so this matches `cargo add doublets`
# in the issue, which picked 0.1.0-pre before stable releases existed.
cat > "$probe/Cargo.toml" <<'MANIFEST'
[package]
name = "issue-18-historical"
version = "0.0.0"
edition = "2021"
rust-version = "1.64"
publish = false

[workspace]

[dependencies]
doublets = "=0.1.0-pre"
MANIFEST
echo 'fn main() {}' > "$probe/src/main.rs"
rm -f "$probe/Cargo.lock"
CARGO_RESOLVER_INCOMPATIBLE_RUST_VERSIONS=fallback \
    cargo +stable generate-lockfile --manifest-path "$probe/Cargo.toml" \
    > "$log_dir/resolve.log" 2>&1 || { cat "$log_dir/resolve.log"; exit 1; }
grep -A1 -E '^name = "(doublets|platform-data|thiserror)"$' "$probe/Cargo.lock" \
    | grep -v '^--' | paste - - | tee "$log_dir/resolved.txt"

build() {
    cargo "+$nightly" -Z sparse-registry build --locked \
        --manifest-path "$probe/Cargo.toml" > "$1" 2>&1
}

log_file="$log_dir/newest-thiserror.log"
if build "$log_file"; then
    echo "Unexpected success; inspect $log_file" >&2
    exit 1
fi
for expected in \
    'error\[E0432\]: unresolved import `thiserror`' \
    'error\[E0412\]: cannot find type `Request` in module `std::error`' \
    "error\[E0658\]: use of unstable library feature 'error_generic_member_access'" \
    'error\[E0599\]: no method named `thiserror_provide`' \
    'platform-data-0\.1\.0-beta\.3/src/links\.rs'; do
    if ! grep -q "$expected" "$log_file"; then
        echo "Missing expected output '$expected'; inspect $log_file" >&2
        exit 1
    fi
done
echo "Reproduced the four issue 18 errors with $(rustc "+$nightly" --version) (log: $log_file)"

# Control: the dependency versions current when 0.1.0-pre was published.
cargo +stable update --manifest-path "$probe/Cargo.toml" \
    -p thiserror --precise 1.0.31 > "$log_dir/pin.log" 2>&1
cargo +stable update --manifest-path "$probe/Cargo.toml" \
    -p platform-data --precise 0.1.0-beta.1 >> "$log_dir/pin.log" 2>&1
log_file="$log_dir/pinned-2022.log"
if ! build "$log_file"; then
    cat "$log_file"
    echo "The pinned 2022 dependency set failed to build" >&2
    exit 1
fi
echo "Pinned thiserror 1.0.31 and platform-data 0.1.0-beta.1 build on $nightly (log: $log_file)"

# The old release can never build on stable: its crates enable nightly features.
log_file="$log_dir/stable.log"
if cargo +stable check --locked --manifest-path "$probe/Cargo.toml" > "$log_file" 2>&1; then
    echo "Unexpected stable success; inspect $log_file" >&2
    exit 1
fi
if ! grep -q 'error\[E0554\]' "$log_file"; then
    echo "Expected the stable-channel feature-gate error; inspect $log_file" >&2
    exit 1
fi
echo "Confirmed E0554 for doublets 0.1.0-pre with $(rustc +stable --version) (log: $log_file)"
