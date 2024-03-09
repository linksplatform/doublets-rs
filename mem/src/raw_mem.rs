use std::{
    alloc::Layout,
    mem::MaybeUninit,
    ops::{Range, RangeBounds},
    slice,
};

/// Error memory allocation
// fixme: maybe we should add `(X bytes)` after `cannot allocate/occupy`
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// Error due to the computed capacity exceeding the maximum
    /// (usually `isize::MAX` bytes).
    ///
    /// ## Examples
    ///
    /// grow more than `isize::MAX` bytes:
    ///
    /// ```
    /// # #![feature(allocator_api)]
    /// # #![feature(assert_matches)]
    /// # use std::alloc::Global;
    /// # use std::assert_matches::assert_matches;
    /// # use platform_mem::{Error, Alloc, RawMem};
    /// let mut mem = Alloc::new(Global);
    /// assert_matches!(mem.grow_filled(usize::MAX, 0u64), Err(Error::CapacityOverflow));
    /// ```
    #[error("exceeding the capacity maximum")]
    CapacityOverflow,

    #[error("can't grow {to_grow} elements, only available {available}")]
    OverGrow { to_grow: usize, available: usize },

    /// The memory allocator returned an error
    #[error("memory allocation of {layout:?} failed")]
    AllocError {
        /// The layout of allocation request that failed
        layout: Layout,

        #[doc(hidden)]
        non_exhaustive: (),
    },

    /// System error memory allocation occurred
    #[error(transparent)]
    System(#[from] std::io::Error),
}

/// Alias for `Result<T, Error>` to return from `RawMem` methods
pub type Result<T> = std::result::Result<T, Error>;

pub trait RawMem {
    type Item;

    fn allocated(&self) -> &[Self::Item];
    fn allocated_mut(&mut self) -> &mut [Self::Item];

    /// # Safety
    /// Caller must guarantee that `fill` makes the uninitialized part valid for
    /// [`MaybeUninit::slice_assume_init_mut`]
    ///
    /// ### Incorrect usage
    /// ```no_run
    /// # #![feature(allocator_api)]
    /// # use std::alloc::Global;
    /// # use std::mem::MaybeUninit;
    /// # use platform_mem::Result;
    /// use platform_mem::{Alloc, RawMem};
    ///
    /// let mut alloc = Alloc::new(Global);
    /// unsafe {
    ///     alloc.grow(10, |_init, (_, _uninit): (_, &mut [MaybeUninit<u64>])| {
    ///         // `RawMem` relies on the fact that we initialize memory
    ///         // even if they are primitives
    ///     })?;
    /// }
    /// # Result::Ok(())
    /// ```
    unsafe fn grow(
        &mut self,
        cap: usize,
        fill: impl FnOnce(usize, (&mut [Self::Item], &mut [MaybeUninit<Self::Item>])),
    ) -> Result<&mut [Self::Item]>;

    fn shrink(&mut self, cap: usize) -> Result<()>;

    fn size_hint(&self) -> Option<usize> {
        None
    }

    /// [`grow`] which assumes that the memory is already initialized
    ///
    /// # Safety
    ///
    /// When calling this method, you have to ensure that one of the following is true:
    ///
    /// * memory already initialized as [`Item`]
    ///
    /// * memory is initialized bytes and [`Item`] can be represented as bytes
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use platform_mem::Result;
    /// use platform_mem::{FileMapped, RawMem};
    ///
    /// let mut file = FileMapped::from_path("..")?;
    /// // file is always represents as initialized bytes
    /// // and usize is transparent as bytes
    /// let _: &mut [usize] = unsafe { file.grow_assumed(10)? };
    /// # Result::Ok(())
    /// ```
    ///
    /// [`grow`]: Self::grow
    /// [`Item`]: Self::Item
    unsafe fn grow_assumed(&mut self, cap: usize) -> Result<&mut [Self::Item]> {
        self.grow(cap, |inited, (_, uninit)| {
            // fixme: maybe change it to `assert_eq!`
            debug_assert_eq!(
                inited,
                uninit.len(),
                "grown memory must be initialized, \
                 usually allocators-like provide uninitialized memory, \
                 which is only safe for writing"
            )
        })
    }

