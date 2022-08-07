#[cfg(feature = "rayon")]
use rayon::prelude::*;
use std::{
    default::default,
    mem::size_of,
    ops::{ControlFlow, Try},
};

use crate::{FuseHandler, Link, LinksError};
use data::{Flow, LinkType, LinksConstants, ToQuery};

pub type Result<T, E = LinksError<T>> = std::result::Result<T, E>;

pub type ReadHandler<'a, T> = &'a mut dyn FnMut(Link<T>) -> Flow;

pub type WriteHandler<'a, T> = &'a mut dyn FnMut(Link<T>, Link<T>) -> Flow;

pub trait Links<T: LinkType>: Send + Sync {
    fn constants(&self) -> &LinksConstants<T>;

    fn count_links(&self, query: &[T]) -> T;

    fn create_links(
        &mut self,
        query: &[T],
        handler: WriteHandler<T>,
    ) -> Result<Flow, LinksError<T>>;

    fn each_links(&self, query: &[T], handler: ReadHandler<T>) -> Flow;

    fn update_links(
        &mut self,
        query: &[T],
        change: &[T],
        handler: WriteHandler<T>,
    ) -> Result<Flow, LinksError<T>>;

    fn delete_links(
        &mut self,
        query: &[T],
        handler: WriteHandler<T>,
    ) -> Result<Flow, LinksError<T>>;

    fn iter_links(&self) -> Box<dyn Iterator<Item = Link<T>>> {
        self.each_iter_links(&[])
    }

    fn each_iter_links(&self, query: &[T]) -> Box<dyn Iterator<Item = Link<T>>> {
        let capacity = self.count_links(query).as_usize();

        cfg_if::cfg_if! {
            if #[cfg(feature = "smallvec-optimization")] {
                let mut vec = smallvec::SmallVec::<[_; 2]>::with_capacity(capacity);
            } else {
                let mut vec = Vec::with_capacity(capacity);
            }
        }

        self.each_links(query, &mut |link| {
            vec.push(link);
            Flow::Continue
        });

