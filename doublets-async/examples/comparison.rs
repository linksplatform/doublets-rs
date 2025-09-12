/*!
# Fast Async Doublets API Comparison

This example demonstrates the difference between the traditional async-trait approach
and the fast async API using GATs (Generic Associated Types) and TAIT (Type Alias Impl Trait).

## Performance Benefits

1. **Zero heap allocations**: No `Pin<Box<dyn Future>>` boxing
2. **Static dispatch**: Compile-time function resolution
3. **Smaller binary size**: No dynamic dispatch overhead
4. **Better inlining**: Compiler can optimize across async boundaries

## Usage Comparison

Traditional async-trait approach would look like:
```rust
// This is what we AVOID:
#[async_trait]
trait SlowAsyncDoublets {
    async fn count(&self, restriction: Option<&[usize]>) -> usize;
    // Each method returns Pin<Box<dyn Future<Output = T> + Send + '_>>
}
```

Our fast approach:
```rust
trait AsyncDoublets<TLinkAddress> {
    type CountFuture<'a>: Future<Output = TLinkAddress> + Send + 'a where Self: 'a;
    fn count<'a>(&'a self, restriction: Option<&'a [TLinkAddress]>) -> Self::CountFuture<'a>;
}
```
*/

use doublets_async::{AsyncDoublets, MemoryDoublets};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Fast Async Doublets API Demo");
    println!("================================");
    
    // Create a new doublets instance
    let doublets = MemoryDoublets::<usize>::new();
    
    println!("\n📊 Basic Operations:");
    
    // Demonstrate basic operations
    let start = Instant::now();
    
    // Count empty doublets
    let initial_count = doublets.count(None).await;
    println!("Initial count: {}", initial_count);
    
    // Create some links
    println!("\n🔗 Creating links...");
    let mut link_ids = Vec::new();
    for i in 1..=5 {
        let id = doublets.create(Some(&[i * 2, i * 2 + 1]), None).await;
        link_ids.push(id);
        println!("Created link {} with source {} and target {}", id, i * 2, i * 2 + 1);
    }
    
    // Count after creation
    let after_create_count = doublets.count(None).await;
    println!("\nCount after creation: {}", after_create_count);
    
    // Demonstrate iteration
    println!("\n📋 Iterating through all links:");
    let collect_handler = |link: &[usize]| -> bool {
        println!("  Link {}: {} -> {}", link[0], link[1], link[2]);
        true // Continue iteration
    };
    
    let processed = doublets.each(None, Some(&collect_handler)).await;
    println!("Processed {} links", processed);
    
    // Demonstrate update
    println!("\n✏️  Updating link 3:");
    let updated = doublets.update(Some(&[3]), Some(&[99, 100]), None).await;
    println!("Updated {} link(s)", updated);
    
    // Show the change
    let show_updated = |link: &[usize]| -> bool {
        if link[0] == 3 {
            println!("  Updated Link {}: {} -> {}", link[0], link[1], link[2]);
        }
        true
    };
    doublets.each(Some(&[3]), Some(&show_updated)).await;
    
    // Demonstrate deletion
    println!("\n🗑️  Deleting link 5:");
    let deleted = doublets.delete(Some(&[5]), None).await;
    println!("Deleted {} link(s)", deleted);
    
    // Final count
    let final_count = doublets.count(None).await;
    println!("\nFinal count: {}", final_count);
    
    let elapsed = start.elapsed();
    println!("\n⚡ All operations completed in {:?}", elapsed);
    
    // Demonstrate performance characteristics
    println!("\n🎯 Performance Demonstration:");
    demonstrate_performance().await;
    
    Ok(())
}

async fn demonstrate_performance() {
    let doublets = MemoryDoublets::<usize>::new();
    
    println!("Creating 1000 links and measuring performance...");
    let start = Instant::now();
    
    // Batch create links
    for i in 1..=1000 {
        doublets.create(Some(&[i, i + 1000]), None).await;
    }
    
    let creation_time = start.elapsed();
    println!("Created 1000 links in {:?}", creation_time);
    
    // Count them
    let start = Instant::now();
    let count = doublets.count(None).await;
    let count_time = start.elapsed();
    println!("Counted {} links in {:?}", count, count_time);
    
    // Iterate through all  
    let start = Instant::now();
    let counter = |_: &[usize]| -> bool { true };
    
    let processed = doublets.each(None, Some(&counter)).await;
    let iteration_time = start.elapsed();
    println!("Iterated through {} links in {:?}", processed, iteration_time);
    
    println!("\n✨ Benefits of GATs + TAIT approach:");
    println!("• No heap allocations for futures");
    println!("• Zero-cost abstractions");
    println!("• Static dispatch");
    println!("• Better compiler optimizations");
    println!("• Smaller binary size");
}