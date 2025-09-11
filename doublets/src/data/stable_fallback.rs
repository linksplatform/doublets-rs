// Stable Rust fallback implementations when platform dependencies are not available

/// Basic LinkType trait for stable compilation
pub trait LinkType: Copy + Clone + PartialEq + PartialOrd + std::fmt::Debug {
    // Basic functionality available on stable
}

/// Basic Flow enum for stable compilation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Flow<T> {
    Continue,
    Break(T),
}

impl<T> Flow<T> {
    pub fn into_break(self) -> Option<T> {
        match self {
            Flow::Break(t) => Some(t),
            Flow::Continue => None,
        }
    }
}

// Implement LinkType for basic numeric types
impl LinkType for u8 {}
impl LinkType for u16 {}
impl LinkType for u32 {}
impl LinkType for u64 {}
impl LinkType for usize {}