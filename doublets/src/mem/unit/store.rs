use crate::{
    mem::{
        header::LinksHeader,
        traits::UnitList,
        unit::{
            LinkPart, LinksSourcesRecursionlessSizeBalancedTree,
            LinksTargetsRecursionlessSizeBalancedTree, UnusedLinks,
        },
        UnitTree,
    },
    Doublets, Link, Links, LinksError, ReadHandler, WriteHandler,
};
use data::{Flow, LinkType, LinksConstants, ToQuery};
use leak_slice::LeakSliceExt;
use mem::{RawMem, DEFAULT_PAGE_SIZE};

use std::{cmp, cmp::Ordering, error::Error, mem::transmute, ptr::NonNull};

pub struct Store<
    T: LinkType,
    M: RawMem<LinkPart<T>>,
    TS: UnitTree<T> = LinksSourcesRecursionlessSizeBalancedTree<T>,
    TT: UnitTree<T> = LinksTargetsRecursionlessSizeBalancedTree<T>,
    TU: UnitList<T> = UnusedLinks<T>,
> {
    mem: M,
    mem_ptr: NonNull<[LinkPart<T>]>,
    reserve_step: usize,
    constants: LinksConstants<T>,

    sources: TS,
    targets: TT,
    unused: TU,
}

impl<T: LinkType, M: RawMem<LinkPart<T>>, TS: UnitTree<T>, TT: UnitTree<T>, TU: UnitList<T>>
    Store<T, M, TS, TT, TU>
{
    #[cfg(not(miri))]
    const SIZE_STEP: usize = 2_usize.pow(20);
    #[cfg(miri)]
    const SIZE_STEP: usize = 2_usize.pow(10);

    pub fn new(mem: M) -> Result<Store<T, M>, LinksError<T>> {
        Self::with_constants(mem, LinksConstants::new())
    }

    pub fn with_constants(
        mem: M,
        constants: LinksConstants<T>,
    ) -> Result<Store<T, M>, LinksError<T>> {
        let dangling_mem = NonNull::slice_from_raw_parts(NonNull::dangling(), 0);
        let sources =
            LinksSourcesRecursionlessSizeBalancedTree::new(constants.clone(), dangling_mem);
        let targets =
            LinksTargetsRecursionlessSizeBalancedTree::new(constants.clone(), dangling_mem);
        let unused = UnusedLinks::new(dangling_mem);
        let mut new = Store::<
            T,
            M,
            LinksSourcesRecursionlessSizeBalancedTree<T>,
            LinksTargetsRecursionlessSizeBalancedTree<T>,
            UnusedLinks<T>,
        > {
            mem,
            mem_ptr: dangling_mem,
            reserve_step: Self::SIZE_STEP,
            constants,
            sources,
            targets,
            unused,
        };

        // SAFETY: Without this, the code will become unsafe
        unsafe {
            new.init()?;
        }
        Ok(new)
    }

    unsafe fn init(&mut self) -> Result<(), LinksError<T>> {
        let mem = NonNull::from(self.mem.alloc(DEFAULT_PAGE_SIZE)?);
        self.update_mem(mem);

        let header = self.get_header().clone();
        let capacity = cmp::max(self.reserve_step, header.allocated.as_usize());
        let mem = self.mem.alloc(capacity)?.leak();
        self.update_mem(mem);

        let reserved = self.mem.allocated();

        let header = self.mut_header();
        header.reserved = T::try_from(reserved - 1).expect("always ok");
        Ok(())
    }

    fn mut_from_mem<'a, U>(mut ptr: NonNull<[U]>, index: usize) -> Option<&'a mut U> {
        if index < ptr.len() {
            // SAFETY: `ptr` is non-dangling slice
            Some(unsafe {
                let slice = ptr.as_mut();
                &mut slice[index]
            })
        } else {
            None
        }
    }

    fn get_from_mem<'a, U>(mem: NonNull<[U]>, index: usize) -> Option<&'a U> {
        Self::mut_from_mem(mem, index).map(|v| &*v)
    }

    fn get_header(&self) -> &LinksHeader<T> {
        // SAFETY: `LinksHeader` and `IndexPart` layout are equivalent
        unsafe {
            Self::get_from_mem(self.mem_ptr, 0)
                .map(|x| transmute(x))
                .expect("Header should be in index memory")
        }
    }

    fn mut_header(&mut self) -> &mut LinksHeader<T> {
        // SAFETY: `LinksHeader` and `IndexPart` layout are equivalent
        unsafe {
            Self::mut_from_mem(self.mem_ptr, 0)
                .map(|x| transmute(x))
                .expect("Header should be in index memory")
        }
    }

    fn get_link_part(&self, index: T) -> &LinkPart<T> {
        Self::get_from_mem(self.mem_ptr, index.as_usize())
            .expect("Data part should be in data memory")
    }

    unsafe fn get_link_part_unchecked(&self, index: T) -> &LinkPart<T> {
        Self::get_from_mem(self.mem_ptr, index.as_usize()).unwrap_unchecked()
    }

    fn mut_link_part(&mut self, index: T) -> &mut LinkPart<T> {
        Self::mut_from_mem(self.mem_ptr, index.as_usize())
            .expect("Data part should be in data memory")
    }

    unsafe fn mut_source_root(&mut self) -> *mut T {
        &mut self.mut_header().root_as_source
    }

    unsafe fn mut_target_root(&mut self) -> *mut T {
        &mut self.mut_header().root_as_target
    }

    unsafe fn detach_source_unchecked(&mut self, root: *mut T, index: T) {
        self.sources.detach(&mut *root, index);
    }

    unsafe fn detach_target_unchecked(&mut self, root: *mut T, index: T) {
        self.targets.detach(&mut *root, index);
    }

    unsafe fn attach_source_unchecked(&mut self, root: *mut T, index: T) {
        self.sources.attach(&mut *root, index);
    }

    unsafe fn attach_target_unchecked(&mut self, root: *mut T, index: T) {
        self.targets.attach(&mut *root, index);
    }

    unsafe fn detach_source(&mut self, index: T) {
        let root = self.mut_source_root();
        self.detach_source_unchecked(root, index);
    }

    unsafe fn detach_target(&mut self, index: T) {
        let root = self.mut_target_root();
        self.detach_target_unchecked(root, index);
    }

    unsafe fn attach_source(&mut self, index: T) {
        let root = self.mut_source_root();
        self.attach_source_unchecked(root, index);
    }

    unsafe fn attach_target(&mut self, index: T) {
        let root = self.mut_target_root();
        self.attach_target_unchecked(root, index);
    }

    fn get_total(&self) -> T {
        let header = self.get_header();
        header.allocated - header.free
    }

    fn is_unused(&self, link: T) -> bool {
        let header = self.get_header();
        if link <= header.allocated && header.first_free != link {
            // SAFETY: link part memory is allocated
            let link = unsafe { self.get_link_part_unchecked(link) };
            // If the link is unused (that is, it was created but deleted),
            // its search tree size is 0,
            // its source and target will be used to build a LinkedList from similar links
            link.size_as_source == T::funty(0) && link.source != T::funty(0)
        } else {
            true
        }
    }

    fn exists(&self, link: T) -> bool {
        let constants = self.constants();
        let header = self.get_header();

        link >= *constants.internal_range.start()
            && link <= header.allocated
            && !self.is_unused(link)
    }

    fn update_mem(&mut self, mem: NonNull<[LinkPart<T>]>) {
        self.mem_ptr = mem;
        self.targets.update_mem(mem);
        self.sources.update_mem(mem);
        self.unused.update_mem(mem);
    }

    unsafe fn get_link_unchecked(&self, index: T) -> Link<T> {
        debug_assert!(self.exists(index));

        let raw = self.get_link_part_unchecked(index);
        Link::new(index, raw.source, raw.target)
    }

    fn each_core(&self, handler: ReadHandler<'_, T>, query: &[T]) -> Flow {
        let constants = self.constants();

        if query.is_empty() {
            for index in T::funty(1)..=self.get_header().allocated {
                if let Some(link) = self.get_link(index) {
                    handler(link)?;
                }
            }
            return Flow::Continue;
        }

        let any = constants.any;
        let index = query[constants.index_part.as_usize()];

        if query.len() == 1 {
            return if index == any {
                self.each_core(handler, &[])
            } else if let Some(link) = self.get_link(index) {
                handler(link)
            } else {
                Flow::Continue
            };
        }

        if query.len() == 2 {
            let value = query[1];
            return if index == any {
                if value == any {
                    self.each_core(handler, &[])
                } else {
                    self.each_core(handler, &[index, value, any])?;
                    self.each_core(handler, &[index, any, value])
                }
            } else if let Some(link) = self.get_link(index) {
                if value == any || link.source == value || link.target == value {
                    handler(link)
                } else {
                    Flow::Continue
                }
            } else {
                Flow::Continue
            };
        }

        if query.len() == 3 {
            let source = query[constants.source_part.as_usize()];
            let target = query[constants.target_part.as_usize()];

            return if index == any {
                if (source, target) == (any, any) {
                    self.each_core(handler, &[])
                } else if source == any {
                    self.targets.each_usages(target, handler)
                } else if target == any {
                    self.sources.each_usages(source, handler)
                } else {
                    let link = self.sources.search(source, target);
                    self.get_link(link).map_or(Flow::Continue, handler)
                }
            } else if let Some(link) = self.get_link(index) {
                if (target, source) == (any, any) {
                    handler(link) // TODO: add (x * *) search test
                } else if target != any && source != any {
                    if (source, target) == (link.source, link.target) {
                        handler(link)
                    } else {
                        Flow::Continue
                    }
                } else if source == any {
                    if link.target == target {
                        handler(link)
                    } else {
                        Flow::Continue
                    }
                } else if target == any {
                    if link.source == source {
                        handler(link)
                    } else {
                        Flow::Continue
                    }
                } else {
                    Flow::Continue
                }
            } else {
                Flow::Continue
            };
        }
        todo!()
    }
}

