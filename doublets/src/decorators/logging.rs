//! Decorator that writes a line to a log for every mutation.
//!
//! Port of C# `LoggingDecorator`.

use std::{
    fmt::{self, Debug, Formatter},
    io::Write,
    marker::PhantomData,
};

use data::{Flow, LinkReference, LinksConstants};

use super::helpers;
use crate::{
    data::{ReadHandler, WriteHandler},
    Doublets, Error, Fuse, Link, Links,
};

/// Logs every create, update and delete to a writer, in the format C#
/// `LoggingDecorator` uses.
///
/// Each mutation appends one line, for example:
///
/// ```text
/// Create. Before: (0->0). After: (3: 0->0)
/// Update. Before: (3: 0->0). After: (3: 1->2)
/// Delete. Before: (3: 1->2). After: (0->0)
/// ```
///
/// # Deviation from C\#
///
/// C# writes to an auto-flushing `StreamWriter` and drops I/O errors. This port keeps the
/// first I/O error and returns it as [`Error::AllocFailed`] once the operation finishes,
/// so a broken log cannot pass unnoticed. Wrap the writer in a
/// [`BufWriter`](std::io::BufWriter) if you want C#'s buffering behaviour, and flush it
/// yourself.
pub struct LoggingDecorator<T: LinkReference, L: Doublets<T>, W: Write + Send + Sync> {
    links: L,
    writer: W,
    _marker: PhantomData<fn(T)>,
}

impl<T: LinkReference, L: Doublets<T>, W: Write + Send + Sync> LoggingDecorator<T, L, W> {
    /// Wraps `links` so that every mutation is logged to `writer`.
    #[inline]
    pub const fn new(links: L, writer: W) -> Self {
        Self {
            links,
            writer,
            _marker: PhantomData,
        }
    }

    /// Returns a shared reference to the wrapped store.
    #[inline]
    #[must_use]
    pub const fn inner(&self) -> &L {
        &self.links
    }

    /// Returns a mutable reference to the wrapped store.
    #[inline]
    #[must_use]
    pub const fn inner_mut(&mut self) -> &mut L {
        &mut self.links
    }

    /// Returns a mutable reference to the log writer.
    #[inline]
    #[must_use]
    pub const fn writer_mut(&mut self) -> &mut W {
        &mut self.writer
    }

    /// Unwraps the decorator and returns the wrapped store and the log writer.
    #[inline]
    #[must_use]
    pub fn into_inner(self) -> (L, W) {
        (self.links, self.writer)
    }
}

impl<T: LinkReference, L: Doublets<T> + Debug, W: Write + Send + Sync> Debug
    for LoggingDecorator<T, L, W>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("LoggingDecorator")
            .field(&self.links)
            .finish()
    }
}

impl<T: LinkReference, L: Doublets<T>, W: Write + Send + Sync> Doublets<T>
    for LoggingDecorator<T, L, W>
{
    #[inline]
    fn get_link(&self, index: T) -> Option<Link<T>> {
        self.links.get_link(index)
    }
}

/// Runs `operation` with a handler that both forwards to `handler` and appends a log line
/// tagged with `tag`.
macro_rules! logged {
    ($self:ident, $tag:literal, $handler:ident, |$links:ident, $logger:ident| $operation:expr) => {{
        let Self {
            links: $links,
            writer,
            ..
        } = $self;
        let mut fuse = Fuse::new(|before, after| $handler(before, after));
        let mut failure = None;
        let mut $logger = |before: Link<T>, after: Link<T>| {
            fuse.call(before.clone(), after.clone());
            if failure.is_none() {
                if let Err(err) = writeln!(
                    writer,
                    concat!($tag, ". Before: {}. After: {}"),
                    helpers::format_link(&before),
                    helpers::format_link(&after)
                ) {
                    failure = Some(err);
                }
            }
            Flow::Continue
        };
        let flow = $operation?;
        match failure {
            Some(err) => Err(Error::from(err)),
            None => Ok(flow),
        }
    }};
}

impl<T: LinkReference, L: Doublets<T>, W: Write + Send + Sync> Links<T>
    for LoggingDecorator<T, L, W>
{
    #[inline]
    fn constants(&self) -> &LinksConstants<T> {
        self.links.constants()
    }

    #[inline]
    fn count_links(&self, query: &[T]) -> T {
        self.links.count_links(query)
    }

    #[inline]
    fn each_links(&self, query: &[T], handler: ReadHandler<'_, T>) -> Flow {
        self.links.each_links(query, handler)
    }

    #[inline]
    fn create_links(
        &mut self,
        query: &[T],
        handler: WriteHandler<'_, T>,
    ) -> Result<Flow, Error<T>> {
        logged!(self, "Create", handler, |links, logger| links
            .create_links(query, &mut logger))
    }

    #[inline]
    fn update_links(
        &mut self,
        query: &[T],
        change: &[T],
        handler: WriteHandler<'_, T>,
    ) -> Result<Flow, Error<T>> {
        logged!(self, "Update", handler, |links, logger| links.update_links(
            query,
            change,
            &mut logger
        ))
    }

    #[inline]
    fn delete_links(
        &mut self,
        query: &[T],
        handler: WriteHandler<'_, T>,
    ) -> Result<Flow, Error<T>> {
        logged!(self, "Delete", handler, |links, logger| links
            .delete_links(query, &mut logger))
    }
}