    /// # Safety
    /// [`Item`] must satisfy [initialization invariant][inv] for [`mem::zeroed`]
    ///
    /// [`Item`]: Self::Item
    ///  [inv]: MaybeUninit#initialization-invariant
    ///
    /// # Examples
    /// Correct usage of this function: initializing an integral-like types with zeroes:
    /// ```
    /// # #![feature(allocator_api)]
    /// # use platform_mem::Error;
    /// use platform_mem::{Global, RawMem};
    ///
    /// let mut alloc = Global::new();
    /// let zeroes: &mut [(u8, u16)] = unsafe {
    ///     alloc.grow_zeroed(10)?
    /// };
    ///
    /// assert_eq!(zeroes, [(0, 0); 10]);
    /// # Ok::<_, Error>(())
    /// ```
    ///
    /// Incorrect usage of this function: initializing a reference with zero:
    /// ```no_run
    /// # #![feature(allocator_api)]
    ///  # use platform_mem::Error;
    /// use platform_mem::{Global, RawMem};
    ///
    /// let mut alloc = Global::new();
    /// let zeroes: &mut [&'static str] = unsafe {
    ///     alloc.grow_zeroed(10)? // Undefined behavior!
    /// };
    ///
    /// # Ok::<_, Error>(())
    /// ```
    ///
    unsafe fn grow_zeroed(&mut self, cap: usize) -> Result<&mut [Self::Item]> {
        self.grow(cap, |_, (_, uninit)| {
            uninit.as_mut_ptr().write_bytes(0u8, uninit.len());
        })
    }

    unsafe fn grow_zeroed_exact(&mut self, cap: usize) -> Result<&mut [Self::Item]> {
        self.grow(cap, |inited, (_, uninit)| {
            uninit.get_unchecked_mut(inited..).as_mut_ptr().write_bytes(0u8, uninit.len());
        })
    }

    fn grow_with(
        &mut self,
        addition: usize,
        f: impl FnMut() -> Self::Item,
    ) -> Result<&mut [Self::Item]> {
        unsafe {
            self.grow(addition, |_, (_, uninit)| {
                uninit::fill_with(uninit, f);
            })
        }
    }

    unsafe fn grow_with_exact(
        &mut self,
        addition: usize,
        f: impl FnMut() -> Self::Item,
    ) -> Result<&mut [Self::Item]> {
        unsafe {
            self.grow(addition, |inited, (_, uninit)| {
                uninit::fill_with(&mut uninit[inited..], f);
            })
        }
    }

    fn grow_filled(&mut self, cap: usize, value: Self::Item) -> Result<&mut [Self::Item]>
    where
        Self::Item: Clone,
    {
        unsafe {
            self.grow(cap, |_, (_, uninit)| {
                uninit::fill(uninit, value);
            })
        }
    }

    unsafe fn grow_filled_exact(
        &mut self,
        cap: usize,
        value: Self::Item,
    ) -> Result<&mut [Self::Item]>
    where
        Self::Item: Clone,
    {
        unsafe {
            self.grow(cap, |inited, (_, uninit)| {
                uninit::fill(&mut uninit[inited..], value);
            })
        }
    }

    fn grow_within<R: RangeBounds<usize>>(&mut self, range: R) -> Result<&mut [Self::Item]>
    where
        Self::Item: Clone,
    {
        let Range { start, end } = slice::range(range, ..self.allocated().len());
        unsafe {
            self.grow(end - start, |_, (within, uninit)| {
                MaybeUninit::clone_from_slice(uninit, &within[start..end]);
            })
        }
    }

    fn grow_from_slice(&mut self, src: &[Self::Item]) -> Result<&mut [Self::Item]>
    where
        Self::Item: Clone,
    {
        unsafe {
            self.grow(src.len(), |_, (_, uninit)| {
                MaybeUninit::clone_from_slice(uninit, src);
            })
        }
    }
}

struct Unique<T>(MaybeUninit<T>);

impl<T> Unique<T> {
    pub unsafe fn assume(unique: T) -> Self {
        Self(MaybeUninit::new(unique))
    }
}

impl<A, B, F: FnOnce(A, B)> FnOnce<(A, B)> for Unique<F> {
    type Output = ();

    extern "rust-call" fn call_once(self, args: (A, B)) -> Self::Output {
        unsafe { self.0.assume_init().call_once(args) }
    }
}

