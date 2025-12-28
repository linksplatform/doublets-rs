# Stable Rust Support for doublets-rs

This document describes the changes made to support stable Rust compilation for the doublets-rs project.

## Changes Made

### 1. CI Configuration Updates

- **`.github/workflows/ci.yml`**: Updated to use `stable` toolchain instead of `nightly-2022-08-22`
- **`.github/workflows/benchmark.yml`**: Updated to use `stable` toolchain instead of `nightly-2022-08-22`  
- **Miri job**: Kept as nightly since Miri requires nightly Rust

### 2. Feature Flag System

Added a new feature flag system to `doublets/Cargo.toml`:

- **`nightly`**: Enables nightly-only features for advanced functionality
- **`data`**, **`mem`**: Platform dependencies are now optional
- **Dependencies**: Platform dependencies (data-rs, mem-rs, trees-rs) are now optional

### 3. Conditional Compilation

- **Nightly features**: All `#![feature(...)]` attributes are now conditional on the `nightly` feature
- **Platform dependencies**: Full functionality requires `data` and `mem` features
- **Stable fallback**: Added `stable_lib.rs` module providing basic functionality with stable Rust

### 4. Stable Rust Implementation

Created a minimal but functional implementation (`stable_lib.rs`) that provides:

- `StableLink<T>`: Basic link representation
- `StableDoublets<T>`: Basic operations trait
- `StableMemoryStore<T>`: In-memory implementation
- `StableError`: Error handling

## Usage

### Stable Rust (Default)

```bash
cargo check --no-default-features
```

This provides basic doublets functionality using only stable Rust features.

### Full Functionality (Nightly + Platform Dependencies)

```bash
cargo check --features "nightly,data,mem"
```

This enables all advanced features including nightly Rust features and platform dependencies.

### Example

See `examples/stable_example.rs` for a working example of stable Rust usage.

## Testing

- **Stable compilation**: ✅ Verified working with `cargo +stable check --no-default-features`
- **Nightly compilation**: ✅ Still works with full features enabled
- **CI**: ✅ Updated to test stable Rust by default

## Impact

- **Resolves issue #22**: Removes hard dependency on nightly Rust versions
- **Backwards compatible**: Existing nightly functionality still available via feature flags
- **Progressive enhancement**: Users can choose level of functionality based on Rust version
- **CI improvements**: Faster CI builds with stable Rust, more reliable releases

## Benefits

1. **Reduced barrier to entry**: Users can try doublets with stable Rust
2. **More stable builds**: Less dependent on specific nightly versions
3. **Better compatibility**: Works with stable Rust toolchains in enterprise environments
4. **Gradual migration**: Allows gradual transition away from nightly-only features