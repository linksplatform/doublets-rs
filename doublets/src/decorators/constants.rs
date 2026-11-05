//! Decorators that give a service constant a meaning inside a substitution.
//!
//! Ports of C# `LinksItselfConstantToSelfReferenceResolver` and
//! `LinksNullConstantToSelfReferenceResolver`.

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
    Doublets, Error, Link, Links,
};

decorator_struct! {
    /// Turns the `itself` constant into a reference to the link being written.
    ///
    /// Port of C# `LinksItselfConstantToSelfReferenceResolver`. A query that contains
    /// `itself` can never match a stored link, so [`Links::each_links`] short-circuits to
    /// an empty result.
    ItselfConstantToSelfReferenceResolver
}

impl<T: LinkReference, L: Doublets<T>> Links<T> for ItselfConstantToSelfReferenceResolver<T, L> {
    forward!(count_links, create_links, delete_links);

    #[inline]
    fn each_links(&self, query: &[T], handler: ReadHandler<'_, T>) -> Flow {
        let constants = self.links.constants();
        if constants.any != constants.itself && query.contains(&constants.itself) {
            return Flow::Continue;
        }
        self.links.each_links(query, handler)
    }

    #[inline]
    fn update_links(
        &mut self,
        query: &[T],
        change: &[T],
        handler: WriteHandler<'_, T>,
    ) -> Result<Flow, Error<T>> {
        let constants = self.links.constants();
        let change =
            helpers::resolve_constant_as_self_reference(constants.itself, query, change, constants);
        self.links.update_links(query, &change, handler)
    }
}

decorator_struct! {
    /// Turns the `null` constant into a reference to the link being written, and makes a
    /// bare create produce a self-referential point.
    ///
    /// Port of C# `LinksNullConstantToSelfReferenceResolver`.
    NullConstantToSelfReferenceResolver
}

impl<T: LinkReference, L: Doublets<T>> Links<T> for NullConstantToSelfReferenceResolver<T, L> {
    forward!(count_links, each_links, delete_links);

    #[inline]
    fn create_links(
        &mut self,
        _query: &[T],
        handler: WriteHandler<'_, T>,
    ) -> Result<Flow, Error<T>> {
        helpers::create_point_with(&mut self.links, &mut *handler)
    }

    #[inline]
    fn update_links(
        &mut self,
        query: &[T],
        change: &[T],
        handler: WriteHandler<'_, T>,
    ) -> Result<Flow, Error<T>> {
        let constants = self.links.constants();
        let change =
            helpers::resolve_constant_as_self_reference(constants.null, query, change, constants);
        self.links.update_links(query, &change, handler)
    }
}
