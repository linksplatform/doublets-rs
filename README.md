# Doublets

[![Build Status][actions-badge]][actions-url]

[actions-badge]: https://github.com/linksplatform/doublets-rs/workflows/CI/badge.svg
[actions-url]: https://github.com/linksplatform/doublets-rs/actions?query=workflow%3ACI+branch%3Amain

**Note: This crate requires Rust nightly (`nightly-2022-08-22` or later) due to unstable features used by platform dependencies.**

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
doublets = "0.1.0-pre"
```

Make sure you have the required Rust nightly toolchain installed:

```bash
rustup install nightly-2022-08-22
rustup override set nightly-2022-08-22
```

Alternatively, the project includes a `rust-toolchain.toml` file that will automatically use the correct toolchain.

## [Overview](https://github.com/linksplatform)

later

## Example

A basic operations in doublets:

```rust
use doublets::{data, mem, unit, Doublets, DoubletsExt, Links};

fn main() -> Result<(), doublets::Error<usize>> {
    // use file as memory for doublets
    let mem = mem::FileMapped::from_path("db.links")?;
    let mut store = unit::Store::<usize, _>::new(mem)?;

    // create 1: 1 1 - it's point: link where source and target it self
    let point = store.create_link(1, 1)?;

    // `any` constant denotes any link
    let any = store.constants().any;

    // print all store from store where (index: any, source: any, target: any)
    store.each_iter([any, any, any]).for_each(|link| {
        println!("{link:?}");
    });

    // delete point with handler (Link, Link)
    store
        .delete_with(point, |before, after| {
            println!("delete: {before:?} => {after:?}");
            // track issue: https://github.com/linksplatform/doublets-rs/issues/4
            data::Flow::Continue
        })
        .map(|_| ())
}
```
