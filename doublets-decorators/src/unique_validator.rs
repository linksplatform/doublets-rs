use std::{borrow::BorrowMut, default::default, marker::PhantomData, ops::Try};

use doublets::{
    data::{LinksConstants, ToQuery},
    num::LinkType,
};

use doublets::{Doublet, Doublets, Error, Link};

pub struct UniqueValidator<L: Doublets> {
    links: L,

    }

impl<L: Doublets> UniqueValidator<L> {
    pub fn new(links: L) -> Self {
        Self {
            links,
            _phantom: default(),
        }
    }
}

impl<L: Doublets> Doublets for UniqueValidator<L> {
    fn constants(&self) -> &LinksConstants<Self::Item> {
        self.links.constants()
    }

    fn count_by(&self, query: impl ToQuery<Self::Item>) -> Self::Item {
        self.links.count_by(query)
    }

    fn create_by_with<F, R>(&mut self, query: impl ToQuery<Self::Item>, handler: F) -> Result<R, Error<Self::Item>>
    where
        F: FnMut(Link<Self::Item>, Link<Self::Item>) -> R,
        R: Try<Output = ()>,
    {
        self.links.create_by_with(query, handler)
    }

    fn each_by<F, R>(&self, restrictions: impl ToQuery<Self::Item>, handler: F) -> R
    where
        F: FnMut(Link<Self::Item>) -> R,
        R: Try<Output = ()>,
    {
        self.links.each_by(restrictions, handler)
    }

    fn update_by_with<F, R>(
        &mut self,
        query: impl ToQuery<Self::Item>,
        change: impl ToQuery<Self::Item>,
        handler: F,
    ) -> Result<R, Error<Self::Item>>
    where
        F: FnMut(Link<Self::Item>, Link<Self::Item>) -> R,
        R: Try<Output = ()>,
    {
        let links = self.links.borrow_mut();
        let any = links.constants().any;
        let change = change.to_query();
        if links.found([any, change[1], change[2]]) {
            Err(Error::AlreadyExists(Doublet::new(change[1], change[2])))
        } else {
            links.update_by_with(query, change, handler)
        }
    }

    fn delete_by_with<F, R>(&mut self, query: impl ToQuery<Self::Item>, handler: F) -> Result<R, Error<Self::Item>>
    where
        F: FnMut(Link<Self::Item>, Link<Self::Item>) -> R,
        R: Try<Output = ()>,
    {
        self.links.delete_by_with(query, handler)
    }
}
