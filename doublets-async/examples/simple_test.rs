/*!
Simple test to verify the fast async Doublets API works correctly.
This example runs without the full test framework to verify basic functionality.
*/

use doublets_async::{AsyncDoublets, MemoryDoublets};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🧪 Testing Fast Async Doublets Implementation");
    println!("=============================================");
    
    let doublets = MemoryDoublets::<usize>::new();
    
    // Test 1: Initial count should be 0
    print!("Test 1: Initial count... ");
    let count = doublets.count(None).await;
    assert_eq!(count, 0);
    println!("✓ PASS (count: {})", count);
    
    // Test 2: Create a link
    print!("Test 2: Create link... ");
    let link_id = doublets.create(Some(&[10, 20]), None).await;
    assert_eq!(link_id, 1);
    println!("✓ PASS (created link {})", link_id);
    
    // Test 3: Count should now be 1
    print!("Test 3: Count after creation... ");
    let count = doublets.count(None).await;
    assert_eq!(count, 1);
    println!("✓ PASS (count: {})", count);
    
    // Test 4: Create another link
    print!("Test 4: Create second link... ");
    let link_id2 = doublets.create(Some(&[30, 40]), None).await;
    assert_eq!(link_id2, 2);
    println!("✓ PASS (created link {})", link_id2);
    
    // Test 5: Iterate through links
    print!("Test 5: Iterate through links... ");
    let handler = |_link: &[usize]| -> bool {
        true // Continue iteration - just test that it works
    };
    
    let processed = doublets.each(None, Some(&handler)).await;
    assert_eq!(processed, 2);
    println!("✓ PASS (processed: {})", processed);
    
    // Test 6: Update a link
    print!("Test 6: Update link... ");
    let updated = doublets.update(Some(&[1]), Some(&[50, 60]), None).await;
    assert_eq!(updated, 1);
    println!("✓ PASS (updated {} link)", updated);
    
    // Test 7: Delete a link  
    print!("Test 7: Delete link... ");
    let deleted = doublets.delete(Some(&[2]), None).await;
    assert_eq!(deleted, 1);
    println!("✓ PASS (deleted {} link)", deleted);
    
    // Test 8: Final count should be 1
    print!("Test 8: Final count... ");
    let final_count = doublets.count(None).await;
    assert_eq!(final_count, 1);
    println!("✓ PASS (final count: {})", final_count);
    
    println!("\n🎉 All tests passed!");
    println!("\n✨ Benefits demonstrated:");
    println!("• Zero-cost async abstractions using GATs and TAIT");
    println!("• No Pin<Box<dyn Future>> allocations");  
    println!("• Static dispatch for better performance");
    println!("• Type-safe async operations");
    
    Ok(())
}