impl<A, B, F: FnOnce(A, B)> FnMut<(A, B)> for Unique<F> {
    extern "rust-call" fn call_mut(&mut self, args: (A, B)) -> Self::Output {
        unsafe { self.0.assume_init_read().call_once(args) }
    }
}

pub unsafe trait ErasedMem {
    type Item;

    fn erased_allocated(&self) -> &[Self::Item];
    fn erased_allocated_mut(&mut self) -> &mut [Self::Item];

    unsafe fn erased_grow(
        &mut self,
        cap: usize,
        fill: &mut dyn FnMut(usize, (&mut [Self::Item], &mut [MaybeUninit<Self::Item>])),
    ) -> Result<&mut [Self::Item]>;

    fn erased_shrink(&mut self, cap: usize) -> Result<()>;

    fn erased_size_hint(&self) -> Option<usize> {
        None
    }
}

macro_rules! impl_erased {
    ($ty:ty => $($imp:tt)+) => {
        impl $($imp)+ {
            type Item = $ty;

            fn allocated(&self) -> &[Self::Item] {
                (**self).erased_allocated()
            }

            fn allocated_mut(&mut self) -> &mut [Self::Item] {
                (**self).erased_allocated_mut()
            }

            unsafe fn grow(
                &mut self,
                cap: usize,
                fill: impl FnOnce(usize, (&mut [Self::Item], &mut [MaybeUninit<Self::Item>])),
            ) -> Result<&mut [Self::Item]> {
                (**self).erased_grow(cap, unsafe { &mut Unique::assume(fill) })
            }

            fn shrink(&mut self, cap: usize) -> Result<()> {
                (**self).erased_shrink(cap)
            }

            fn size_hint(&self) -> Option<usize> {
                (**self).erased_size_hint()
            }
        }
    };
}

impl_erased!(All::Item => <'a, All: ?Sized + RawMem> RawMem for &'a mut All);

impl_erased!(I => <'a, I> RawMem for Box<dyn ErasedMem<Item = I> + 'a>);
impl_erased!(I => <'a, I> RawMem for Box<dyn ErasedMem<Item = I> + Sync + 'a>);
impl_erased!(I => <'a, I> RawMem for Box<dyn ErasedMem<Item = I> + Sync + Send + 'a>);

unsafe impl<All: RawMem + ?Sized> ErasedMem for All {
    type Item = All::Item;

    fn erased_allocated(&self) -> &[Self::Item] {
        self.allocated()
    }

    fn erased_allocated_mut(&mut self) -> &mut [Self::Item] {
        self.allocated_mut()
    }

    unsafe fn erased_grow(
        &mut self,
        cap: usize,
        fill: &mut dyn FnMut(usize, (&mut [Self::Item], &mut [MaybeUninit<Self::Item>])),
    ) -> Result<&mut [Self::Item]> {
        self.grow(cap, fill)
    }

    fn erased_shrink(&mut self, cap: usize) -> Result<()> {
        self.shrink(cap)
    }

    fn erased_size_hint(&self) -> Option<usize> {
        self.size_hint()
    }
}

pub mod uninit {
    use std::{mem, mem::MaybeUninit, ptr};

    pub fn fill<T: Clone>(uninit: &mut [MaybeUninit<T>], val: T) {
        let mut guard = Guard { slice: uninit, init: 0 };

        if let Some((last, elems)) = guard.slice.split_last_mut() {
            for el in elems.iter_mut() {
                el.write(val.clone());
                guard.init += 1;
            }
            last.write(val);
            guard.init += 1;
        }

        mem::forget(guard);
    }

    pub fn fill_with<T>(uninit: &mut [MaybeUninit<T>], mut fill: impl FnMut() -> T) {
        let mut guard = Guard { slice: uninit, init: 0 };

        for el in guard.slice.iter_mut() {
            el.write(fill());
            guard.init += 1;
        }

        mem::forget(guard);
    }

    struct Guard<'a, T> {
        slice: &'a mut [MaybeUninit<T>],
        init: usize,
    }

    impl<T> Drop for Guard<'_, T> {
        fn drop(&mut self) {
            debug_assert!(self.init <= self.slice.len());
            // SAFETY: this raw slice will contain only initialized objects
            // that's why, it is allowed to drop it.
            unsafe {
                ptr::drop_in_place(MaybeUninit::slice_assume_init_mut(
                    self.slice.get_unchecked_mut(..self.init),
                ));
            }
        }
    }
}
