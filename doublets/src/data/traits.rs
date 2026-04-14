#[cfg(feature = "rayon")]
use rayon::prelude::*;

use crate::{Error, Fuse, Link};
use data::{Flow, LinkType, LinksConstants, ToQuery};

pub type ReadHandler<'a, T> = &'a mut dyn FnMut(Link<T>) -> Flow;

pub type WriteHandler<'a, T> = &'a mut dyn FnMut(Link<T>, Link<T>) -> Flow;

pub trait Links<T: LinkType>: Send + Sync {
    fn constants(&self) -> &LinksConstants<T>;

    fn count_links(&self, query: &[T]) -> T;

    fn create_links(&mut self, query: &[T], handler: WriteHandler<'_, T>)
    -> Result<Flow, Error<T>>;

    fn each_links(&self, query: &[T], handler: ReadHandler<'_, T>) -> Flow;

    fn update_links(
        &mut self,
        query: &[T],
        change: &[T],
        handler: WriteHandler<'_, T>,
    ) -> Result<Flow, Error<T>>;

    fn delete_links(&mut self, query: &[T], handler: WriteHandler<'_, T>)
    -> Result<Flow, Error<T>>;
}

pub trait Doublets<T: LinkType>: Links<T> {
    fn count_by(&self, query: impl ToQuery<T>) -> T
    where
        Self: Sized,
    {
        self.count_links(&query.to_query()[..])
    }

    fn count(&self) -> T
    where
        Self: Sized,
    {
        self.count_by([])
    }

    fn create_by_with<F>(
        &mut self,
        query: impl ToQuery<T>,
        mut handler: F,
    ) -> Result<Flow, Error<T>>
    where
        F: FnMut(Link<T>, Link<T>) -> Flow,
        Self: Sized,
    {
        let query = query.to_query();
        self.create_links(&query[..], &mut handler)
    }

    fn create_by(&mut self, query: impl ToQuery<T>) -> Result<T, Error<T>>
    where
        Self: Sized,
    {
        let mut index = T::funty(0);
        self.create_by_with(query, |_before, link| {
            index = link.index;
            Flow::Continue
        })
        .map(|_| index)
    }

    fn create_with<F>(&mut self, handler: F) -> Result<Flow, Error<T>>
    where
        F: FnMut(Link<T>, Link<T>) -> Flow,
        Self: Sized,
    {
        self.create_by_with([], handler)
    }

    fn create(&mut self) -> Result<T, Error<T>>
    where
        Self: Sized,
    {
        self.create_by([])
    }

    fn each_by<F>(&self, query: impl ToQuery<T>, mut handler: F) -> Flow
    where
        F: FnMut(Link<T>) -> Flow,
        Self: Sized,
    {
        let query = query.to_query();
        self.each_links(&query[..], &mut handler)
    }

    fn each<F>(&self, handler: F) -> Flow
    where
        F: FnMut(Link<T>) -> Flow,
        Self: Sized,
    {
        self.each_by([], handler)
    }

    fn update_by_with<H>(
        &mut self,
        query: impl ToQuery<T>,
        change: impl ToQuery<T>,
        mut handler: H,
    ) -> Result<Flow, Error<T>>
    where
        H: FnMut(Link<T>, Link<T>) -> Flow,
        Self: Sized,
    {
        let query = query.to_query();
        let change = change.to_query();
        self.update_links(&query[..], &change[..], &mut handler)
    }

    fn update_by(&mut self, query: impl ToQuery<T>, change: impl ToQuery<T>) -> Result<T, Error<T>>
    where
        Self: Sized,
    {
        let mut result = T::funty(0);
        self.update_by_with(query, change, |_, after| {
            result = after.index;
            Flow::Continue
        })
        .map(|_| result)
    }

    fn update_with<F>(
        &mut self,
        index: T,
        source: T,
        target: T,
        handler: F,
    ) -> Result<Flow, Error<T>>
    where
        F: FnMut(Link<T>, Link<T>) -> Flow,
        Self: Sized,
    {
        self.update_by_with([index], [index, source, target], handler)
    }

    fn update(&mut self, index: T, source: T, target: T) -> Result<T, Error<T>>
    where
        Self: Sized,
    {
        self.update_by([index], [index, source, target])
    }

    fn delete_by_with<F>(
        &mut self,
        query: impl ToQuery<T>,
        mut handler: F,
    ) -> Result<Flow, Error<T>>
    where
        F: FnMut(Link<T>, Link<T>) -> Flow,
        Self: Sized,
    {
        let query = query.to_query();
        self.delete_links(&query[..], &mut handler)
    }

    fn delete_by(&mut self, query: impl ToQuery<T>) -> Result<T, Error<T>>
    where
        Self: Sized,
    {
        let mut result = T::funty(0);
        self.delete_by_with(query, |_before, after| {
            result = after.index;
            Flow::Continue
        })
        .map(|_| result)
    }

    fn delete_with<F>(&mut self, index: T, handler: F) -> Result<Flow, Error<T>>
    where
        F: FnMut(Link<T>, Link<T>) -> Flow,
        Self: Sized,
    {
        self.delete_by_with([index], handler)
    }

    fn delete(&mut self, index: T) -> Result<T, Error<T>>
    where
        Self: Sized,
    {
        self.delete_by([index])
    }

    fn try_get_link(&self, index: T) -> Result<Link<T>, Error<T>> {
        self.get_link(index).ok_or(Error::NotExists(index))
    }

    fn get_link(&self, index: T) -> Option<Link<T>>;

    fn delete_all(&mut self) -> Result<(), Error<T>>
    where
        Self: Sized,
    {
        let mut count = self.count();
        while count != T::funty(0) {
            self.delete(count)?;
            count = self.count();
        }
        Ok(())
    }

    fn delete_query_with<F>(
        &mut self,
        query: impl ToQuery<T>,
        handler: F,
    ) -> Result<(), Error<T>>
    where
        F: FnMut(Link<T>, Link<T>) -> Flow,
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
            self.delete_links(&[index], &mut |before, after| handler.call(before, after))?;
        }
        Ok(())
    }

    fn delete_usages_with<F>(&mut self, index: T, handler: F) -> Result<(), Error<T>>
    where
        F: FnMut(Link<T>, Link<T>) -> Flow,
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
            self.delete_links(&[index], &mut |before, after| handler.call(before, after))?;
        }
        Ok(())
    }

    fn delete_usages(&mut self, index: T) -> Result<(), Error<T>>
    where
        Self: Sized,
    {
        self.delete_usages_with(index, |_, _| Flow::Continue)
    }

    fn create_point(&mut self) -> Result<T, Error<T>>
    where
        Self: Sized,
    {
        let new = self.create()?;
        self.update(new, new, new)
    }

    fn create_link_with<F>(&mut self, source: T, target: T, handler: F) -> Result<Flow, Error<T>>
    where
        F: FnMut(Link<T>, Link<T>) -> Flow,
        Self: Sized,
    {
        let mut new = T::funty(0);
        let mut handler = Fuse::new(handler);
        self.create_with(|before, after| {
            new = after.index;
            handler.call(before, after);
            Flow::Continue
        })?;

        self.update_links(
            &[new],
            &[new, source, target],
            &mut |before, after| handler.call(before, after),
        )
    }

    fn create_link(&mut self, source: T, target: T) -> Result<T, Error<T>>
    where
        Self: Sized,
    {
        let mut result = T::funty(0);
        self.create_link_with(source, target, |_, link| {
            result = link.index;
            Flow::Continue
        })
        .map(|_| result)
    }

    fn found(&self, query: impl ToQuery<T>) -> bool
    where
        Self: Sized,
    {
        self.count_by(query) != T::funty(0)
    }

    fn find(&self, query: impl ToQuery<T>) -> Option<Link<T>>
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

    fn search(&self, source: T, target: T) -> Option<T>
    where
        Self: Sized,
    {
        self.find([self.constants().any, source, target])
            .map(|link| link.index)
    }

    #[deprecated(note = "use `search` instead")]
    fn search_or(&self, source: T, target: T, default: T) -> T
    where
        Self: Sized,
    {
        self.search(source, target).unwrap_or(default)
    }

    fn single(&self, query: impl ToQuery<T>) -> Option<Link<T>>
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

    fn get_or_create(&mut self, source: T, target: T) -> Result<T, Error<T>>
    where
        Self: Sized,
    {
        if let Some(link) = self.search(source, target) {
            Ok(link)
        } else {
            self.create_link(source, target)
        }
    }

    fn count_usages(&self, index: T) -> Result<T, Error<T>>
    where
        Self: Sized,
    {
        let any = self.constants().any;

        let link = self.try_get_link(index)?;

        let mut usage_source = self.count_by([any, index, any]);
        if index == link.source {
            usage_source -= T::funty(1);
        }

        let mut usage_target = self.count_by([any, any, index]);
        if index == link.target {
            usage_target -= T::funty(1);
        }

        Ok(usage_source + usage_target)
    }

    fn usages(&self, index: T) -> Result<Vec<T>, Error<T>>
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

    fn exist(&self, link: T) -> bool
    where
        Self: Sized,
    {
        let constants = self.constants();
        if constants.is_external(link) {
            true
        } else {
            constants.is_internal(link) && self.count_by([link]) != T::funty(0)
        }
    }

    fn has_usages(&self, link: T) -> bool
    where
        Self: Sized,
    {
        self.count_usages(link)
            .map_or(false, |link| link != T::funty(0))
    }

    fn rebase_with<F>(&mut self, old: T, new: T, handler: F) -> Result<(), Error<T>>
    where
        F: FnMut(Link<T>, Link<T>) -> Flow,
        Self: Sized,
    {
        let _ = self.try_get_link(old)?;

        if old == new {
            return Ok(());
        }

        let any = self.constants().any;

        let mut handler = Fuse::new(handler);

        let usages: Vec<_> = None
            .into_iter()
            .chain(self.each_iter([any, old, any]))
            .chain(self.each_iter([any, any, old]))
            .filter(|usage| usage.index != old)
            .collect();
        for usage in usages {
            if usage.source == old {
                self.update_links(
                    &[usage.index],
                    &[usage.index, new, usage.target],
                    &mut |before, after| handler.call(before, after),
                )?;
            }
            if usage.target == old {
                self.update_links(
                    &[usage.index],
                    &[usage.index, usage.source, new],
                    &mut |before, after| handler.call(before, after),
                )?;
            }
        }
        Ok(())
    }

    fn rebase(&mut self, old: T, new: T) -> Result<T, Error<T>>
    where
        Self: Sized,
    {
        self.rebase_with(old, new, |_, _| Flow::Continue)
            .map(|_| new)
    }

    fn rebase_and_delete(&mut self, old: T, new: T) -> Result<T, Error<T>>
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

