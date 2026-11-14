//! Split-memory store implementation for doublets.
//!
//! Link data (`source`, `target`) is kept in a [`DataPart`] memory region while
//! tree-index metadata lives in a separate [`IndexPart`] memory region.

mod data_part;
mod generic;
mod index_part;
mod store;

pub use data_part::DataPart;
pub use index_part::IndexPart;

pub use generic::*;
pub use store::Store;