        Box::new(vec.into_iter())
    }
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

    fn create_by_with<F, R>(
        &mut self,
        query: impl ToQuery<T>,
        mut handler: F,
    ) -> Result<R, LinksError<T>>
    where
        F: FnMut(Link<T>, Link<T>) -> R,
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

    fn create_by(&mut self, query: impl ToQuery<T>) -> Result<T, LinksError<T>>
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

    fn create_with<F, R>(&mut self, handler: F) -> Result<R, LinksError<T>>
    where
        F: FnMut(Link<T>, Link<T>) -> R,
        R: Try<Output = ()>,
        Self: Sized,
    {
        self.create_by_with([], handler)
    }

    fn create(&mut self) -> Result<T>
    where
        Self: Sized,
    {
        self.create_by([])
    }

    fn each_by<F, R>(&self, query: impl ToQuery<T>, mut handler: F) -> R
    where
        F: FnMut(Link<T>) -> R,
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
        F: FnMut(Link<T>) -> R,
        R: Try<Output = ()>,
        Self: Sized,
    {
        self.each_by([], handler)
    }

    fn iter(&self) -> Box<dyn Iterator<Item = Link<T>> + 'static> {
        self.iter_links()
    }

    fn each_iter(&self, query: impl ToQuery<T>) -> Box<dyn Iterator<Item = Link<T>> + 'static>
    where
        Self: Sized,
    {
        self.each_iter_links(&query.to_query()[..])
    }

    fn update_by_with<H, R>(
        &mut self,
        query: impl ToQuery<T>,
        change: impl ToQuery<T>,
        mut handler: H,
    ) -> Result<R, LinksError<T>>
    where
        H: FnMut(Link<T>, Link<T>) -> R,
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

    fn update_by(&mut self, query: impl ToQuery<T>, change: impl ToQuery<T>) -> Result<T>
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
        index: T,
        source: T,
        target: T,
        handler: F,
    ) -> Result<R, LinksError<T>>
    where
        F: FnMut(Link<T>, Link<T>) -> R,
        R: Try<Output = ()>,
        Self: Sized,
    {
        self.update_by_with([index], [index, source, target], handler)
    }

    fn update(&mut self, index: T, source: T, target: T) -> Result<T>
    where
        Self: Sized,
    {
        self.update_by([index], [index, source, target])
    }

    fn delete_by_with<F, R>(
        &mut self,
        query: impl ToQuery<T>,
        mut handler: F,
    ) -> Result<R, LinksError<T>>
    where
        F: FnMut(Link<T>, Link<T>) -> R,
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

    fn delete_by(&mut self, query: impl ToQuery<T>) -> Result<T>
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

    fn delete_with<F, R>(&mut self, index: T, handler: F) -> Result<R, LinksError<T>>
    where
        F: FnMut(Link<T>, Link<T>) -> R,
        R: Try<Output = ()>,
        Self: Sized,
    {
        self.delete_by_with([index], handler)
    }

    fn delete(&mut self, index: T) -> Result<T>
    where
        Self: Sized,
    {
        self.delete_by([index])
    }

    fn try_get_link(&self, index: T) -> Result<Link<T>, LinksError<T>> {
        self.get_link(index).ok_or(LinksError::NotExists(index))
    }

    fn get_link(&self, index: T) -> Option<Link<T>>;

    fn delete_all(&mut self) -> Result<(), LinksError<T>>
    where
        Self: Sized,
    {
        // delete all links while self.count() != T::funty(0)
        let mut count = self.count();
        while count != T::funty(0) {
            self.delete(count)?;
            count = self.count();
        }
        Ok(())
    }

    fn delete_query_with<F, R>(
        &mut self,
        query: impl ToQuery<T>,
        handler: F,
    ) -> Result<(), LinksError<T>>
    where
        F: FnMut(Link<T>, Link<T>) -> R,
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

        let mut handler = FuseHandler::new(handler);
        for index in vec.into_iter().rev() {
            self.delete_with(index, &mut handler)?;
        }
        Ok(())
    }

    fn delete_usages_with<F, R>(&mut self, index: T, handler: F) -> Result<(), LinksError<T>>
    where
        F: FnMut(Link<T>, Link<T>) -> R,
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

        let mut handler = FuseHandler::new(handler);
        for index in to_delete.into_iter().rev() {
            self.delete_with(index, &mut handler)?;
        }
        Ok(())
    }

    fn delete_usages(&mut self, index: T) -> Result<(), LinksError<T>>
    where
        Self: Sized,
    {
        self.delete_usages_with(index, |_, _| Flow::Continue)
    }

    fn create_point(&mut self) -> Result<T>
    where
        Self: Sized,
    {
        let new = self.create()?;
        self.update(new, new, new)
    }

    fn create_link_with<F, R>(
        &mut self,
        source: T,
        target: T,
        handler: F,
    ) -> Result<Flow, LinksError<T>>
    where
        F: FnMut(Link<T>, Link<T>) -> R,
        R: Try<Output = ()>,
        Self: Sized,
    {
        let mut new = default();
        let mut handler = FuseHandler::new(handler);
        self.create_with(|before, after| {
            new = after.index;
            handler(before, after);
            Flow::Continue
        })?;

        self.update_with(new, source, target, handler)
    }

    fn create_link(&mut self, source: T, target: T) -> Result<T>
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

    fn get_or_create(&mut self, source: T, target: T) -> Result<T>
    where
        Self: Sized,
    {
        if let Some(link) = self.search(source, target) {
            Ok(link)
        } else {
            self.create_link(source, target)
        }
    }

    fn count_usages(&self, index: T) -> Result<T>
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

    fn usages(&self, index: T) -> Result<Vec<T>, LinksError<T>>
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

    fn rebase_with<F, R>(&mut self, old: T, new: T, handler: F) -> Result<(), LinksError<T>>
    where
        F: FnMut(Link<T>, Link<T>) -> R,
        R: Try<Output = ()>,
        Self: Sized,
    {
        // guard
        let _ = self.try_get_link(old)?;

        if old == new {
            return Ok(());
        }

        let any = self.constants().any;

        let mut handler = FuseHandler::new(handler);

        None.into_iter()
            // best readability
            .chain(self.each_iter([any, old, any]))
            .chain(self.each_iter([any, any, old]))
            .filter(|usage| usage.index != old)
            .try_for_each(|usage| {
                self.update_with(usage.index, usage.source, new, &mut handler)
                    .map(|_| ())
            })
    }

    fn rebase(&mut self, old: T, new: T) -> Result<T>
    where
        Self: Sized,
    {
        self.rebase_with(old, new, |_, _| Flow::Continue)
            .map(|_| new)
    }

    fn rebase_and_delete(&mut self, old: T, new: T) -> Result<T>
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
        handler: WriteHandler<T>,
    ) -> Result<Flow, LinksError<T>> {
        (**self).create_links(query, handler)
    }

    fn each_links(&self, query: &[T], handler: ReadHandler<T>) -> Flow {
        (**self).each_links(query, handler)
    }

    fn update_links(
        &mut self,
        query: &[T],
        change: &[T],
        handler: WriteHandler<T>,
    ) -> Result<Flow, LinksError<T>> {
        (**self).update_links(query, change, handler)
    }

    fn delete_links(
        &mut self,
        query: &[T],
        handler: WriteHandler<T>,
    ) -> Result<Flow, LinksError<T>> {
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
    type IdxParIterEach: IndexedParallelIterator<Item = Link<T>>;

    #[cfg(feature = "rayon")]
    fn par_each_iter(&self, query: impl ToQuery<T>) -> Self::IdxParIterEach;
}

impl<T: LinkType, All: Doublets<T> + Sized> DoubletsExt<T> for All {
    #[cfg(feature = "rayon")]
    type IdxParIterEach = impl IndexedParallelIterator<Item = Link<T>>;

    #[cfg(feature = "rayon")]
    fn par_each_iter(&self, query: impl ToQuery<T>) -> Self::IdxParIterEach {
        let mut vec = Vec::with_capacity(self.count_by(query.to_query()).as_usize());
        self.each_by(query, |link| {
            vec.push(link);
            Flow::Continue
        });
        vec.into_par_iter()
    }
}
