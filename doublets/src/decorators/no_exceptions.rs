//! Decorator that turns a failing mutation into a [`Flow::Break`] instead of an error.
//!
//! Port of C# `NoExceptionsDecorator`.

use std::{
    fmt::{self, Debug, Formatter},
    marker::PhantomData,
};

use data::{Flow, LinkReference, LinksConstants};

use super::macros::{decorator_struct, forward};
use crate::{
    data::{ReadHandler, WriteHandler},
    Doublets, Error, Link, Links,
};

decorator_struct! {
    /// Swallows every error a mutation returns, reporting it as [`Flow::Break`].
    ///
    /// Port of C# `NoExceptionsDecorator`.
    ///
    /// # Deviation from C\#
    ///
    /// C# catches exceptions from `Count` and `Each` as well and returns the `Error`
    /// constant. In Rust `count_links` and `each_links` cannot fail, and [`Flow`] has no
    /// error value, so this port maps `Err(_)` to `Ok(Flow::Break)` on the three fallible
    /// operations and forwards the two infallible ones unchanged.
    ///
    /// Panics are *not* caught: this decorator handles errors, not bugs.
    NoExceptionsDecorator
}

impl<T: LinkReference, L: Doublets<T>> Links<T> for NoExceptionsDecorator<T, L> {
    forward!(count_links, each_links);

    #[inline]
    fn create_links(
        &mut self,
        query: &[T],
        handler: WriteHandler<'_, T>,
    ) -> Result<Flow, Error<T>> {
        Ok(self
            .links
            .create_links(query, handler)
            .unwrap_or(Flow::Break))
    }

    #[inline]
    fn update_links(
        &mut self,
        query: &[T],
        change: &[T],
        handler: WriteHandler<'_, T>,
    ) -> Result<Flow, Error<T>> {
        Ok(self
            .links
            .update_links(query, change, handler)
            .unwrap_or(Flow::Break))
    }

    #[inline]
    fn delete_links(
        &mut self,
        query: &[T],
        handler: WriteHandler<'_, T>,
    ) -> Result<Flow, Error<T>> {
        Ok(self
            .links
            .delete_links(query, handler)
            .unwrap_or(Flow::Break))
    }
}
