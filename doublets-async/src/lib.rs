#![feature(type_alias_impl_trait)]
#![feature(impl_trait_in_assoc_type)]
#![allow(incomplete_features)]

//! Fast async API implementation for Doublets using Generic Associated Types (GATs) and 
//! Type Alias Impl Trait (TAIT) to avoid the performance overhead of `Pin<Box<dyn Future>>`.
//! 
//! This implementation provides zero-cost async abstractions for the Doublets data structure,
//! allowing for high-performance asynchronous operations without heap allocations for futures.

use std::future::Future;

/// Link address type that can be used as an identifier in the Doublets structure
pub trait LinkAddress: Copy + Clone + Send + Sync + 'static {}

impl<T> LinkAddress for T where T: Copy + Clone + Send + Sync + 'static {}

/// Handler type for read operations that can be called on each link during iteration
pub type ReadHandler<T> = dyn Fn(&[T]) -> bool + Send + Sync;

/// Handler type for write operations that can process creation/update operations
pub type WriteHandler<T> = dyn Fn(&[T], &[T]) -> T + Send + Sync;

/// Fast async Doublets trait using Generic Associated Types (GATs) to avoid
/// the performance overhead of Pin<Box<dyn Future>> from traditional async-trait.
/// 
/// This trait provides asynchronous operations for:
/// - Counting links with optional restrictions
/// - Iterating through links with handlers
/// - Creating new links
/// - Updating existing links  
/// - Deleting links
pub trait AsyncDoublets<TLinkAddress>
where
    TLinkAddress: LinkAddress,
{
    // Associated types for each async method using GATs
    type CountFuture<'a>: Future<Output = TLinkAddress> + Send + 'a
    where
        Self: 'a;
        
    type EachFuture<'a>: Future<Output = TLinkAddress> + Send + 'a
    where
        Self: 'a;
        
    type CreateFuture<'a>: Future<Output = TLinkAddress> + Send + 'a
    where
        Self: 'a;
        
    type UpdateFuture<'a>: Future<Output = TLinkAddress> + Send + 'a
    where
        Self: 'a;
        
    type DeleteFuture<'a>: Future<Output = TLinkAddress> + Send + 'a
    where
        Self: 'a;

    /// Count links matching the optional restriction criteria
    fn count<'a>(&'a self, restriction: Option<&'a [TLinkAddress]>) -> Self::CountFuture<'a>;

    /// Iterate through links matching the restriction, calling the handler for each
    fn each<'a>(&'a self, restriction: Option<&'a [TLinkAddress]>, handler: Option<&'a ReadHandler<TLinkAddress>>) -> Self::EachFuture<'a>;

    /// Create a new link with the given substitution and optional write handler
    fn create<'a>(&'a self, substitution: Option<&'a [TLinkAddress]>, handler: Option<&'a WriteHandler<TLinkAddress>>) -> Self::CreateFuture<'a>;

    /// Update links matching the restriction with the substitution values
    fn update<'a>(
        &'a self, 
        restriction: Option<&'a [TLinkAddress]>, 
        substitution: Option<&'a [TLinkAddress]>, 
        handler: Option<&'a WriteHandler<TLinkAddress>>
    ) -> Self::UpdateFuture<'a>;

    /// Delete links matching the restriction criteria
    fn delete<'a>(&'a self, restriction: Option<&'a [TLinkAddress]>, handler: Option<&'a WriteHandler<TLinkAddress>>) -> Self::DeleteFuture<'a>;
}

/// Example implementation of AsyncDoublets for demonstration
pub struct MemoryDoublets<TLinkAddress> {
    links: std::sync::RwLock<Vec<[TLinkAddress; 3]>>, // [index, source, target]
    next_id: std::sync::atomic::AtomicUsize,
}

impl<TLinkAddress> MemoryDoublets<TLinkAddress>
where
    TLinkAddress: LinkAddress + Default + From<usize> + Into<usize> + PartialEq,
{
    pub fn new() -> Self {
        Self {
            links: std::sync::RwLock::new(Vec::new()),
            next_id: std::sync::atomic::AtomicUsize::new(1),
        }
    }
}

