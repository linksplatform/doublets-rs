use {
    crate::{
        mem::{unit::LinkRepr, Header},
        Link,
    },
    core::{LinkType, Repr},
    either::Either,
    mem::RawMem,
    smallvec::SmallVec,
    std::{cmp, marker::PhantomData, mem::size_of},
};

pub enum Param<T> {
    Any,
    Some(T),
}

impl<T> Param<T> {
    pub fn option(self) -> Option<T> {
        match self {
            Param::Any => None,
            Param::Some(x) => Some(x),
        }
    }
}

impl<T> From<T> for Param<T> {
    fn from(value: T) -> Self {
        Param::Some(value)
    }
}

pub struct IterNever<T> {
    _marker: PhantomData<T>,
}

impl<T> Iterator for IterNever<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        unreachable!()
    }
}

macro_rules! choice {
    [$ty:ty] => (Either<$ty, IterNever<Link<T>>>);
    [$ty:ty, $($tail:ty),+] => (Either<$ty, choice![$($tail),+]>);

    (0 <- $x:expr) => (Either::Left($x));
    (1 <- $x:expr) => (Either::Right(choice!(0 <- $x)));
    (2 <- $x:expr) => (Either::Right(choice!(1 <- $x)));
    (3 <- $x:expr) => (Either::Right(choice!(2 <- $x)));
    (4 <- $x:expr) => (Either::Right(choice!(3 <- $x)));
    (5 <- $x:expr) => (Either::Right(choice!(4 <- $x)));
    (6 <- $x:expr) => (Either::Right(choice!(5 <- $x)));
    (7 <- $x:expr) => (Either::Right(choice!(6 <- $x)));
    (8 <- $x:expr) => (Either::Right(choice!(7 <- $x)));
    (9 <- $x:expr) => (Either::Right(choice!(8 <- $x)));
}

// temp constant
pub const STEP: usize = 1024 * 1024;

pub struct Store<T, M, Sources: ?Sized = super::Sources<T>, Targets: ?Sized = super::Targets<T>> {
    mem: M,
    _marker: PhantomData<(T, fn(Sources), fn(Targets))>,
}

impl<
    T: LinkType,
    M: RawMem<Item = LinkRepr<T>>,
    Sources: super::Tree<Item = T> + ?Sized,
    Targets: super::Tree<Item = T> + ?Sized,
