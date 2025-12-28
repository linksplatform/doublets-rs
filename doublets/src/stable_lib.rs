// Minimal doublets functionality for stable Rust compilation
// This module provides basic functionality without nightly features

/// A basic link representation for stable Rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StableLink<T> {
    pub index: T,
    pub source: T,
    pub target: T,
}

impl<T> StableLink<T> {
    pub fn new(index: T, source: T, target: T) -> Self {
        Self { index, source, target }
    }
}

/// Basic error type for stable Rust
#[derive(Debug, Clone)]
pub enum StableError {
    NotImplemented,
    InvalidOperation,
}

impl std::fmt::Display for StableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StableError::NotImplemented => write!(f, "Feature not implemented in stable mode"),
            StableError::InvalidOperation => write!(f, "Invalid operation"),
        }
    }
}

impl std::error::Error for StableError {}

/// Basic operations available with stable Rust
pub trait StableDoublets<T> {
    fn create_link(&mut self, source: T, target: T) -> Result<T, StableError>;
    fn delete_link(&mut self, link: T) -> Result<(), StableError>;
    fn count(&self) -> usize;
}

/// Memory-based implementation for stable Rust
pub struct StableMemoryStore<T> {
    links: Vec<StableLink<T>>,
    next_index: T,
}

impl<T> StableMemoryStore<T> 
where 
    T: Copy + Clone + PartialEq + From<usize> + Into<usize> + std::fmt::Debug,
{
    pub fn new() -> Self {
        Self {
            links: Vec::new(),
            next_index: T::from(1),
        }
    }
}

impl<T> StableDoublets<T> for StableMemoryStore<T>
where 
    T: Copy + Clone + PartialEq + From<usize> + Into<usize> + std::fmt::Debug,
{
    fn create_link(&mut self, source: T, target: T) -> Result<T, StableError> {
        let index = self.next_index;
        let link = StableLink::new(index, source, target);
        self.links.push(link);
        
        // Increment next_index
        let next_val: usize = self.next_index.into() + 1;
        self.next_index = T::from(next_val);
        
        Ok(index)
    }
    
    fn delete_link(&mut self, link_index: T) -> Result<(), StableError> {
        if let Some(pos) = self.links.iter().position(|l| l.index == link_index) {
            self.links.remove(pos);
            Ok(())
        } else {
            Err(StableError::InvalidOperation)
        }
    }
    
    fn count(&self) -> usize {
        self.links.len()
    }
}