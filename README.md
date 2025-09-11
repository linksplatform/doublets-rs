# Doublets

[![Build Status][actions-badge]][actions-url]

[actions-badge]: https://github.com/linksplatform/doublets-rs/workflows/CI/badge.svg
[actions-url]: https://github.com/linksplatform/doublets-rs/actions?query=workflow%3ACI+branch%3Amain

## ⚠️ Current Build Status

**This crate is currently not buildable** due to incompatible platform dependencies that use unstable and obsolete Rust features. The registry versions of `platform-data`, `platform-mem`, and `platform-treesmethods` contain code that is incompatible with any current Rust toolchain.

### Issues:
- Platform dependencies use removed unstable features (`~const`, `default_free_fn`, etc.)
- Missing or incorrect platform dependency implementations 
- Version conflicts between thiserror and platform packages

### Resolution Status:
This is a known issue being tracked in [#18](https://github.com/linksplatform/doublets-rs/issues/18). Until the platform dependencies are updated or replaced, this crate cannot be compiled.

## For Developers

If you need to use doublets functionality, consider:
1. Using the C# version from [linksplatform/Data.Doublets](https://github.com/linksplatform/Data.Doublets)
2. Waiting for platform dependency updates
3. Contributing to fix the platform dependencies

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
