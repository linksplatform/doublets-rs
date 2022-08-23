use crate::c_char;
use core::ffi::c_size_t;
use doublets::{data::LinkType, Doublet, Error, Link};
use std::{cell::RefCell, cmp, error, ffi::c_short, mem::MaybeUninit, ptr};

#[repr(u8)]
pub enum DoubletsResultKind {
    // oks
    Break,
    Continue,
    // errors 
    NotExists,
    HasUsages,
    AlreadyExists,
    LimitReached,
    AllocFailed,
    Other,
}

#[thread_local]
static ERROR_PLACE: RefCell<MaybeUninit<Error<usize>>> = RefCell::new(MaybeUninit::uninit());

fn link_cast<T: LinkType>(
    Link {
        index,
        source,
        target,
    }: Link<T>,
) -> Link<usize> {
    Link::new(index.as_usize(), source.as_usize(), target.as_usize())
}

pub(crate) fn place_error<T: LinkType>(error: Error<T>) {
    use doublets::Error::*;

    ERROR_PLACE.borrow_mut().write(match error {
        NotExists(link) => NotExists(link.as_usize()),
        HasUsages(usages) => HasUsages(usages.into_iter().map(link_cast).collect()),
        AlreadyExists(Doublet { source, target }) => {
            AlreadyExists(Doublet::new(source.as_usize(), target.as_usize()))
        }
        LimitReached(limit) => LimitReached(limit.as_usize()),
        AllocFailed(alloc) => AllocFailed(alloc),
        Other(other) => Other(other),
    });
}

unsafe fn write_raw_msg(buf: *mut c_char, size: c_short, msg: &str) {
    let cap = cmp::min(size as usize, msg.len()) - 1;
    ptr::copy_nonoverlapping(msg.as_ptr(), buf.cast(), cap);
    ptr::write(buf.add(cap), 0);
}

#[no_mangle]
pub extern "C" fn doublets_read_error() -> *const Error<usize> {
    ERROR_PLACE.borrow().as_ptr()
}

#[no_mangle]
pub unsafe extern "C" fn doublets_read_error_message(
    buf: *mut c_char,
    size: c_short,
    err: *const Error<usize>,
) {
    let error_msg = format!("{}", &*err);
    write_raw_msg(buf, size, &error_msg);
}

#[no_mangle]
pub unsafe extern "C" fn doublets_read_error_backtrace(
    buf: *mut c_char,
    size: c_short,
    err: *const Error<usize>,
) {
    let error_msg = format!("{:?}", error::Error::source(&*err));
    write_raw_msg(buf, size, &error_msg);
}

#[no_mangle]
pub unsafe extern "C" fn doublets_read_error_as_not_found(err: *const Error<usize>) -> c_size_t {
    match &*err {
        Error::NotExists(link) => *link as c_size_t,
        _ => panic!("error type is not `NotExists`"),
    }
}

#[no_mangle]
pub unsafe extern "C" fn doublets_read_error_as_already_exists(
    err: *const Error<usize>,
) -> Doublet<c_size_t> {
    match &*err {
        Error::AlreadyExists(Doublet { source, target }) => {
            Doublet::new(*source as c_size_t, *target as c_size_t)
        }
        _ => panic!("error type is not `AlreadyExists`"),
    }
}

#[no_mangle]
pub unsafe extern "C" fn doublets_read_error_as_limit_reached(
    err: *const Error<usize>,
) -> c_size_t {
    match &*err {
        Error::LimitReached(limit) => *limit as c_size_t,
        _ => panic!("error type is not `LimitReached`"),
    }
}
