# Doublets

[![CI](https://github.com/linksplatform/doublets-rs/workflows/CI/badge.svg)](https://github.com/linksplatform/doublets-rs/actions?query=workflow%3ACI+branch%3Amain)
[![Benchmark](https://github.com/linksplatform/doublets-rs/workflows/Benchmark/badge.svg)](https://github.com/linksplatform/doublets-rs/actions?query=workflow%3ABenchmark+branch%3Amain)
[![Crates.io](https://img.shields.io/crates/v/doublets.svg)](https://crates.io/crates/doublets)
[![License](https://img.shields.io/badge/license-Unlicense-blue.svg)](https://github.com/linksplatform/doublets-rs/blob/main/LICENSE)

LinksPlatform's Rust implementation of Doublets (associative storage links).

Rust port of [Data.Doublets](https://github.com/linksplatform/Data.Doublets) library.

## Overview

Doublets is an associative data structure that represents a link store where each link consists of:
- **Index** - unique identifier of the link
- **Source** - the link's source (can reference itself or another link)
- **Target** - the link's target (can reference itself or another link)

This simple yet powerful structure can represent any data model, from traditional key-value pairs to complex graph structures. Doublets provide a unified approach to data storage with constant-time lookup operations using size-balanced trees.

### Key Features

- **File-mapped storage** - data persists directly to disk using memory-mapped files
- **Generic link types** - supports any unsigned integer type (`u8`, `u16`, `u32`, `u64`, `usize`)
- **Two storage modes**:
  - `unit::Store` - combined data and index storage in a single memory region
  - `split::Store` - separated data and index storage for optimized memory layouts
- **Thread-safe** - `Send` and `Sync` implementations for concurrent access
- **Query system** - flexible pattern matching using `any` constants
- **FFI bindings** - C-compatible foreign function interface for cross-language integration

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
doublets = "0.1.0-pre"
```

**Note:** This crate requires nightly Rust due to usage of experimental features.

```bash
rustup default nightly
```

## Example

Basic CRUD operations with doublets:

```rust
use doublets::{mem, unit, Doublets, DoubletsExt, Link, Links};
use data::Flow;

fn main() -> Result<(), doublets::Error<usize>> {
    // Use file as persistent storage for doublets
    let mem = mem::FileMapped::from_path("db.links")?;
    let mut store = unit::Store::<usize, _>::new(mem)?;

    // Create a point link (a link that references itself as both source and target)
    // Point: 1 -> (1, 1)
    let point = store.create_point()?;
    println!("Created point: {}", point);

    // Create a regular link with explicit source and target
    // Link: 2 -> (1, 1)
    let link = store.create_link(point, point)?;
    println!("Created link: {}", link);

    // The `any` constant matches any link in queries
    let any = store.constants().any;

    // Count all links in the store
    println!("Total links: {}", store.count());

    // Iterate over all links using pattern [any, any, any]
    println!("All links:");
    store.each_iter([any, any, any]).for_each(|link| {
        println!("  {}: {} -> {}", link.index, link.source, link.target);
    });

    // Query links by source: find all links where source = point
    println!("Links with source = {}:", point);
    store.each_iter([any, point, any]).for_each(|link| {
        println!("  {}: {} -> {}", link.index, link.source, link.target);
    });

    // Update a link: change the target
    let updated = store.update(link, point, link)?;
    println!("Updated link {} to target {}", link, updated);

    // Delete a link with a callback handler
    store.delete_with(link, |before, after| {
        println!("Deleted: {:?} => {:?}", before, after);
        Flow::Continue
    })?;

    // Clean up: delete all remaining links
    store.delete_all()?;

    Ok(())
}
```

### Using In-Memory Storage

For testing or temporary data, you can use heap-allocated memory instead of file storage:

```rust
use doublets::{mem, unit, Doublets, Links};

fn main() -> Result<(), doublets::Error<usize>> {
    // Use global allocator for in-memory storage
    let mem = mem::Global::new();
    let mut store = unit::Store::<usize, _>::new(mem)?;

    // Create some links
    for i in 0..100 {
        store.create_point()?;
    }

    println!("Created {} links", store.count());
    Ok(())
}
```

### Using Split Storage

Split storage separates data and index trees for potentially better cache utilization:

```rust
use doublets::{split::Store, Doublets, Links};
use mem::Global;

fn main() -> Result<(), doublets::Error<usize>> {
    let mut store = Store::<usize, _, _>::new(Global::new(), Global::new())?;

    // Create 1 million point links
    for _ in 0..1_000_000 {
        store.create_point()?;
    }

    println!("Total links: {}", store.count());
    Ok(())
}
```

## API Overview

### Core Traits

| Trait | Description |
|-------|-------------|
| `Links<T>` | Low-level CRUD operations with handlers |
| `Doublets<T>` | High-level operations with ergonomic API |
| `DoubletsExt<T>` | Iterator extensions and parallel processing |

### Main Types

| Type | Description |
|------|-------------|
| `Link<T>` | A triplet of (index, source, target) |
| `Doublet<T>` | A pair of (source, target) without index |
| `unit::Store` | Combined memory layout storage |
| `split::Store` | Separated memory layout storage |
| `Error<T>` | Error type for link operations |

### Key Operations

```rust
// Create operations
store.create()?;                      // Create empty link
store.create_point()?;                // Create self-referencing link
store.create_link(source, target)?;   // Create link with source and target

// Read operations
store.count();                        // Count all links
store.count_by([any, source, any]);   // Count by pattern
store.get_link(index);                // Get link by index
store.search(source, target);         // Find link by source and target
store.iter();                         // Iterate all links
store.each_iter(query);               // Iterate by pattern

// Update operations
store.update(index, source, target)?; // Update link

// Delete operations
store.delete(index)?;                 // Delete link
store.delete_all()?;                  // Delete all links
```

## Architecture

```
doublets-rs/
├── doublets/           # Core library
│   ├── src/
│   │   ├── data/       # Core data structures (Link, Doublet, traits)
│   │   └── mem/        # Memory management and storage
│   │       ├── unit/   # Combined storage implementation
│   │       └── split/  # Split storage implementation
│   └── benches/        # Performance benchmarks
├── doublets-ffi/       # C FFI bindings
├── dev-deps/           # Platform dependencies
│   ├── data-rs/        # Data primitives (LinkType, Flow, etc.)
│   ├── mem-rs/         # Memory abstractions (RawMem, FileMapped)
│   └── trees-rs/       # Tree structures (size-balanced trees)
└── integration/        # Integration tests
```

## Features

| Feature | Description |
|---------|-------------|
| `platform` (default) | Core platform types and traits |
| `mem` | Memory management utilities |
| `num` | Numeric utilities |
| `data` | Re-exports from `platform-data` |
| `rayon` | Parallel iteration support |
| `small-search` | Stack-allocated buffers for small queries |
| `full` | All features enabled |

## Performance

The library is optimized for high-throughput operations:

- **Size-balanced trees** for O(log n) search, insert, and delete operations
- **Memory-mapped files** for efficient disk I/O
- **Optional parallel iteration** with rayon feature
- Benchmarks show creation of 1 million points in ~1 second on modern hardware

Run benchmarks:

```bash
cargo bench --all-features
```

## Related Projects

- [Data.Doublets](https://github.com/linksplatform/Data.Doublets) - C# implementation
- [Comparisons.SQLiteVSDoublets](https://github.com/linksplatform/Comparisons.SQLiteVSDoublets) - Performance comparison with SQLite

## Documentation

- [API Documentation](https://docs.rs/doublets) (when published)
- [LinksPlatform Overview](https://github.com/linksplatform)

## Dependencies

- [platform-data](https://github.com/linksplatform/data-rs) - Core data types
- [platform-mem](https://github.com/linksplatform/mem-rs) - Memory abstractions
- [platform-trees](https://github.com/linksplatform/trees-rs) - Tree implementations

## Support

- Ask questions at [stackoverflow.com/tags/links-platform](https://stackoverflow.com/tags/links-platform) (use tag `links-platform`)
- Join our [Discord server](https://discord.gg/eEXJyjWv5e) for real-time support
- Open issues on [GitHub](https://github.com/linksplatform/doublets-rs/issues)

## License

This project is released into the public domain under the [Unlicense](LICENSE).
