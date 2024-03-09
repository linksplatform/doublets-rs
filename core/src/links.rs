use std::{error, io};

type Repr<T> = Box<[T]>;

#[derive(thiserror::Error, Debug)]
pub enum Error<T: crate::LinkType> {
    #[error("link `{0}` does not exist")]
    NotExists(T),

    #[error("link `{0:?}` cannot change link because it is also has usages")]
    HasUsages(Box<[Repr<T>]>),

    #[error("link `{0}` already exists")]
    AlreadyExists(Repr<T>),

    #[error("limit for the number of links in the storage has been reached: `{0}`")]
    LimitReached(T),

    #[error("unable to allocate memory for links storage: `{0}`")]
    AllocFailed(#[from] io::Error),

    #[error("other internal error: `{0}`")]
    Other(#[from] Box<dyn error::Error + Sync + Send>),
}

fn _assert_send_sync<T: crate::LinkType>() {
    fn _assert_send_sync<T: Send + Sync>() {}
    _assert_send_sync::<Error<T>>();
}

// todo: general `Links` trait is undefined now due to lack of context