impl<T: LinkType, All: Doublets<T> + ?Sized> Links<T> for Box<All> {
    fn constants(&self) -> &LinksConstants<T> {
        (**self).constants()
    }

    fn count_links(&self, query: &[T]) -> T {
        (**self).count_links(query)
    }

    fn create_links(
        &mut self,
        query: &[T],
        handler: WriteHandler<'_, T>,
    ) -> Result<Flow, Error<T>> {
        (**self).create_links(query, handler)
    }

    fn each_links(&self, query: &[T], handler: ReadHandler<'_, T>) -> Flow {
        (**self).each_links(query, handler)
    }

    fn update_links(
        &mut self,
        query: &[T],
        change: &[T],
        handler: WriteHandler<'_, T>,
    ) -> Result<Flow, Error<T>> {
        (**self).update_links(query, change, handler)
    }

    fn delete_links(
        &mut self,
        query: &[T],
        handler: WriteHandler<'_, T>,
    ) -> Result<Flow, Error<T>> {
        (**self).delete_links(query, handler)
    }
}

impl<T: LinkType, All: Doublets<T> + ?Sized> Doublets<T> for Box<All> {
    fn get_link(&self, index: T) -> Option<Link<T>> {
        (**self).get_link(index)
    }
}

pub trait DoubletsExt<T: LinkType>: Sized + Doublets<T> {
    #[cfg(feature = "rayon")]
    type IdxParIter: IndexedParallelIterator<Item = Link<T>>;

