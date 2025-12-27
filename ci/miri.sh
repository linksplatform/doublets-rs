#!/bin/sh
set -e

# Pin to a specific nightly version that is compatible with the codebase.
# The platform-data dependency uses unstable const features that are not
# compatible with newer nightly compilers (const_deref, const_result_drop,
# const_ops, etc. were removed or changed).
MIRI_NIGHTLY=nightly-2022-08-22
echo "Installing Miri with nightly: $MIRI_NIGHTLY"
rustup set profile minimal
rustup default "$MIRI_NIGHTLY"

rustup component add miri
cargo miri setup

MIRIFLAGS="-Zmiri-disable-stacked-borrows" cargo miri test --all-features --package doublets
