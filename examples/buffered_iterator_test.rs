// Example demonstrating the use of buffered iterators to improve search performance
// This script shows the difference between the old Vec approach and the new buter approach

use std::time::Instant;

#[cfg(feature = "buffered-iter")]
use buter::Buter;

// Mock data structures to simulate the doublets functionality
struct MockDoublets {
    links: Vec<usize>,
}

impl MockDoublets {
    fn new(size: usize) -> Self {
        Self {
            links: (1..=size).collect(),
        }
    }

    // Old approach using Vec
    fn each_iter_vec(&self) -> impl Iterator<Item = usize> {
        let mut vec = Vec::with_capacity(self.links.len());
        for &link in &self.links {
            vec.push(link);
        }
        vec.into_iter()
    }

    // New approach using buffered iterators
    #[cfg(feature = "buffered-iter")]
    fn each_iter_buffered(&self) -> impl Iterator<Item = usize> {
        let buter = Buter::with_capacity(self.links.len());
        let writer = buter.writer();
        for &link in &self.links {
            writer.extend(Some(link));
        }
        writer.into_iter().collect::<Vec<_>>().into_iter()
    }

    #[cfg(not(feature = "buffered-iter"))]
    fn each_iter_buffered(&self) -> impl Iterator<Item = usize> {
        // Fall back to Vec approach when buffered-iter feature is not enabled
        self.each_iter_vec()
    }
}

fn benchmark_approach<F, I>(name: &str, mut f: F, iterations: usize) 
where 
    F: FnMut() -> I,
    I: Iterator<Item = usize>,
{
    let start = Instant::now();
    
    for _ in 0..iterations {
        let _: Vec<_> = f().collect();
    }
    
    let elapsed = start.elapsed();
    println!("{}: {:?} ({} iterations)", name, elapsed, iterations);
}

fn main() {
    println!("Buffered Iterator Performance Test");
    println!("===================================");
    
    let doublets = MockDoublets::new(10000);
    let iterations = 100;
    
    // Benchmark Vec approach
    benchmark_approach(
        "Vec approach", 
        || doublets.each_iter_vec(), 
        iterations
    );
    
    // Benchmark buffered iterator approach
    benchmark_approach(
        "Buffered iterator approach", 
        || doublets.each_iter_buffered(), 
        iterations
    );
    
    println!();
    
    // Verify both approaches produce the same results
    let vec_result: Vec<_> = doublets.each_iter_vec().take(10).collect();
    let buffered_result: Vec<_> = doublets.each_iter_buffered().take(10).collect();
    
    println!("Correctness Test:");
    println!("Vec result: {:?}", vec_result);
    println!("Buffered result: {:?}", buffered_result);
    
    if vec_result == buffered_result {
        println!("✓ Both approaches produce identical results");
    } else {
        println!("✗ Results differ!");
    }
    
    #[cfg(feature = "buffered-iter")]
    println!("Note: Running with buffered-iter feature enabled");
    
    #[cfg(not(feature = "buffered-iter"))]
    println!("Note: Running without buffered-iter feature (fallback to Vec)");
}