impl<TLinkAddress> AsyncDoublets<TLinkAddress> for MemoryDoublets<TLinkAddress>
where
    TLinkAddress: LinkAddress + Default + From<usize> + Into<usize> + PartialEq,
{
    // Use Type Alias Impl Trait (TAIT) for zero-cost async abstractions
    type CountFuture<'a> = impl Future<Output = TLinkAddress> + Send + 'a where Self: 'a;
    type EachFuture<'a> = impl Future<Output = TLinkAddress> + Send + 'a where Self: 'a;
    type CreateFuture<'a> = impl Future<Output = TLinkAddress> + Send + 'a where Self: 'a;
    type UpdateFuture<'a> = impl Future<Output = TLinkAddress> + Send + 'a where Self: 'a;
    type DeleteFuture<'a> = impl Future<Output = TLinkAddress> + Send + 'a where Self: 'a;

    fn count<'a>(&'a self, restriction: Option<&'a [TLinkAddress]>) -> Self::CountFuture<'a> {
        async move {
            let links = self.links.read().unwrap();
            if let Some(restriction) = restriction {
                if restriction.is_empty() {
                    return TLinkAddress::from(links.len());
                }
                // Count links matching restriction
                let count = links.iter()
                    .filter(|link| {
                        match restriction.len() {
                            1 => link[0] == restriction[0], // Match by index
                            2 => link[1] == restriction[0] && link[2] == restriction[1], // Match source and target
                            3 => link[0] == restriction[0] && link[1] == restriction[1] && link[2] == restriction[2],
                            _ => false,
                        }
                    })
                    .count();
                TLinkAddress::from(count)
            } else {
                TLinkAddress::from(links.len())
            }
        }
    }

    fn each<'a>(&'a self, restriction: Option<&'a [TLinkAddress]>, handler: Option<&'a ReadHandler<TLinkAddress>>) -> Self::EachFuture<'a>
    {
        async move {
            let links = self.links.read().unwrap();
            let mut processed = 0;
            
            for link in links.iter() {
                let should_process = if let Some(restriction) = restriction {
                    match restriction.len() {
                        0 => true,
                        1 => link[0] == restriction[0],
                        2 => link[1] == restriction[0] && link[2] == restriction[1],
                        3 => link[0] == restriction[0] && link[1] == restriction[1] && link[2] == restriction[2],
                        _ => false,
                    }
                } else {
                    true
                };
                
                if should_process {
                    if let Some(handler) = handler {
                        if !handler(link) {
                            break; // Handler returned false, stop iteration
                        }
                    }
                    processed += 1;
                }
            }
            
            TLinkAddress::from(processed)
        }
    }

    fn create<'a>(&'a self, substitution: Option<&'a [TLinkAddress]>, handler: Option<&'a WriteHandler<TLinkAddress>>) -> Self::CreateFuture<'a>
    {
        async move {
            let id = self.next_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let link_id = TLinkAddress::from(id);
            
            let new_link = if let Some(sub) = substitution {
                match sub.len() {
                    2 => [link_id, sub[0], sub[1]], // Create link with source and target
                    1 => [link_id, sub[0], sub[0]], // Create self-referencing link
                    _ => [link_id, TLinkAddress::default(), TLinkAddress::default()],
                }
            } else {
                [link_id, TLinkAddress::default(), TLinkAddress::default()]
            };
            
            // Call handler if provided
            if let Some(handler) = handler {
                let result = handler(&[], &new_link);
                if result != TLinkAddress::default() {
                    // Handler modified the creation
                }
            }
            
            let mut links = self.links.write().unwrap();
            links.push(new_link);
            
            link_id
        }
    }

    fn update<'a>(
        &'a self, 
        restriction: Option<&'a [TLinkAddress]>, 
        substitution: Option<&'a [TLinkAddress]>, 
        handler: Option<&'a WriteHandler<TLinkAddress>>
    ) -> Self::UpdateFuture<'a>
    {
        async move {
            let mut links = self.links.write().unwrap();
            let mut updated_count = 0;
            
            if let Some(restriction) = restriction {
                if let Some(substitution) = substitution {
                    for link in links.iter_mut() {
                        let matches = match restriction.len() {
                            1 => link[0] == restriction[0],
                            2 => link[1] == restriction[0] && link[2] == restriction[1],
                            3 => *link == [restriction[0], restriction[1], restriction[2]],
                            _ => false,
                        };
                        
                        if matches {
                            let old_link = *link;
                            match substitution.len() {
                                2 => {
                                    link[1] = substitution[0];
                                    link[2] = substitution[1];
                                }
                                3 => *link = [substitution[0], substitution[1], substitution[2]],
                                _ => {}
                            }
                            
                            if let Some(handler) = handler {
                                let result = handler(&old_link, link);
                                if result != TLinkAddress::default() {
                                    // Handler provided additional processing
                                }
                            }
                            
                            updated_count += 1;
                        }
                    }
                }
            }
            
            TLinkAddress::from(updated_count)
        }
    }

    fn delete<'a>(&'a self, restriction: Option<&'a [TLinkAddress]>, handler: Option<&'a WriteHandler<TLinkAddress>>) -> Self::DeleteFuture<'a>
    {
        async move {
            let mut links = self.links.write().unwrap();
            let mut deleted_count = 0;
            
            if let Some(restriction) = restriction {
                let _original_len = links.len();
                links.retain(|link| {
                    let should_delete = match restriction.len() {
                        1 => link[0] == restriction[0],
                        2 => link[1] == restriction[0] && link[2] == restriction[1],
                        3 => *link == [restriction[0], restriction[1], restriction[2]],
                        _ => false,
                    };
                    
                    if should_delete {
                        if let Some(handler) = handler {
                            let result = handler(link, &[]);
                            if result != TLinkAddress::default() {
                                // Handler provided additional processing
                            }
                        }
                        deleted_count += 1;
                        false // Remove this link
                    } else {
                        true // Keep this link
                    }
                });
            }
            
            TLinkAddress::from(deleted_count)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_doublets_basic_operations() {
        let doublets = MemoryDoublets::<usize>::new();
        
        // Test count on empty doublets
        let count = doublets.count(None).await;
        assert_eq!(count, 0);
        
        // Create a link
        let link_id = doublets.create(Some(&[2, 3]), None).await;
        assert_eq!(link_id, 1);
        
        // Count should now be 1
        let count = doublets.count(None).await;
        assert_eq!(count, 1);
        
        // Create another link
        let link_id2 = doublets.create(Some(&[4, 5]), None).await;
        assert_eq!(link_id2, 2);
        
        // Test each operation
        let mut visited_links = Vec::new();
        let handler = |link: &[usize]| -> bool {
            visited_links.push(link.to_vec());
            true
        };
        
        let processed = doublets.each(None, Some(&handler)).await;
        assert_eq!(processed, 2);
        
        // Test update
        let updated = doublets.update(Some(&[1]), Some(&[6, 7]), None).await;
        assert_eq!(updated, 1);
        
        // Test delete
        let deleted = doublets.delete(Some(&[2]), None).await;
        assert_eq!(deleted, 1);
        
        // Final count should be 1
        let final_count = doublets.count(None).await;
        assert_eq!(final_count, 1);
    }
}