> Store<T, M, Sources, Targets>
{
    pub fn new(mem: M) -> Self {
        Self { mem, _marker: Default::default() }.init()
    }

    // header getters is safe because there is no way to call it avoiding `.init()`
    fn header(&self) -> Header<T> {
        // Safety: first has similar layout with header
        unsafe { *(self.mem.allocated().get_unchecked(0) as *const _ as *const Header<T>) }
    }

    fn header_mut(&mut self) -> &mut Header<T> {
        // Safety: first has similar layout with header
        unsafe { std::mem::transmute(&mut self.mem.allocated_mut()[0]) }
    }

    fn grow_exact(&mut self, cap: usize) {
        unsafe {
            self.grow_exact_dumb(cap);
            self.header_mut().reserved.add_addr(cap);
        }
    }

    unsafe fn grow_exact_dumb(&mut self, cap: usize) {
        self.mem.grow_zeroed_exact(cap).unwrap();
    }

    fn init(mut self) -> Self {
        // grow to read header
        unsafe {
            self.grow_exact_dumb(1);
        }

        // header was initialized before
        unsafe {
            self.grow_exact_dumb(cmp::max(STEP, self.header().allocated.addr()));
        }

        // `reserved` is a size of allocated link reprs, excluding header
        self.header_mut().reserved = T::from_addr(self.mem.allocated().len() - 1);
        self
    }

    unsafe fn get_repr_unchecked(&self, idx: T) -> &LinkRepr<T> {
        self.mem.allocated().get_unchecked(idx.addr())
    }

    unsafe fn get_repr_unchecked_mut(&mut self, idx: T) -> &mut LinkRepr<T> {
        self.mem.allocated_mut().get_unchecked_mut(idx.addr())
    }

    fn get_link(&self, idx: T) -> Option<Link<T>> {
        if self.is_exist(idx) {
            let &LinkRepr { source, target, .. } = unsafe { self.get_repr_unchecked(idx) };
            Some(Link::new(idx, source, target))
        } else {
            None
        }
    }

    fn try_get_link(&self, idx: T) -> Result<Link<T>, !> {
        self.get_link(idx).ok_or_else(|| todo!())
    }

    fn is_uninit(&self, _idx: T) -> bool {
        false
    }

    fn is_valid(&self, idx: usize) -> bool {
        idx < (u64::MAX >> (64 - size_of::<T>() * 8)) as usize
    }

    fn is_exist(&self, idx: T) -> bool {
        idx.addr() > 0 && idx.addr() <= self.header().allocated.addr() && !self.is_uninit(idx)
    }

    fn sources(&self) -> &Sources {
        Sources::with(self.mem.allocated())
    }

    fn sources_mut(&mut self) -> &mut Sources {
        Sources::with_mut(self.mem.allocated_mut())
    }

    fn targets(&self) -> &Targets {
        Targets::with(self.mem.allocated())
    }

    fn targets_mut(&mut self) -> &mut Targets {
        Targets::with_mut(self.mem.allocated_mut())
    }

    pub fn create_new(&mut self) -> Link<T> {
        let header = self.header();
        let idx = if header.first_free.addr() == 0 {
            if !self.is_valid(header.allocated.addr() + 1) {
                todo!("overflows")
            }

            if header.allocated.addr() + 1 >= header.reserved.addr() {
                self.grow_exact(STEP);
            }

            self.header_mut().allocated.add_addr(1);
            self.header().allocated
        } else {
            todo!()
        };

        Link::new(idx, T::from_addr(0), T::from_addr(0))
    }

    fn update_one(&mut self, idx: T, (source, target): (T, T)) -> Result<(), !> {
        let link = self.try_get_link(idx)?;

        if let Some(source) = Repr::from_addr(link.source) {
            let root = self.header().sources_root;
            self.header_mut().sources_root =
                Some(self.sources_mut().detach(root, source).ok_or_else(|| todo!())?);
        }

        if let Some(target) = Repr::from_addr(link.target) {
            let root = self.header().targets_root;
            self.header_mut().targets_root =
                Some(self.targets_mut().detach(root, target).ok_or_else(|| todo!())?);
        }

        let place = unsafe { self.get_repr_unchecked_mut(idx) };
        *place = LinkRepr { source, target, ..*place };

        if let Some(source) = Repr::from_addr(source) {
            let root = self.header().sources_root;
            self.header_mut().sources_root =
                Some(self.sources_mut().attach(root, source).ok_or_else(|| todo!())?);
        }

        if let Some(target) = Repr::from_addr(target) {
            let root = self.header().targets_root;
            self.header_mut().targets_root =
                Some(self.targets_mut().attach(root, target).ok_or_else(|| todo!())?)
        }

        Ok(())
    }

    fn count_total(&self) -> usize {
        let Header { allocated, free, .. } = self.header();
        allocated.addr() - free.addr()
    }

    fn count_of(
        &self,
        (index, source, target): (impl Into<Param<T>>, impl Into<Param<T>>, impl Into<Param<T>>),
    ) -> usize {
        use Param::{Any, Some};

        match (index.into(), source.into(), target.into()) {
            (Any, Any, Any) => self.count_total(),
            (Any, Some(source), Any) => {
                if let Option::Some(root) = self.header().sources_root {
                    self.sources().count_usage(root, source)
                } else {
                    0
                }
            }
            (Any, Any, Some(target)) => {
                if let Option::Some(root) = self.header().targets_root {
                    self.targets().count_usage(root, target)
                } else {
                    0
                }
            }
            other => self.each_iter(other).count(),
        }
    }

    fn each_iter(
        &self,
        (index, source, target): (impl Into<Param<T>>, impl Into<Param<T>>, impl Into<Param<T>>),
    ) -> impl Iterator<Item = Link<T>> + '_ {
        use Param::{Any, Some};

        let infer: choice![_, _, _] = match (index.into(), source.into(), target.into()) {
            (Any, Any, Any) => choice!(0 <- {
                (1..self.header().allocated.addr() )
                    .map(T::from_addr)
                    .filter_map(|idx| self.get_link(idx))
            }),
            (Some(index), Any, Any) => choice!(1 <- {
                self.get_link(index).into_iter()
            }),
            (Any, Some(source), Some(target)) => choice!(1 <- {
                let index = self.sources().find(
                    self.header().sources_root,
                    source,
                    target,
                );
                self.get_link(T::from_repr(index)).into_iter()
            }),
            (Some(index), source, target) => choice!(1 <- {
                // compare with `source` and `target` or skip
                if let Option::Some(link) = self.get_link(index)
                    && source.option().map(|s| s.addr() == link.source.addr()).unwrap_or(true)
                    && target.option().map(|t| t.addr() == link.target.addr()).unwrap_or(true)
                {
                    Option::Some(link)
                } else {
                    None
                }
                .into_iter()
            }),
            (Any, Some(source), Any) => choice!(2 <- {
                Stacked::new(|stack| {
                    self.sources().each_usages(self.header().sources_root, source, |link| {
                        stack.push(link)
                    });
                })
            }),
            (Any, Any, Some(target)) => choice!(2 <- {
                Stacked::new(|stack| {
                    self.targets().each_usages(self.header().targets_root, target, |link| {
                        stack.push(link)
                    });
                })
            }),
        };
        infer
    }
}

struct Stacked<T> {
    stack: SmallVec<T, 8>,
}

impl<T> Stacked<T> {
    pub fn new(mut iter: impl FnMut(&mut SmallVec<T, 8>)) -> Self {
        let mut stack = SmallVec::new();
        iter(&mut stack);
        Self { stack }
    }
}

impl<T> Iterator for Stacked<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.stack.pop()
    }
}
