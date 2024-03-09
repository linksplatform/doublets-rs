use std::{mem, num::*, ops::Range};

/// Types that can be stored at tree's nodes
///
/// # Safety
/// - the [`remove_idx`] implementation should return `true`
/// if the value of element at provided address matches the [root at stack].\
/// Example of `NonZero` implementation:
/// ```
/// # use std::{mem, ops::Range};
/// # struct Compile;
/// # impl Compile {
/// unsafe fn remove_idx(addr: *mut Self, from: Range<*const u8>) -> bool {
///     let addr = addr as *const u8;
///     // addr contains in tree addr space - it's inner root
///     if from.contains(&addr) {
///         let ptr = addr.sub(mem::size_of::<Self>()) as *mut Option<Self>;
///         *ptr = None;
///         false
///     } else {
///         // it's root at the stack
///         true
///     }
/// }
/// # }
/// ```
///
/// [`remove_idx`]: Self::remove_idx
/// [root at stack]: #
pub unsafe trait Leaf: Copy {
    #[must_use] // wait for: arbitrary_self_types
    unsafe fn remove_idx(addr: *mut Self, from: Range<*const u8>) -> bool;

    #[must_use]
    fn addr_of(self) -> usize;

    #[must_use]
    fn same(self, other: Self) -> bool {
        self.addr_of() == other.addr_of()
    }
}

macro_rules! impl_integral {
    ($($ty:ty)*) => {$(
        unsafe impl Leaf for $ty {
            #[inline]
            #[allow(clippy::size_of_in_element_count)] // false positive?
            unsafe fn remove_idx(addr: *mut Self, from: Range<*const u8>) -> bool {
                let addr = addr as *const u8;
                if from.contains(&addr) {
                    // calculate parent `Option`'s addr
                    let ptr = addr.sub(mem::size_of::<Self>()) as *mut Option<Self>;
                    *ptr = None;
                    false
                } else {
                    // if addr of root at the stack
                    true
                }
            }

            #[inline]
            fn addr_of(self) -> usize {
                self as usize
            }

            #[inline]
            fn same(self, other: Self) -> bool {
                self == other
            }
        }
    )*};
}

macro_rules! impl_non_zero {
    ($($ty:ty)*) => {$(
        unsafe impl Leaf for $ty {
            #[inline]
            unsafe fn remove_idx(addr: *mut Self, from: Range<*const u8>) -> bool {
                let addr = addr as *const u8;
                if from.contains(&addr) {
                    // Option<NonZero[num]> is repr as [num]
                    let ptr = addr as *mut Option<Self>;
                    *ptr = None;
                    false
                } else {
                    // it is addr of root at the stack
                    true
                }
            }

            #[inline]
            fn addr_of(self) -> usize {
                self.get() as usize
            }

            #[inline]
            fn same(self, other: Self) -> bool {
                self == other
            }
        }
    )*};
}

impl_integral! {
    u8 u16 u32 u64 usize
}

impl_non_zero! {
    NonZeroU8 NonZeroU16 NonZeroU32 NonZeroU64 NonZeroUsize
}
