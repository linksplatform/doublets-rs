use crate::{c_char, Marker};
use doublets::{data::LinkType, mem, Doublet, Error, Link};
use std::{
    cell::RefCell,
    cmp, error,
    ffi::c_short,
    fmt,
    fmt::{Debug, Display, Formatter},
    mem::MaybeUninit,
    ptr,
    ptr::NonNull,
};
use tracing::warn;

type OpaqueError = Box<dyn error::Error>;

/// `OwnedSlice<T>` is a FFI-Safe `Box<[T]>` representation
#[repr(C)]
pub struct OwnedSlice<T> {
    ptr: NonNull<T>,
    len: usize,
}

impl<T> OwnedSlice<T> {
    pub fn leak(place: Box<[T]>) -> Self {
        let leak = NonNull::from(Box::leak(place));
        OwnedSlice {
            ptr: leak.as_non_null_ptr(),
            len: leak.len(),
        }
    }

    pub fn as_slice(&self) -> &[T] {
        let slice = NonNull::slice_from_raw_parts(self.ptr, self.len);
        // SAFETY: `Self` is opaque we create Box and we drop it
        unsafe { slice.as_ref() }
    }

    /// # Safety
    /// forget `self` after `.keep_own`
    pub unsafe fn keep_own(&self) -> Box<[T]> {
        let slice = NonNull::slice_from_raw_parts(self.ptr, self.len);
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

#[repr(C, usize)]
#[derive(Debug)]
pub enum DoubletsResult<T: LinkType> {
    // oks
    Break,
    Continue,
    // errors
    NotExists(T),
    LimitReached(T),
    HasUsages(OwnedSlice<Link<T>>),
    AlreadyExists(Doublet<T>),
    AllocFailed(Box<mem::Error>),
    Other(Box<OpaqueError>),
}

impl<T: LinkType> Display for DoubletsResult<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DoubletsResult::NotExists(exists) => {
                write!(f, "link {exists} does not exist.")
            }
            DoubletsResult::LimitReached(limit) => {
                write!(
                    f,
                    "limit for the number of links in the storage has been reached: {limit}"
                )
            }
            DoubletsResult::HasUsages(usages) => {
                write!(f, "link {usages:?} has dependencies")
            }
            DoubletsResult::AlreadyExists(exists) => {
                write!(f, "link {exists} already exists")
            }
            DoubletsResult::AllocFailed(alloc) => {
                write!(f, "unable to allocate memory for links storage: `{alloc}`")
            }
            DoubletsResult::Other(other) => {
                write!(f, "other internal error: `{other}`")
            }
            other @ _ => Debug::fmt(other, f),
        }
    }
}

use ffi_attributes as ffi;

#[ffi::specialize_for(
    types::<T>(
        u8  => u8,
        u16 => u16,
        u32 => u32,
        u64 => u64,
    ),
    name = "doublets_free_error_*",
    attributes(
        #[no_mangle]
    )
)]
pub extern "C" fn free_error<T: LinkType>(err: DoubletsResult<T>) {
    let _ = err;
}

unsafe fn write_raw_msg(buf: *mut c_char, size: c_short, msg: &str) {
    let cap = cmp::min(size as usize, msg.len()) - 1;
    ptr::copy_nonoverlapping(msg.as_ptr(), buf.cast(), cap);
    ptr::write(buf.add(cap), 0);
}

#[ffi::specialize_for(
    types::<T>(
        u8  => u8,
        u16 => u16,
        u32 => u32,
        u64 => u64,
    ),
    name = "doublets_read_error_message_*",
    attributes(
        #[no_mangle]
    )
)]
pub unsafe extern "C" fn read_error<T: LinkType>(
    buf: *mut c_char,
    size: c_short,
    error: &DoubletsResult<T>,
) {
    match error {
        /* invalid @ */
        DoubletsResult::Break | DoubletsResult::Continue => {
            warn!("`DoubletsResult` is expected to contain an error, got: `{error:?}`");
        }
        valid => {
            let msg = valid.to_string();
            let cap = cmp::min(size as usize, msg.len()) - 1;
            ptr::copy_nonoverlapping(msg.as_ptr(), buf.cast(), cap);
            ptr::write(buf.add(cap), 0);
        }
    }
}
