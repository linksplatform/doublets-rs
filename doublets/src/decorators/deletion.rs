//! Deletion decorator that resets a link's contents before deleting it.
//!
//! Port of C# `NonNullContentsLinkDeletionResolver`.

use std::{
    fmt::{self, Debug, Formatter},
    marker::PhantomData,
};

use data::{Flow, LinkReference, LinksConstants};

use super::{
    helpers,
    macros::{decorator_struct, forward},
};
use crate::{
    data::{ReadHandler, WriteHandler},
    Doublets, Error, Fuse, Link, Links,
};

decorator_struct! {
    /// Resets a link to `(null, null)` before deleting it, so the store's indexes are
    /// updated before the link disappears.
    ///
    /// Port of C# `NonNullContentsLinkDeletionResolver`.
    NonNullContentsLinkDeletionResolver
}

impl<T: LinkReference, L: Doublets<T>> Links<T> for NonNullContentsLinkDeletionResolver<T, L> {
    forward!(count_links, each_links, create_links, update_links);

    #[inline]
    fn delete_links(
        &mut self,
        query: &[T],
        handler: WriteHandler<'_, T>,
    ) -> Result<Flow, Error<T>> {
        let index = helpers::part(query, self.links.constants().index_part);
        let mut fuse = Fuse::new(&mut *handler);
        helpers::enforce_reset_values(&mut self.links, index, &mut |before, after| {
            fuse.call(before, after)
        })?;
        self.links
            .delete_links(query, &mut |before, after| fuse.call(before, after))
    }
}
