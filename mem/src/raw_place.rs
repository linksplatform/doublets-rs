use std::{
    alloc::Layout,
    fmt::{self, Formatter},
    marker::PhantomData,
    mem::{self, MaybeUninit},
    ptr::{self, NonNull},
    slice,
};

pub struct RawPlace<T> {
    ptr: NonNull<T>,
    len: usize, // use to drop at panic
    cap: usize, // usually `cap` is same `len`
    _marker: PhantomData<T>,
}

impl<T> RawPlace<T> {
    pub const fn dangling() -> Self {
        Self { ptr: NonNull::dangling(), len: 0, cap: 0, _marker: PhantomData }
    }

    pub fn cap(&self) -> usize {
        self.cap
    }

    pub unsafe fn as_slice(&self) -> &[T] {
        slice::from_raw_parts(self.ptr.as_ptr(), self.len)
    }

    pub unsafe fn as_slice_mut(&mut self) -> &mut [T] {
        slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len)
    }

    pub fn current_memory(&self) -> Option<(NonNull<u8>, Layout)> {
        // rust does not support such types,
        // so we can do better by skipping some checks and avoid an unwrap.
        const { assert!(mem::size_of::<T>() % mem::align_of::<T>() == 0) };

        if self.cap == 0 {
            None
        } else {
            unsafe {
                let layout = Layout::from_size_align_unchecked(
                    mem::size_of::<T>().unchecked_mul(self.cap),
                    mem::align_of::<T>(),
                );
                Some((self.ptr.cast(), layout))
            }
        }
    }

    pub unsafe fn handle_fill(
        &mut self,
        (ptr, cap): (NonNull<T>, usize),
        inited: usize,
        fill: impl FnOnce(usize, (&mut [T], &mut [MaybeUninit<T>])),
    ) -> &mut [T] {
        // fixme: ZST correctness isn't checked now,
        // it forbid growing, but allow `RawPlace::<ZST>::dangling` and thus `Alloc::<ZST>::new`'s
        const { assert!(mem::size_of::<T>() != 0) };

        let uninit = NonNull::slice_from_raw_parts(ptr, cap)
            .get_unchecked_mut(self.cap..)
            .as_uninit_slice_mut();

        self.ptr = ptr;
        self.cap = cap; // `ptr` and `cap` changes after panicking `fill`
        //                 ( alloc memory )

        // slice from `as_slice_mut` will be the initialized part of owned memory
        // while (&mut [T], &mut [MaybeUninit<T>]) will be the full memory
        fill(inited, (self.as_slice_mut(), uninit)); // panic out!

        self.len = cap; // `len` is same `cap` only if `uninit` was init

        MaybeUninit::slice_assume_init_mut(uninit)
    }

    pub fn shrink_to(&mut self, cap: usize) {
        assert!(cap <= self.cap);

        unsafe {
            ptr::drop_in_place(&mut self.as_slice_mut()[cap..]);
        }

        self.cap = cap;
        self.len = cap;
    }

    pub fn set_ptr(&mut self, ptr: NonNull<[u8]>) {
        debug_assert_eq!(
            ptr.len(),
            self.cap * mem::size_of::<T>(),
            "Usually you have to call `.shrink_to` first to drop shrunk memory, \
             this should be followed by a call to `.set_ptr`."
        );

        self.ptr = ptr.cast();
    }
}

impl<T> fmt::Debug for RawPlace<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "({:?}::{})", self.ptr, self.cap)
    }
}

unsafe impl<T: Sync> Sync for RawPlace<T> {}
unsafe impl<T: Send> Send for RawPlace<T> {}

#[test]
fn zst_build() {
    let _: RawPlace<()> = RawPlace::dangling();
}
