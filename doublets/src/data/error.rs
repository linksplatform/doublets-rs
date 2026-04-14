use crate::{Doublet, Link};
use data::LinkReference;
use std::{error::Error as StdError, io};

/// Errors that can be returned by doublets store operations.
#[derive(thiserror::Error, Debug)]
pub enum Error<T: LinkReference> {
    /// The requested link index does not exist in the store.
    #[error("link {0} does not exist.")]
    NotExists(T),

    /// The link cannot be deleted because other links still reference it.
    #[error("link {0:?} has dependencies")]
    HasUsages(Vec<Link<T>>),

    /// A link with the same `(source, target)` pair already exists.
    #[error("link {0} already exists")]
    AlreadyExists(Doublet<T>),

    /// The store has reached its maximum link capacity.
    #[error("limit for the number of links in the storage has been reached: {0}")]
    LimitReached(T),

    /// A memory allocation or I/O error occurred in the underlying storage.
    #[error("unable to allocate memory for links storage: `{0}`")]
    AllocFailed(#[from] mem::Error),

    /// Any other internal error.
    #[error("other internal error: `{0}`")]
    Other(#[from] Box<dyn StdError + Sync + Send>),
}

impl<T: LinkReference> From<io::Error> for Error<T> {
    fn from(err: io::Error) -> Self {
        Self::AllocFailed(err.into())
    }
}
