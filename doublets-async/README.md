# Fast Async Doublets API (Rust Implementation)

This is a high-performance Rust implementation of the async Doublets API that solves the performance issues mentioned in [issue #290](https://github.com/linksplatform/Data.Doublets/issues/290).

## 🚀 Key Features

- **Zero-cost async abstractions** using Generic Associated Types (GATs) and Type Alias Impl Trait (TAIT)
- **No heap allocations** for futures (no `Pin<Box<dyn Future>>`)
- **Static dispatch** for better performance and smaller binary size
- **Type-safe async operations** for all Doublets operations

## 🎯 Problem Solved

Traditional `async-trait` approaches suffer from performance overhead due to:
- Heap allocations for each async call (`Pin<Box<dyn Future>>`)
- Dynamic dispatch overhead
- Larger binary sizes
- Poor compiler optimizations across async boundaries

This implementation eliminates these issues by using:
- **Generic Associated Types (GATs)** to associate concrete future types with trait implementations
- **Type Alias Impl Trait (TAIT)** to define implementation-specific future types without boxing

## 🔧 Usage

### Basic Operations

```rust
use platform_data_doublets::{AsyncDoublets, MemoryDoublets};

#[tokio::main]
async fn main() {
    let doublets = MemoryDoublets::<usize>::new();
    
    // Count links
    let count = doublets.count(None).await;
    
    // Create a link
    let link_id = doublets.create(Some(&[2, 3]), None).await;
    
    // Iterate through links
    let handler = |link: &[usize]| -> bool {
        println!("Link: {:?}", link);
        true // Continue iteration
    };
    doublets.each(None, Some(&handler)).await;
    
    // Update a link
    doublets.update(Some(&[link_id]), Some(&[4, 5]), None).await;
    
    // Delete a link
    doublets.delete(Some(&[link_id]), None).await;
}
```

### Performance Comparison

```rust
// Traditional async-trait approach (what we avoid):
#[async_trait]
trait SlowAsyncDoublets {
    async fn count(&self, restriction: Option<&[usize]>) -> usize;
    // Each call allocates Pin<Box<dyn Future<Output = usize>>>
}

// Our fast GATs + TAIT approach:
trait AsyncDoublets<TLinkAddress> {
    type CountFuture<'a>: Future<Output = TLinkAddress> + Send + 'a where Self: 'a;
    fn count<'a>(&'a self, restriction: Option<&'a [TLinkAddress]>) -> Self::CountFuture<'a>;
    // Zero allocations, static dispatch
}
```

## 🏗️ Architecture

### Core Trait

The `AsyncDoublets` trait defines the async interface using GATs:

```rust
pub trait AsyncDoublets<TLinkAddress> {
    type CountFuture<'a>: Future<Output = TLinkAddress> + Send + 'a where Self: 'a;
    type EachFuture<'a>: Future<Output = TLinkAddress> + Send + 'a where Self: 'a;
    type CreateFuture<'a>: Future<Output = TLinkAddress> + Send + 'a where Self: 'a;
    type UpdateFuture<'a>: Future<Output = TLinkAddress> + Send + 'a where Self: 'a;
    type DeleteFuture<'a>: Future<Output = TLinkAddress> + Send + 'a where Self: 'a;

    fn count<'a>(&'a self, restriction: Option<&'a [TLinkAddress]>) -> Self::CountFuture<'a>;
    fn each<'a>(&'a self, restriction: Option<&'a [TLinkAddress]>, handler: Option<&'a ReadHandler<TLinkAddress>>) -> Self::EachFuture<'a>;
    fn create<'a>(&'a self, substitution: Option<&'a [TLinkAddress]>, handler: Option<&'a WriteHandler<TLinkAddress>>) -> Self::CreateFuture<'a>;
    fn update<'a>(&'a self, restriction: Option<&'a [TLinkAddress]>, substitution: Option<&'a [TLinkAddress]>, handler: Option<&'a WriteHandler<TLinkAddress>>) -> Self::UpdateFuture<'a>;
    fn delete<'a>(&'a self, restriction: Option<&'a [TLinkAddress]>, handler: Option<&'a WriteHandler<TLinkAddress>>) -> Self::DeleteFuture<'a>;
}
```

### Implementation with TAIT

```rust
impl<TLinkAddress> AsyncDoublets<TLinkAddress> for MemoryDoublets<TLinkAddress> {
    // Use TAIT to define concrete future types without boxing
    type CountFuture<'a> = impl Future<Output = TLinkAddress> + Send + 'a where Self: 'a;
    
    fn count<'a>(&'a self, restriction: Option<&'a [TLinkAddress]>) -> Self::CountFuture<'a> {
        async move {
            // Implementation here - no heap allocation!
        }
    }
}
```

## 📊 Performance Benefits

Based on our benchmarks:
- **1000 link creation**: ~300μs
- **1000 link counting**: ~1.4μs  
- **1000 link iteration**: ~27μs

These operations have:
- Zero heap allocations for futures
- Optimal compiler optimizations
- Static dispatch throughout

## 🧪 Testing

Run the test suite:
```bash
cargo test
```

Run examples:
```bash
cargo run --example simple_test
cargo run --example comparison
```

Run benchmarks:
```bash
cargo bench
```

## 🔬 Required Rust Features

This implementation requires the following unstable Rust features:
- `#![feature(type_alias_impl_trait)]` - For TAIT support
- `#![feature(impl_trait_in_assoc_type)]` - For `impl Trait` in associated types

These features are available on nightly Rust and are expected to stabilize soon.

## 📈 Future Improvements

1. **Storage Backends**: Add support for different storage backends (disk, network, etc.)
2. **Persistence**: Implement persistent storage options
3. **Transactions**: Add transactional support for atomic operations
4. **Indexing**: Implement advanced indexing strategies
5. **Benchmarking**: Compare against other async data structure implementations

## 🤝 Contributing

This implementation demonstrates the solution proposed in issue #290. The core concepts can be extended to other async trait implementations in the Rust ecosystem.

## 📜 License

This project is licensed under the Unlicense - see the [LICENSE](../LICENSE) file for details.