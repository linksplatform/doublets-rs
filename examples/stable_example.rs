// Example demonstrating doublets functionality with stable Rust
// This compiles without any nightly features

use doublets::{StableDoublets, StableMemoryStore};

fn main() {
    println!("Doublets-rs: Stable Rust Example");
    
    // Create a basic in-memory doublets store
    let mut store = StableMemoryStore::<usize>::new();
    
    // Create some links
    let link1 = store.create_link(1, 2).expect("Failed to create link");
    let link2 = store.create_link(2, 3).expect("Failed to create link");
    let link3 = store.create_link(1, 3).expect("Failed to create link");
    
    println!("Created {} links", store.count());
    println!("Link IDs: {}, {}, {}", link1, link2, link3);
    
    // Delete a link
    store.delete_link(link2).expect("Failed to delete link");
    println!("After deletion: {} links", store.count());
    
    println!("✓ Compiled successfully with stable Rust!");
    println!("This demonstrates basic doublets functionality without nightly features.");
    println!("For full functionality, enable the 'data' and 'mem' feature flags with nightly Rust.");
}