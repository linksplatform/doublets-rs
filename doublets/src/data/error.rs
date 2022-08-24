use crate::{Doublet, Link};
use data::LinkType;
use std::{error::Error as StdError, io};

#[derive(thiserror::Error, Debug)]
pub enum Error<T: LinkType> {
    #[error("link {0} does not exist.")]
    NotExists(T),

    #[error("link {0:?} has dependencies")]
    HasUsages(Box<[Link<T>]>),

    #[error("link {0} already exists")]
    AlreadyExists(Doublet<T>),

    #[error("limit for the number of links in the storage has been reached: {0}")]
    LimitReached(T),

    #[error("unable to allocate memory for links storage: `{0}`")]
    AllocFailed(#[from] mem::Error),

    #[error("other internal error: `{0}`")]
    Other(#[from] Box<dyn StdError + Sync + Send>),
}

impl<T: LinkType> From<io::Error> for Error<T> {
    fn from(err: io::Error) -> Self {
        Self::AllocFailed(err.into())
    }
}
