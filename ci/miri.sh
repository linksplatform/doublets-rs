#!/bin/sh
set -e

MIRI_NIGHTLY=nightly-$(curl -s https://rust-lang.github.io/rustup-components-history/x86_64-unknown-linux-gnu/miri)
echo "Installing latest nightly with Miri: $MIRI_NIGHTLY"
rustup set profile minimal
rustup toolchain install "$MIRI_NIGHTLY" --component miri

# Use +toolchain syntax to override rust-toolchain.toml which pins to an older nightly
echo "Running Miri setup..."
cargo +"$MIRI_NIGHTLY" miri setup

echo "Running Miri tests..."
MIRIFLAGS="-Zmiri-disable-stacked-borrows" cargo +"$MIRI_NIGHTLY" miri test --all-features --package doublets