impl<T: LinkType, M: RawMem<LinkPart<T>>, TS: UnitTree<T>, TT: UnitTree<T>, TU: UnitList<T>>
    Links<T> for Store<T, M, TS, TT, TU>
{
    fn constants(&self) -> &LinksConstants<T> {
        &self.constants
    }

    fn count_links(&self, query: &[T]) -> T {
        if query.is_empty() {
            return self.get_total();
        };

        let constants = self.constants();
        let any = constants.any;
        let index = query[constants.index_part.as_usize()];

        if query.len() == 1 {
            return if index == any {
                self.get_total()
            } else if self.exists(index) {
                T::funty(1)
            } else {
                T::funty(0)
            };
        }

        if query.len() == 2 {
            let value = query[1];
            return if index == any {
                if value == any {
                    self.get_total()
                } else {
                    self.targets.count_usages(value) + self.sources.count_usages(value)
                }
            } else {
                if !self.exists(index) {
                    return T::funty(0);
                }
                if value == any {
                    return T::funty(1);
                }

                return self.get_link(index).map_or_else(
                    || T::funty(0),
                    |stored| {
                        if stored.source == value || stored.target == value {
                            T::funty(1)
                        } else {
                            T::funty(0)
                        }
                    },
                );
            };
        }

        if query.len() == 3 {
            let source = query[constants.source_part.as_usize()];
            let target = query[constants.target_part.as_usize()];

            return if index == any {
                if (target, source) == (any, any) {
                    self.get_total()
                } else if source == any {
                    self.targets.count_usages(target)
                } else if target == any {
                    self.sources.count_usages(source)
                } else {
                    let link = self.sources.search(source, target);
                    if link == constants.null {
                        T::funty(0)
                    } else {
                        T::funty(1)
                    }
                }
            } else if !self.exists(index) {
                T::funty(0)
            } else if (source, target) == (any, any) {
                T::funty(1)
            } else {
                let link = unsafe { self.get_link_unchecked(index) };
                if source != any && target != any {
                    if (link.source, link.target) == (source, target) {
                        T::funty(1)
                    } else {
                        T::funty(0)
                    }
                } else if source == any {
                    if link.target == target {
                        T::funty(1)
                    } else {
                        T::funty(0)
                    }
                } else if target == any {
                    if link.source == source {
                        T::funty(1)
                    } else {
                        T::funty(0)
                    }
                } else {
                    T::funty(0)
                }
            };
        }
        todo!()
    }

    fn create_links(
        &mut self,
        _query: &[T],
        handler: WriteHandler<'_, T>,
    ) -> Result<Flow, LinksError<T>> {
        let constants = self.constants();
        let header = self.get_header();
        let mut free = header.first_free;
        if free == constants.null {
            let max_inner = *constants.internal_range.end();
            if header.allocated >= max_inner {
                return Err(LinksError::LimitReached(max_inner));
            }

            if header.allocated >= header.reserved - T::funty(1) {
                let mem = self
                    .mem
                    .alloc(self.mem.allocated() + self.reserve_step)?
                    .leak();
                self.update_mem(mem);
                let reserved = self.mem.allocated();
                let header = self.mut_header();
                header.reserved = T::try_from(reserved).expect("always ok");
            }
            let header = self.mut_header();
            header.allocated += T::funty(1);
            free = header.allocated;
        } else {
            self.unused.detach(free);
        }
        Ok(handler(
            Link::nothing(),
            Link::new(free, T::funty(0), T::funty(0)),
        ))
    }

    fn each_links(&self, query: &[T], handler: ReadHandler<'_, T>) -> Flow {
        self.each_core(handler, &query.to_query()[..])
    }

    fn update_links(
        &mut self,
        query: &[T],
        change: &[T],
        handler: WriteHandler<'_, T>,
    ) -> Result<Flow, LinksError<T>> {
        let index = query[0];
        let source = change[1];
        let target = change[2];
        let old_source = source;
        let old_target = target;

        let link = self.try_get_link(index)?;

        if link.source != T::funty(0) {
            // SAFETY: Here index detach from sources
            // by default source is zero
            unsafe {
                self.detach_source(index);
            }
        }
        if link.target != T::funty(0) {
            // SAFETY: Here index detach from targets
            // by default target is zero
            unsafe {
                self.detach_target(index);
            }
        }

        let place = self.mut_link_part(index);
        place.source = source;
        place.target = target;
        let place = place.clone();

        if place.source != T::funty(0) {
            // SAFETY: Here index attach to sources
            unsafe {
                self.attach_source(index);
            }
        }
        if place.target != T::funty(0) {
            // SAFETY: Here index attach to targets
            unsafe {
                self.attach_target(index);
            }
        }

        Ok(handler(
            Link::new(index, old_source, old_target),
            Link::new(index, source, target),
        ))
    }

    fn delete_links(
        &mut self,
        query: &[T],
        handler: WriteHandler<'_, T>,
    ) -> Result<Flow, LinksError<T>> {
        let index = query[0];

        let link = self.try_get_link(index)?;
        self.update(index, T::funty(0), T::funty(0))?;

        let header = self.get_header();
        match index.cmp(&header.allocated) {
            Ordering::Less => self.unused.attach_as_first(index),
            Ordering::Equal => {
                let allocated = self.get_header().allocated;
                let header = self.mut_header();
                header.allocated = allocated - T::funty(1);

                loop {
                    let allocated = self.get_header().allocated;
                    if !(allocated > T::funty(0) && self.is_unused(allocated)) {
                        break;
                    }
                    self.unused.detach(allocated);
                    self.mut_header().allocated = allocated - T::funty(1);
                }
            }
            // fixme: possible unreachable_unchecked
            Ordering::Greater => unreachable!(),
        }

        Ok(handler(link, Link::nothing()))
    }
}

impl<T: LinkType, M: RawMem<LinkPart<T>>, TS: UnitTree<T>, TT: UnitTree<T>, TU: UnitList<T>>
    Doublets<T> for Store<T, M, TS, TT, TU>
{
    fn get_link(&self, index: T) -> Option<Link<T>> {
        if self.exists(index) {
            // SAFETY: links is exists
            Some(unsafe { self.get_link_unchecked(index) })
        } else {
            None
        }
    }
}

// SAFETY: No read operations result in a write
unsafe impl<T: LinkType, M: RawMem<LinkPart<T>>, TS: UnitTree<T>, TT: UnitTree<T>, TU: UnitList<T>>
    Sync for Store<T, M, TS, TT, TU>
{
}

// SAFETY: All data is moved together with the `Store`
unsafe impl<T: LinkType, M: RawMem<LinkPart<T>>, TS: UnitTree<T>, TT: UnitTree<T>, TU: UnitList<T>>
    Send for Store<T, M, TS, TT, TU>
{
}
