use crate::c_char;
use doublets::{data::LinkType, mem, Doublet, Link};
#[cfg(unstable_backtrace)]
use std::backtrace::Backtrace;
use std::{
    cmp, error,
    ffi::c_short,
    fmt::{self, Debug, Display, Formatter},
    ptr,
};
use tracing::warn;

type OpaqueError = Box<dyn error::Error>;

#[repr(C, usize)]
#[derive(Debug)]
pub enum DoubletsError<T: LinkType> {
    NotExists(T),
    LimitReached(T),
    HasUsages(OwnedSlice<Link<T>>),
    AlreadyExists(Doublet<T>),
    AllocFailed(Box<mem::Error>),
    Other(Box<OpaqueError>),
}

impl<T: LinkType> DoubletsError<T> {
    #[cfg(unstable_backtrace)]
    fn backtrace(&self) -> Option<&Backtrace> {
        fn erasure(err: &mem::Error) -> &dyn error::Error {
            err as _
        }

        match self {
            DoubletsError::AllocFailed(err) => erasure(err).request_ref(),
            DoubletsError::Other(err) => err.request_ref(),
            _ => None,
        }
    }
}

impl<T: LinkType> Display for DoubletsError<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DoubletsError::NotExists(exists) => {
                write!(f, "link {exists} does not exist.")
            }
            DoubletsError::LimitReached(limit) => {
                write!(f, "links limit in storage has been reached: {limit}")
            }
            DoubletsError::HasUsages(usages) => {
                write!(f, "link {usages:?} has dependencies")
            }
            DoubletsError::AlreadyExists(exists) => {
                write!(f, "link {exists} already exists")
            }
            DoubletsError::AllocFailed(alloc) => {
                write!(f, "alloc memory error: `{alloc}`")
            }
            DoubletsError::Other(other) => {
                write!(f, "other internal error: `{other}`")
            }
        }
    }
}

use crate::utils::OwnedSlice;
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
pub extern "C" fn free_error<T: LinkType>(err: DoubletsError<T>) {
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
    error: &DoubletsError<T>,
) {
    write_raw_msg(buf, size, &error.to_string());
}

#[cfg(unstable_backtrace)]
#[ffi::specialize_for(
    types::<T>(
        u8  => u8,
        u16 => u16,
        u32 => u32,
        u64 => u64,
    ),
    name = "doublets_read_backtrace_*",
        attributes(
        #[no_mangle]
    )
)]
pub unsafe extern "C" fn read_backtrace<T: LinkType>(
    buf: *mut c_char,
    size: c_short,
    error: &DoubletsError<T>,
) {
    if let Some(backtrace) = error.backtrace() {
        write_raw_msg(buf, size, &backtrace.to_string());
    }
}
