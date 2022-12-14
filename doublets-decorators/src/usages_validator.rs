use std::{borrow::BorrowMut, default::default, marker::PhantomData, ops::Try};

use doublets::{
    data::{LinksConstants, ToQuery},
    num::LinkType,
};

use doublets::{Doublets, Error, Link};

pub struct UsagesValidator<T: LinkType, L: Doublets<T>> {
    links: L,

    _phantom: PhantomData<T>,
}

impl<T: LinkType, L: Doublets<T>> UsagesValidator<T, L> {
    pub fn new(links: L) -> Self {
        Self {
            links,
            _phantom: default(),
        }
    }
}

impl<T: LinkType, L: Doublets<T>> Doublets<T> for UsagesValidator<T, L> {
    fn constants(&self) -> LinksConstants<T> {
        self.links.constants()
    }

    fn count_by(&self, query: impl ToQuery<T>) -> T {
        self.links.count_by(query)
    }

    fn create_by_with<F, R>(&mut self, query: impl ToQuery<T>, handler: F) -> Result<R, Error<T>>
    where
        F: FnMut(Link<T>, Link<T>) -> R,
        R: Try<Output = ()>,
    {
        self.links.create_by_with(query, handler)
    }

    fn each_by<F, R>(&self, restrictions: impl ToQuery<T>, handler: F) -> R
    where
        F: FnMut(Link<T>) -> R,
        R: Try<Output = ()>,
    {
        self.links.each_by(restrictions, handler)
    }

    fn update_by_with<F, R>(
        &mut self,
        query: impl ToQuery<T>,
        change: impl ToQuery<T>,
        handler: F,
    ) -> Result<R, Error<T>>
    where
        F: FnMut(Link<T>, Link<T>) -> R,
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

    fn delete_by_with<F, R>(&mut self, query: impl ToQuery<T>, handler: F) -> Result<R, Error<T>>
    where
        F: FnMut(Link<T>, Link<T>) -> R,
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
