use std::fmt::{Debug, Display, Formatter};

use data::LinkReference;

/// A `(source, target)` pair that identifies a link by its endpoints without an index.
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Doublet<T: LinkReference> {
    /// The source (left) endpoint.
    pub source: T,
    /// The target (right) endpoint.
    pub target: T,
}

impl<T: LinkReference> Doublet<T> {
    /// Creates a new [`Doublet`] with the given `source` and `target`.
    pub fn new(source: T, target: T) -> Self {
        Self { source, target }
    }
}

impl<T: LinkReference> Display for Doublet<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}->{}", self.source, self.target)
    }
}