    #[cfg(feature = "rayon")]
    fn par_iter(&self) -> Self::IdxParIter;

    #[cfg(feature = "rayon")]
    fn par_each_iter(&self, query: impl ToQuery<T>) -> Self::IdxParIter;

    fn iter(&self) -> impl Iterator<Item = Link<T>> + ExactSizeIterator + DoubleEndedIterator;

    fn each_iter(
        &self,
        query: impl ToQuery<T>,
    ) -> impl Iterator<Item = Link<T>> + ExactSizeIterator + DoubleEndedIterator;

    #[cfg(feature = "small-search")]
    fn iter_small(&self) -> impl Iterator<Item = Link<T>> + ExactSizeIterator + DoubleEndedIterator;

    #[cfg(feature = "small-search")]
    fn each_iter_small(
        &self,
        query: impl ToQuery<T>,
    ) -> impl Iterator<Item = Link<T>> + ExactSizeIterator + DoubleEndedIterator;
}

impl<T: LinkType, All: Doublets<T> + Sized> DoubletsExt<T> for All {
    #[cfg(feature = "rayon")]
    type IdxParIter = rayon::vec::IntoIter<Link<T>>;

    #[cfg(feature = "rayon")]
    fn par_iter(&self) -> Self::IdxParIter {
        self.par_each_iter([self.constants().any; 3])
    }

    #[cfg(feature = "rayon")]
    fn par_each_iter(&self, query: impl ToQuery<T>) -> Self::IdxParIter {
        let mut vec = Vec::with_capacity(self.count_by(query.to_query()).as_usize());
        self.each_by(query, |link| {
            vec.push(link);
            Flow::Continue
        });
        vec.into_par_iter()
    }

    #[inline]
    fn iter(&self) -> impl Iterator<Item = Link<T>> + ExactSizeIterator + DoubleEndedIterator {
        self.each_iter([self.constants().any; 3])
    }

    #[cfg_attr(feature = "more-inline", inline)]
    fn each_iter(
        &self,
        query: impl ToQuery<T>,
    ) -> impl Iterator<Item = Link<T>> + ExactSizeIterator + DoubleEndedIterator {
        let cap = self.count_by(query.to_query()).as_usize();

        let mut vec = Vec::with_capacity(cap);
        self.each_by(query, &mut |link| {
            vec.push(link);
            Flow::Continue
        });
        vec.into_iter()
    }

    #[inline]
    #[cfg(feature = "small-search")]
    fn iter_small(&self) -> impl Iterator<Item = Link<T>> + ExactSizeIterator + DoubleEndedIterator {
        self.each_iter_small([self.constants().any; 3])
    }

    #[cfg(feature = "small-search")]
    #[cfg_attr(feature = "more-inline", inline)]
    fn each_iter_small(
        &self,
        query: impl ToQuery<T>,
    ) -> impl Iterator<Item = Link<T>> + ExactSizeIterator + DoubleEndedIterator {
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
