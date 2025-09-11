use std::{borrow::BorrowMut, marker::PhantomData, ops::Try};

use doublets::{
    data::{LinksConstants, ToQuery},
    num::LinkType,
};

use doublets::{Doublets, Error, Link};

pub struct UniqueResolver<L: Doublets> {
    links: L,

    }

impl<L: Doublets> UniqueResolver<L> {
    pub fn new(links: L) -> Self {
        UniqueResolver {
            links,
            }
    }
}

impl<L: Doublets> Links for UniqueResolver<L> {
    type Item = L::Item;
    
    fn constants(&self) -> &LinksConstants<Self::Item> {
        self.links.constants()
    }

    fn count_links(&self, query: &[Self::Item]) -> Self::Item {
        self.links.count_links(query)
    }

    fn create_links(&mut self, query: &[Self::Item], handler: doublets::WriteHandler<'_, Self::Item>) 
        -> Result<doublets::Flow, doublets::Error<Self::Item>> {
        self.links.create_links(query, handler)
    }

    fn each_links(&self, query: &[Self::Item], handler: doublets::ReadHandler<'_, Self::Item>) -> doublets::Flow {
        self.links.each_links(query, handler)
    }

    fn update_links(&mut self, query: &[Self::Item], change: &[Self::Item], handler: doublets::WriteHandler<'_, Self::Item>) 
        -> Result<doublets::Flow, doublets::Error<Self::Item>> {
        self.links.update_links(query, change, handler)
    }

    fn delete_links(&mut self, query: &[Self::Item], handler: doublets::WriteHandler<'_, Self::Item>) 
        -> Result<doublets::Flow, doublets::Error<Self::Item>> {
        self.links.delete_links(query, handler)
    }
}

impl<L: Doublets> Doublets for UniqueResolver<L> {

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
        mut handler: F,
    ) -> Result<R, Error<Self::Item>>
    where
        F: FnMut(Link<Self::Item>, Link<Self::Item>) -> R,
        R: Try<Output = ()>,
    {
        let links = self.links.borrow_mut();
        let query = query.to_query();
        let change = change.to_query();
        let (new, source, target) = (query[0], change[1], change[2]);
        // todo find by [[any], change[1..]].concat()
        let index = if let Some(old) = links.search(source, target) {
            links.delete_with(new, &mut handler)?;
            old
        } else {
            new
        };

        // TODO: update_by maybe has query style
        links.update_with(index, source, target, handler)
    }

    fn delete_by_with<F, R>(&mut self, query: impl ToQuery<Self::Item>, handler: F) -> Result<R, Error<Self::Item>>
    where
        F: FnMut(Link<Self::Item>, Link<Self::Item>) -> R,
        R: Try<Output = ()>,
    {
        self.links.delete_by_with(query, handler)
    }

    fn get_link(&self, index: Self::Item) -> Option<Link<Self::Item>> {
        self.links.get_link(index)
    }
}
