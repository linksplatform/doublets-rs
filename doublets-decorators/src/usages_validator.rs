use std::{borrow::BorrowMut, default::default, marker::PhantomData, ops::Try};

use doublets::{
    data::{LinksConstants, ToQuery},
    num::LinkType,
};

use doublets::{Doublets, Error, Link};

pub struct UsagesValidator<L: Doublets> {
    links: L,

    }

impl<L: Doublets> UsagesValidator<L> {
    pub fn new(links: L) -> Self {
        Self {
            links,
            _phantom: default(),
        }
    }
}

impl<L: Doublets> Doublets for UsagesValidator<L> {
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
        let query = query.to_query();
        let index = query[links.constants().index_part.as_usize()];
        let usages = links.usages(index)?;
        if usages.is_empty() {
            links.update_by_with(query, change, handler)
        } else {
            let usages: Result<_, Error<T>> = usages
                .into_iter()
                .map(|index| links.try_get_link(index))
                .collect();
            Err(Error::HasUsages(usages?))
        }
    }

    fn delete_by_with<F, R>(&mut self, query: impl ToQuery<Self::Item>, handler: F) -> Result<R, Error<Self::Item>>
    where
        F: FnMut(Link<Self::Item>, Link<Self::Item>) -> R,
        R: Try<Output = ()>,
    {
        let links = self.links.borrow_mut();
        let query = query.to_query();
        let index = query[links.constants().index_part.as_usize()];
        let usages = links.usages(index)?;
        if usages.is_empty() {
            links.delete_by_with(query, handler)
        } else {
            let usages: Result<_, Error<T>> = usages
                .into_iter()
                .map(|index| links.try_get_link(index))
                .collect();
            Err(Error::HasUsages(usages?))
        }
    }
}
