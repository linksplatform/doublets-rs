use bumpalo::Bump;
#[cfg(feature = "rayon")]
use rayon::prelude::*;
use std::{
    default::default,
    ops::{ControlFlow, Try},
};

use crate::{Error, Fuse, Link};
use data::{Flow, LinkType, LinksConstants, ToQuery};

pub type ReadHandler<'a, T> = &'a mut dyn FnMut(Link<Self::Item>) -> Flow;

pub type WriteHandler<'a, T> = &'a mut dyn FnMut(Link<Self::Item>, Link<Self::Item>) -> Flow;

pub trait Links: Send + Sync {
    type Item: LinkType;
    
    fn constants(&self) -> &LinksConstants<Self::Item>;

    fn count_links(&self, query: &[Self::Item]) -> Self::Item;

    fn create_links(&mut self, query: &[Self::Item], handler: WriteHandler<'_, Self::Item>)
    -> Result<Flow, Error<Self::Item>>;

    fn each_links(&self, query: &[Self::Item], handler: ReadHandler<'_, Self::Item>) -> Flow;

    fn update_links(
        &mut self,
        query: &[Self::Item],
        change: &[Self::Item],
        handler: WriteHandler<'_, Self::Item>,
    ) -> Result<Flow, Error<Self::Item>>;

    fn delete_links(&mut self, query: &[Self::Item], handler: WriteHandler<'_, Self::Item>)
    -> Result<Flow, Error<Self::Item>>;
}

pub trait Doublets: Links {
    fn count_by(&self, query: impl ToQuery<Self::Item>) -> Self::Item
    where
        Self: Sized,
    {
        self.count_links(&query.to_query()[..])
    }

    fn count(&self) -> Self::Item
    where
        Self: Sized,
    {
        self.count_by([])
    }

    fn create_by_with<F, R>(
        &mut self,
        query: impl ToQuery<Self::Item>,
        mut handler: F,
    ) -> Result<R, Error<Self::Item>>
    where
        F: FnMut(Link<Self::Item>, Link<Self::Item>) -> R,
        R: Try<Output = ()>,
        Self: Sized,
    {
        let mut output = R::from_output(());
        let query = query.to_query();

        self.create_links(
            &query[..],
            &mut |before, after| match handler(before, after).branch() {
                ControlFlow::Continue(_) => Flow::Continue,
                ControlFlow::Break(residual) => {
                    output = R::from_residual(residual);
                    Flow::Break
                }
            },
        )
        .map(|_| output)
    }

    fn create_by(&mut self, query: impl ToQuery<Self::Item>) -> Result<Self::Item, Error<Self::Item>>
    where
        Self: Sized,
    {
        let mut index = default();
        self.create_by_with(query, |_before, link| {
            index = link.index;
            Flow::Continue
        })
        .map(|_| index)
    }

    fn create_with<F, R>(&mut self, handler: F) -> Result<R, Error<Self::Item>>
    where
        F: FnMut(Link<Self::Item>, Link<Self::Item>) -> R,
        R: Try<Output = ()>,
        Self: Sized,
    {
        self.create_by_with([], handler)
    }

    fn create(&mut self) -> Result<Self::Item, Error<Self::Item>>
    where
        Self: Sized,
    {
        self.create_by([])
    }

    fn each_by<F, R>(&self, query: impl ToQuery<Self::Item>, mut handler: F) -> R
    where
        F: FnMut(Link<Self::Item>) -> R,
        R: Try<Output = ()>,
        Self: Sized,
    {
        let mut output = R::from_output(());
        let query = query.to_query();

        self.each_links(&query[..], &mut |link| match handler(link).branch() {
            ControlFlow::Continue(_) => Flow::Continue,
            ControlFlow::Break(residual) => {
                output = R::from_residual(residual);
                Flow::Break
            }
        });

        output
    }

    fn each<F, R>(&self, handler: F) -> R
    where
        F: FnMut(Link<Self::Item>) -> R,
        R: Try<Output = ()>,
        Self: Sized,
    {
        self.each_by([], handler)
    }

    fn update_by_with<H, R>(
        &mut self,
        query: impl ToQuery<Self::Item>,
        change: impl ToQuery<Self::Item>,
        mut handler: H,
    ) -> Result<R, Error<Self::Item>>
    where
        H: FnMut(Link<Self::Item>, Link<Self::Item>) -> R,
        R: Try<Output = ()>,
        Self: Sized,
    {
        let mut output = R::from_output(());
        let query = query.to_query();
        let change = change.to_query();

        self.update_links(
            &query[..],
            &change[..],
            &mut |before, after| match handler(before, after).branch() {
                ControlFlow::Continue(_) => Flow::Continue,
                ControlFlow::Break(residual) => {
                    output = R::from_residual(residual);
                    Flow::Break
                }
            },
        )
        .map(|_| output)
    }

    fn update_by(&mut self, query: impl ToQuery<Self::Item>, change: impl ToQuery<Self::Item>) -> Result<Self::Item, Error<Self::Item>>
    where
        Self: Sized,
    {
        let mut result = default();
        self.update_by_with(query, change, |_, after| {
            result = after.index;
            Flow::Continue
        })
        .map(|_| result)
    }

    fn update_with<F, R>(
        &mut self,
        index: Self::Item,
        source: Self::Item,
        target: Self::Item,
        handler: F,
    ) -> Result<R, Error<Self::Item>>
    where
        F: FnMut(Link<Self::Item>, Link<Self::Item>) -> R,
        R: Try<Output = ()>,
        Self: Sized,
    {
        self.update_by_with([index], [index, source, target], handler)
    }

    fn update(&mut self, index: Self::Item, source: Self::Item, target: Self::Item) -> Result<Self::Item, Error<Self::Item>>
    where
        Self: Sized,
    {
        self.update_by([index], [index, source, target])
    }

    fn delete_by_with<F, R>(
        &mut self,
        query: impl ToQuery<Self::Item>,
        mut handler: F,
    ) -> Result<R, Error<Self::Item>>
    where
        F: FnMut(Link<Self::Item>, Link<Self::Item>) -> R,
        R: Try<Output = ()>,
        Self: Sized,
    {
        let mut output = R::from_output(());
        let query = query.to_query();

        self.delete_links(
            &query[..],
            &mut |before, after| match handler(before, after).branch() {
                ControlFlow::Continue(_) => Flow::Continue,
                ControlFlow::Break(residual) => {
                    output = R::from_residual(residual);
                    Flow::Break
                }
            },
        )
        .map(|_| output)
    }

    fn delete_by(&mut self, query: impl ToQuery<Self::Item>) -> Result<Self::Item, Error<Self::Item>>
    where
        Self: Sized,
    {
        let mut result = default();
        self.delete_by_with(query, |_before, after| {
            result = after.index;
            Flow::Continue
        })
        .map(|_| result)
    }

    fn delete_with<F, R>(&mut self, index: Self::Item, handler: F) -> Result<R, Error<Self::Item>>
    where
        F: FnMut(Link<Self::Item>, Link<Self::Item>) -> R,
        R: Try<Output = ()>,
        Self: Sized,
    {
        self.delete_by_with([index], handler)
    }

    fn delete(&mut self, index: Self::Item) -> Result<Self::Item, Error<Self::Item>>
    where
        Self: Sized,
    {
        self.delete_by([index])
    }

    fn try_get_link(&self, index: Self::Item) -> Result<Link<Self::Item>, Error<Self::Item>> {
        self.get_link(index).ok_or(Error::NotExists(index))
    }

    fn get_link(&self, index: Self::Item) -> Option<Link<Self::Item>>;

    fn delete_all(&mut self) -> Result<(), Error<Self::Item>>
    where
        Self: Sized,
    {
        // delete all links while self.count() != Self::Item::funty(0)
        let mut count = self.count();
        while count != Self::Item::funty(0) {
            self.delete(count)?;
            count = self.count();
        }
        Ok(())
    }

    fn delete_query_with<F, R>(
        &mut self,
        query: impl ToQuery<Self::Item>,
        handler: F,
    ) -> Result<(), Error<Self::Item>>
    where
        F: FnMut(Link<Self::Item>, Link<Self::Item>) -> R,
        R: Try<Output = ()>,
        Self: Sized,
    {
        let query = query.to_query();
        let len = self.count_by(query.to_query()).as_usize();
        let mut vec = Vec::with_capacity(len);

        self.each_by(query, |link| {
            vec.push(link.index);
            Flow::Continue
        });

        let mut handler = Fuse::new(handler);
        for index in vec.into_iter().rev() {
            self.delete_with(index, &mut handler)?;
        }
        Ok(())
    }

    fn delete_usages_with<F, R>(&mut self, index: Self::Item, handler: F) -> Result<(), Error<Self::Item>>
    where
        F: FnMut(Link<Self::Item>, Link<Self::Item>) -> R,
        R: Try<Output = ()>,
        Self: Sized,
    {
        let any = self.constants().any;
        let mut to_delete = Vec::with_capacity(
            self.count_by([any, index, any]).as_usize()
                + self.count_by([any, any, index]).as_usize(),
        );
        self.each_by([any, index, any], |link| {
            if link.index != index {
                to_delete.push(link.index);
            }
            Flow::Continue
        });

        self.each_by([any, any, index], |link| {
            if link.index != index {
                to_delete.push(link.index);
            }
            Flow::Continue
        });

        let mut handler = Fuse::new(handler);
        for index in to_delete.into_iter().rev() {
            self.delete_with(index, &mut handler)?;
        }
        Ok(())
    }

    fn delete_usages(&mut self, index: Self::Item) -> Result<(), Error<Self::Item>>
    where
        Self: Sized,
    {
        self.delete_usages_with(index, |_, _| Flow::Continue)
    }

    fn create_point(&mut self) -> Result<Self::Item, Error<Self::Item>>
    where
        Self: Sized,
    {
        let new = self.create()?;
        self.update(new, new, new)
    }

    fn create_link_with<F, R>(&mut self, source: Self::Item, target: Self::Item, handler: F) -> Result<Flow, Error<Self::Item>>
    where
        F: FnMut(Link<Self::Item>, Link<Self::Item>) -> R,
        R: Try<Output = ()>,
        Self: Sized,
    {
        let mut new = default();
        let mut handler = Fuse::new(handler);
        self.create_with(|before, after| {
            new = after.index;
            handler(before, after);
            Flow::Continue
        })?;

        self.update_with(new, source, target, handler)
    }

    fn create_link(&mut self, source: Self::Item, target: Self::Item) -> Result<Self::Item, Error<Self::Item>>
    where
        Self: Sized,
    {
        let mut result = default();
        self.create_link_with(source, target, |_, link| {
            result = link.index;
            Flow::Continue
        })
        .map(|_| result)
    }

    fn found(&self, query: impl ToQuery<Self::Item>) -> bool
    where
        Self: Sized,
    {
        self.count_by(query) != Self::Item::funty(0)
    }

    fn find(&self, query: impl ToQuery<Self::Item>) -> Option<Link<Self::Item>>
    where
        Self: Sized,
    {
        let mut result = None;
        self.each_by(query, |link| {
            result = Some(link);
            Flow::Break
        });
        result
    }

    fn search(&self, source: Self::Item, target: Self::Item) -> Option<T>
    where
        Self: Sized,
    {
        self.find([self.constants().any, source, target])
            .map(|link| link.index)
    }

    #[deprecated(note = "use `search` instead")]
    fn search_or(&self, source: Self::Item, target: Self::Item, default: Self::Item) -> Self::Item
    where
        Self: Sized,
    {
        self.search(source, target).unwrap_or(default)
    }

    fn single(&self, query: impl ToQuery<Self::Item>) -> Option<Link<Self::Item>>
    where
        Self: Sized,
    {
        let mut result = None;
        self.each_by(query, |link| {
            if result.is_none() {
                result = Some(link);
                Flow::Continue
            } else {
                result = None;
                Flow::Break
            }
        });
        result
    }

    fn get_or_create(&mut self, source: Self::Item, target: Self::Item) -> Result<Self::Item, Error<Self::Item>>
    where
        Self: Sized,
    {
        if let Some(link) = self.search(source, target) {
            Ok(link)
        } else {
            self.create_link(source, target)
        }
    }

    fn count_usages(&self, index: Self::Item) -> Result<Self::Item, Error<Self::Item>>
    where
        Self: Sized,
    {
        let any = self.constants().any;

        let link = self.try_get_link(index)?;

        let mut usage_source = self.count_by([any, index, any]);
        if index == link.source {
            usage_source -= Self::Item::funty(1);
        }

        let mut usage_target = self.count_by([any, any, index]);
        if index == link.target {
            usage_target -= Self::Item::funty(1);
        }

        Ok(usage_source + usage_target)
    }

    fn usages(&self, index: Self::Item) -> Result<Vec<T>, Error<Self::Item>>
    where
        Self: Sized,
    {
        let any = self.constants().any;
        let mut usages = Vec::with_capacity(self.count_usages(index)?.as_usize());

        self.each_by([any, index, any], |link| {
            if link.index != index {
                usages.push(link.index);
            }
            Flow::Continue
        });

        self.each_by([any, any, index], |link| {
            if link.index != index {
                usages.push(link.index);
            }
            Flow::Continue
        });
        Ok(usages)
    }

    fn exist(&self, link: Self::Item) -> bool
    where
        Self: Sized,
    {
        let constants = self.constants();
        if constants.is_external(link) {
            true
        } else {
            constants.is_internal(link) && self.count_by([link]) != Self::Item::funty(0)
        }
    }

    fn has_usages(&self, link: Self::Item) -> bool
    where
        Self: Sized,
    {
        self.count_usages(link)
            .map_or(false, |link| link != Self::Item::funty(0))
    }

    fn rebase_with<F, R>(&mut self, old: Self::Item, new: Self::Item, handler: F) -> Result<(), Error<Self::Item>>
    where
        F: FnMut(Link<Self::Item>, Link<Self::Item>) -> R,
        R: Try<Output = ()>,
        Self: Sized,
    {
        // guard
        let _ = self.try_get_link(old)?;

        if old == new {
            return Ok(());
        }

        let any = self.constants().any;

        let mut handler = Fuse::new(handler);

        None.into_iter()
            // best readability
            .chain(self.each_iter([any, old, any]))
            .chain(self.each_iter([any, any, old]))
            .filter(|usage| usage.index != old)
            .try_for_each(|usage| {
                if usage.source == old {
                    self.update_with(usage.index, new, usage.target, &mut handler)?;
                }
                if usage.target == old {
                    self.update_with(usage.index, usage.source, new, &mut handler)?;
                }
                Ok(())
            })
    }

    fn rebase(&mut self, old: Self::Item, new: Self::Item) -> Result<Self::Item, Error<Self::Item>>
    where
        Self: Sized,
    {
        self.rebase_with(old, new, |_, _| Flow::Continue)
            .map(|_| new)
    }

    fn rebase_and_delete(&mut self, old: Self::Item, new: Self::Item) -> Result<Self::Item, Error<Self::Item>>
    where
        Self: Sized,
    {
        if old == new {
            Ok(new)
        } else {
            self.rebase(old, new)?;
            self.delete(old)
        }
    }
}

impl<All: Doublets + ?Sized> Links for Box<All> {
    type Item = All::Item;
    
    fn constants(&self) -> &LinksConstants<Self::Item> {
        (**self).constants()
    }

    fn count_links(&self, query: &[Self::Item]) -> Self::Item {
        (**self).count_links(query)
    }

    fn create_links(
        &mut self,
        query: &[Self::Item],
        handler: WriteHandler<'_, Self::Item>,
    ) -> Result<Flow, Error<Self::Item>> {
        (**self).create_links(query, handler)
    }

    fn each_links(&self, query: &[Self::Item], handler: ReadHandler<'_, Self::Item>) -> Flow {
        (**self).each_links(query, handler)
    }

    fn update_links(
        &mut self,
        query: &[Self::Item],
        change: &[Self::Item],
        handler: WriteHandler<'_, Self::Item>,
    ) -> Result<Flow, Error<Self::Item>> {
        (**self).update_links(query, change, handler)
    }

    fn delete_links(
        &mut self,
        query: &[Self::Item],
        handler: WriteHandler<'_, Self::Item>,
    ) -> Result<Flow, Error<Self::Item>> {
        (**self).delete_links(query, handler)
    }
}

impl<All: Doublets + ?Sized> Doublets for Box<All> {
    fn get_link(&self, index: Self::Item) -> Option<Link<Self::Item>> {
        (**self).get_link(index)
    }
}

pub trait DoubletsExt: Sized + Doublets {
    #[cfg(feature = "rayon")]
    type IdxParIter: IndexedParallelIterator<Item = Link<Self::Item>>;

    #[cfg(feature = "rayon")]
    fn par_iter(&self) -> Self::IdxParIter;

    #[cfg(feature = "rayon")]
    fn par_each_iter(&self, query: impl ToQuery<Self::Item>) -> Self::IdxParIter;

    // Box<dyn Iterator<Item = T>> must used while `-> impl Trait` is not stabilized
    // Box<dyn> than easier `Self::ImplIterator1,2,...`
    // and have same performance if has only one possible dyn variant

    type ImplIter: Iterator<Item = Link<Self::Item>>;
    fn iter(&self) -> Self::ImplIter;

    type ImplIterEach: Iterator<Item = Link<Self::Item>>;
    fn each_iter(&self, query: impl ToQuery<Self::Item>) -> Self::ImplIterEach;

    #[cfg(feature = "small-search")]
    type ImplIterSmall: Iterator<Item = Link<Self::Item>>;
    #[cfg(feature = "small-search")]
    fn iter_small(&self) -> Self::ImplIterSmall;

    #[cfg(feature = "small-search")]
    type ImplIterEachSmall: Iterator<Item = Link<Self::Item>>;
    #[cfg(feature = "small-search")]
    fn each_iter_small(&self, query: impl ToQuery<Self::Item>) -> Self::ImplIterEachSmall;
}

impl<All: Doublets + Sized> DoubletsExt for All {
    #[cfg(feature = "rayon")]
    type IdxParIter = impl IndexedParallelIterator<Item = Link<Self::Item>>;

    #[cfg(feature = "rayon")]
    fn par_iter(&self) -> Self::IdxParIter {
        self.par_each_iter([self.constants().any; 3])
    }

    #[cfg(feature = "rayon")]
    fn par_each_iter(&self, query: impl ToQuery<Self::Item>) -> Self::IdxParIter {
        let mut vec = Vec::with_capacity(self.count_by(query.to_query()).as_usize());
        self.each_by(query, |link| {
            vec.push(link);
            Flow::Continue
        });
        vec.into_par_iter()
    }

    type ImplIter = Self::ImplIterEach;

    #[inline]
    fn iter(&self) -> Self::ImplIter {
        self.each_iter([self.constants().any; 3])
    }

    type ImplIterEach = impl Iterator<Item = Link<Self::Item>> + ExactSizeIterator + DoubleEndedIterator;

    #[cfg_attr(feature = "more-inline", inline)]
    fn each_iter(&self, query: impl ToQuery<Self::Item>) -> Self::ImplIterEach {
        let cap = self.count_by(query.to_query()).as_usize();

        let mut vec = Vec::with_capacity(cap);
        self.each_by(query, &mut |link| {
            vec.push(link);
            Flow::Continue
        });
        vec.into_iter()
    }

    #[cfg(feature = "small-search")]
    type ImplIterSmall = Self::ImplIterEachSmall;

    #[inline]
    #[cfg(feature = "small-search")]
    fn iter_small(&self) -> Self::ImplIterSmall {
        self.each_iter_small([self.constants().any; 3])
    }

    #[cfg(feature = "small-search")]
    type ImplIterEachSmall =
        impl Iterator<Item = Link<Self::Item>> + ExactSizeIterator + DoubleEndedIterator;

    #[cfg(feature = "small-search")]
    #[cfg_attr(feature = "more-inline", inline)]
    fn each_iter_small(&self, query: impl ToQuery<Self::Item>) -> Self::ImplIterEachSmall {
        // fixme: later use const generics
        const SIZE_HINT: usize = 2;

        let mut vec = smallvec::SmallVec::<[Link<_>; SIZE_HINT]>::with_capacity(
            self.count_by(query.to_query()).as_usize(),
        );
        self.each_by(query, |link| {
            vec.push(link);
            Flow::Continue
        });
        vec.into_iter()
    }
}
