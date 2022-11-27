/// Stable `try` block only for `Result`
macro_rules! stable_try {
    ($($block:tt)*) => {
        (|| {
            let ret = { $($block)* };
            core::result::Result::Ok(ret)
        })()
    };
}

pub(crate) use stable_try;
use std::{
    fmt::{self, Debug, Formatter},
    marker::PhantomData,
    ptr::{self, NonNull},
};

/// repr C type unit type
/// with `sizeof == 0` and `alignof == 1`:
#[repr(C)]
pub struct None {
    _nonzero: [u8; 0],
}

impl None {
    pub fn new() -> Self {
        Self { _nonzero: [] }
    }
}

impl Debug for None {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "None value")
    }
}

#[repr(C, usize)]
pub enum Fallible<T, E> {
    Ok(T),
    Err(E),
}

impl<T, E: Debug> Fallible<T, E> {
    pub fn unwrap(self) -> T {
        match self {
            Self::Ok(val) => val,
            Self::Err(err) => {
                panic!("called `Fallible::unwrap()` on a `Err` value: {err:?}")
            }
        }
    }
}

pub type Maybe<T> = Fallible<T, None>;

impl<T> Maybe<T> {
    pub fn none() -> Self {
        Self::Err(None::new())
    }
}

impl<T, E> From<Result<T, E>> for Fallible<T, E> {
    fn from(result: Result<T, E>) -> Self {
        match result {
            Ok(ok) => Self::Ok(ok),
            Err(err) => Self::Err(err),
        }
    }
}

impl<T, E> From<Fallible<T, E>> for Result<T, E> {
    fn from(fallible: Fallible<T, E>) -> Self {
        match fallible {
            Fallible::Ok(ok) => Ok(ok),
            Fallible::Err(err) => Err(err),
        }
    }
}

impl<T> From<Option<T>> for Maybe<T> {
    fn from(opt: Option<T>) -> Self {
        match opt {
            Some(val) => Maybe::Ok(val),
            None => Maybe::none(),
        }
    }
}

impl<T> From<Maybe<T>> for Option<T> {
    fn from(maybe: Maybe<T>) -> Self {
        if let Maybe::Ok(val) = maybe {
            Some(val)
        } else {
            None
        }
    }
}

/// `OwnedSlice<T>` is a FFI-Safe `Box<[T]>` representation
#[repr(C)]
pub struct OwnedSlice<T> {
    ptr: NonNull<T>,
    len: usize,
    // actually it's still a `Box<[T]>`
    _marker: PhantomData<Box<[T]>>,
}

impl<T> OwnedSlice<T> {
    #[inline]
    pub fn slice_from_raw_parts(data: NonNull<T>, len: usize) -> NonNull<[T]> {
        // SAFETY: `data` is a `NonNull` pointer which is necessarily non-null
        unsafe { NonNull::new_unchecked(ptr::slice_from_raw_parts_mut(data.as_ptr(), len)) }
    }

    pub fn leak(place: Box<[T]>) -> Self {
        let leak = NonNull::from(Box::leak(place));
        OwnedSlice {
            // ptr: leak.as_non_null_ptr(),
            ptr: leak.cast(),
            len: leak.len(),
            _marker: PhantomData,
        }
    }

    pub fn as_slice(&self) -> &[T] {
        let slice = Self::slice_from_raw_parts(self.ptr, self.len);
        // SAFETY: `Self` is opaque we create Box and we drop it
        unsafe { slice.as_ref() }
    }

    /// # Safety
    /// forget `self` after `.keep_own`
    pub unsafe fn keep_own(&self) -> Box<[T]> {
        let slice = Self::slice_from_raw_parts(self.ptr, self.len);
        unsafe { Box::from_raw(slice.as_ptr()) }
    }

    pub fn into_owned(self) -> Box<[T]> {
        // SAFETY: `self` drop after call `.into_owned`
        unsafe { self.keep_own() }
    }
}

impl<T: Debug> Debug for OwnedSlice<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.as_slice())
    }
}

impl<T> Drop for OwnedSlice<T> {
    fn drop(&mut self) {
        // SAFETY: `self` drop at end of this scope
        let _ = unsafe { self.keep_own() };
    }
}
