# Case Study: Issue #47 — Port to Stable Rust and Modernize CI/CD

## Overview

**Issue**: [#47](https://github.com/linksplatform/doublets-rs/issues/47)
**PR**: [#48](https://github.com/linksplatform/doublets-rs/pull/48)
**Date**: April 2026
**Status**: Completed

## Problem Statement

The doublets-rs crate was pinned to `nightly-2022-08-22` (nearly 4 years old) due to dependencies on 12 unstable Rust features. This created several problems:

1. **Security risk**: Old nightly toolchain couldn't receive security patches
2. **Ecosystem isolation**: Couldn't use modern crates that require recent Rust editions
3. **CI fragility**: Nightly features could break at any time
4. **Contributor friction**: Required specific nightly version to compile
5. **Dependency lock-in**: Used git submodules for `platform-mem` and `platform-trees` instead of crates.io

## Requirements Analysis

### From Issue Description
1. Use latest stable Rust version
2. Depend on crates.io packages, not source code submodules
3. Support best practices CI/CD from reference repositories:
   - [mem-rs](https://github.com/linksplatform/mem-rs)
   - [trees-rs](https://github.com/linksplatform/trees-rs)
   - [Numbers](https://github.com/linksplatform/Numbers)
   - [rust-ai-driven-development-pipeline-template](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template)
4. Ensure documentation supports automated generation (rustdoc)
5. Increase test coverage
6. Create case study documentation

## Technical Challenges

### 1. Nightly Feature Removal (12 features)

| Feature | Usage | Stable Replacement |
|---------|-------|--------------------|
| `try_trait_v2` | `Flow` with `?` operator | Explicit `is_break()` checks |
| `fn_traits` | `Fuse` callable as `FnMut` | `.call()` method |
| `unboxed_closures` | Custom `FnOnce`/`FnMut` impls | `.call()` method |
| `type_alias_impl_trait` | Associated type with impl Trait | `-> impl Trait` (stable 1.75) |
| `default_free_fn` | `default()` free function | `Default::default()` / `PhantomData` |
| `box_syntax` | `box expr` syntax | `Box::new(expr)` |
| `allocator_api` | Custom allocators (via bumpalo) | Removed bumpalo (unused) |
| `associated_type_defaults` | Default associated types | Explicit type parameters |
| `generic_associated_types` | GATs | Stable since Rust 1.65 |
| `min_specialization` | Specialization | Removed (not needed) |
| `negative_impls` | Negative trait impls | Removed (not needed) |
| `auto_traits` | Auto trait impls | Removed (not needed) |

### 2. Incompatible Trait Definitions

The `data` crate defines `LinkType` using `funty::Unsigned`, while `trees` defines its own `LinkType` using `num_traits::Unsigned`. These are different traits with the same name. Solution: Added dual bounds `T: data::LinkType + trees::LinkType` (via `crate::TreesLinkType` alias) everywhere tree operations are used.

### 3. RawMem API Migration

`platform-mem` 0.3.0 completely redesigned the `RawMem` trait:

| Old API (submodule) | New API (0.3.0) |
|---------------------|-----------------|
| `alloc(capacity) -> Result<&mut [T]>` | `grow_filled(n, val) -> Result<&mut [T]>` |
| `allocated() -> usize` | `allocated() -> &[Self::Item]` |
| N/A | `shrink(n) -> Result<()>` |
| Associated type `RawMem<T>` | Associated type `RawMem<Item = T>` |

Solution: Created `resize_mem()` helper function that bridges the API gap.

### 4. Range Iteration Without Step Trait

`for index in T::funty(1)..=allocated` requires the `Step` trait which is nightly-only. Solution: Replaced with while loops that manually increment using `index = index + T::funty(1)`.

### 5. Borrow Checker Strictures

Several methods collected iterator results from `&self` and then called `&mut self` methods in the same loop. Solution: Collected iterator results into `Vec` first, then iterated the Vec while mutating self.

## Solution Architecture

### Approach: Incremental Migration

1. **Phase 1**: Remove nightly features, replace with stable equivalents
2. **Phase 2**: Migrate dependencies to crates.io
3. **Phase 3**: Fix compilation errors from API changes
4. **Phase 4**: Fix clippy warnings and format code
5. **Phase 5**: Update tests for new API
6. **Phase 6**: Replace CI/CD pipeline

### Key Decisions

- **Fuse API change**: Changed from callable (`fuse(before, after)`) to method (`fuse.call(before, after)`) because implementing `FnMut` requires nightly features. This is a breaking change but keeps the same semantics.
- **Handler trait removal**: The `Handler` trait existed only to support the `Try` trait integration. With explicit `Flow` returns, handlers are just `FnMut(Link<T>, Link<T>) -> Flow`.
- **Dual LinkType bounds**: Rather than forking the data or trees crate, we added `+ crate::TreesLinkType` bounds throughout the mem module. This is verbose but non-invasive.

## Useful Libraries and Tools

| Library/Tool | Purpose | Version |
|-------------|---------|---------|
| `platform-mem` | Memory management backends | 0.3.0 (crates.io) |
| `platform-trees` | Size-balanced tree implementations | 0.3.3 (crates.io) |
| `platform-data` | Core data types (Flow, LinkType, etc.) | 0.1.0-beta.3 (local) |
| `rust-script` | CI/CD automation scripts | Latest |
| `cargo-llvm-cov` | Code coverage generation | Latest |

## Results

- **12 nightly features** removed → **0 nightly features**
- **Toolchain**: `nightly-2022-08-22` → `stable` (Rust 1.85+)
- **Dependencies**: 2 git submodules → 2 crates.io packages
- **Tests**: 226+ tests passing on stable
- **CI/CD**: Modernized with release.yml pattern, multi-platform testing, automated releases
- **Clippy**: Clean with `-D warnings`

## Lessons Learned

1. **Commit early**: With long-running migrations, committing incrementally preserves work even if the session is interrupted.
2. **Dual trait bounds**: When two crates define similar but incompatible traits, adding dual bounds everywhere is verbose but correct.
3. **API migration helpers**: When an API changes fundamentally (like RawMem), a thin adapter function (`resize_mem`) avoids touching every call site.
4. **Test-driven verification**: Running tests after each change catches issues early, especially when multiple subsystems change simultaneously.
