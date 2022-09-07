#![allow(clippy::missing_safety_doc)]

use crate::{
    c_char,
    constants::Constants,
    errors::{DoubletsResult, OwnedSlice},
    stable_try as tri,
    utils::Maybe,
    FFICallbackContext,
};
use doublets::{
    data::{Flow, LinkType},
    mem::FileMapped,
    parts, unit, Doublets, Error, Link, Links,
};
use ffi_attributes as ffi;
use std::{
    ffi::CStr,
    fmt::{self, Debug, Formatter},
    slice,
};
use tap::Pipe;
use tracing::{debug, warn};

type UnitedLinks<T> = unit::Store<T, FileMapped<parts::LinkPart<T>>>;

type EachCallback<T> = extern "C" fn(FFICallbackContext, Link<T>) -> Flow;

type CUDCallback<T> = extern "C" fn(FFICallbackContext, Link<T>, Link<T>) -> Flow;

pub struct StoreHandle<T: LinkType> {
    pointer: Box<dyn Doublets<T>>,
}

impl<T: LinkType> Debug for StoreHandle<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("StoreHandle")
            .field("pointer", &(self.pointer.as_ref() as *const _))
            .finish()
    }
}

impl<T: LinkType> StoreHandle<T> {
    pub fn new(store: Box<dyn Doublets<T>>) -> Self {
        Self { pointer: store }
    }

    pub unsafe fn assume(&mut self) -> &mut Box<dyn Doublets<T>> {
        &mut self.pointer
    }

    #[allow(clippy::borrowed_box)] // needs for `Self: Sized` also can use `&impl Doublets<T>`
    pub unsafe fn assume_ref(&self) -> &Box<dyn Doublets<T>> {
        &self.pointer
    }

    /// This function is actually unsafe
    ///
    /// # Safety
    ///
    /// Caller guarantee that will not drop handle
    // fixme: may be we can port `result::Result` to C
    pub fn invalid<Any>(err: Error<T>) -> Maybe<Any> {
        acquire_error(err);

        Maybe::none()
    }
}

unsafe fn thin_query_from_raw<'a, T>(query: *const T, len: u32) -> &'a [T] {
    if query.is_null() {
        &[]
    } else {
        slice::from_raw_parts(query, len as usize)
    }
}

unsafe fn query_from_raw<'a, T>(query: *const T, len: u32) -> &'a [T] {
    if query.is_null() && len != 0 {
        warn!("query ptr is null, but len is not null: handle could be a potential mistake.");
    }

    thin_query_from_raw(query, len)
}

impl<T: LinkType> DoubletsResult<T> {
    pub fn from_branch(flow: Flow) -> Self {
        if let Flow::Continue = flow {
            DoubletsResult::Continue
        } else {
            DoubletsResult::Break
        }
    }

    pub fn from_err(err: Error<T>) -> Self {
        match err {
            Error::NotExists(link) => Self::NotExists(link),
            Error::HasUsages(usages) => Self::HasUsages(OwnedSlice::leak(usages)),
            Error::AlreadyExists(exists) => Self::AlreadyExists(exists),
            Error::LimitReached(limit) => Self::LimitReached(limit),
            // these errors are difficult to handle as data
            // I hope no one will be offended if we alloc them at the heap
            Error::AllocFailed(alloc) => Self::AllocFailed(Box::new(alloc)),
            Error::Other(other) => Self::Other(Box::new(other)),
        }
    }

    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Break | Self::Continue | Self::Handle(_))
    }

    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }
}

impl<T: LinkType> From<Flow> for DoubletsResult<T> {
    fn from(flow: Flow) -> Self {
        Self::from_branch(flow)
    }
}

impl<T: LinkType> From<Box<StoreHandle<T>>> for DoubletsResult<T> {
    fn from(handle: Box<StoreHandle<T>>) -> Self {
        Self::Handle(handle)
    }
}

fn acquire_error<T: LinkType>(err: Error<T>) -> DoubletsResult<T> {
    // It can be very expensive to handle each error
    debug!(op_error = % err);
    DoubletsResult::from_err(err)
}

fn acquire_result<Ok, T>(result: Result<Ok, Error<T>>) -> DoubletsResult<T>
where
    T: LinkType,
    Ok: Into<DoubletsResult<T>>,
{
    match result {
        Ok(ok) => ok.into(),
        Err(err) => acquire_error(err),
    }
}

#[tracing::instrument(
    skip_all,
    fields(
        path = ?CStr::from_ptr(path).to_str(),
        path.ptr = ?path,
    ),
)]
#[ffi::specialize_for(
    types::<T>(
        u8  => u8,
        u16 => u16,
        u32 => u32,
        u64 => u64,
    ),
    name = "doublets_create_united_store_*",
    attributes(
        #[no_mangle]
    )
)]
pub unsafe extern "C" fn create_unit_store<T: LinkType>(
    path: *const c_char,
    constants: Constants<T>,
) -> DoubletsResult<T> {
    let result: Result<_, Error<T>> = tri! {
        let path = CStr::from_ptr(path).to_str().unwrap();
        let mem = FileMapped::from_path(path)?;
        StoreHandle::new(Box::new(UnitedLinks::with_constants(
            mem,
            constants.into(),
        )?))
    };
    result.map(Box::new).pipe(acquire_result)
}

#[ffi::specialize_for(
    types::<T>(
        u8  => u8,
        u16 => u16,
        u32 => u32,
        u64 => u64,
    ),
    name = "doublets_free_store_*",
    attributes(
        #[no_mangle]
    )
)]
pub unsafe extern "C" fn free_store<T: LinkType>(handle: Box<StoreHandle<T>>) {
    let _ = handle;
}

#[ffi::specialize_for(
    types::<T>(
        u8  => u8,
        u16 => u16,
        u32 => u32,
        u64 => u64,
    ),
    name = "doublets_constants_*",
    attributes(
        #[no_mangle]
    )
)]
pub unsafe extern "C" fn constants_from_store<T: LinkType>(
    handle: &StoreHandle<T>,
) -> Constants<T> {
    handle
        .assume_ref()
        .constants()
        .clone() // fixme: useless .clone
        .into()
}

#[tracing::instrument(
    skip_all,
    fields(
        query = ?thin_query_from_raw(query, len),
        query.ptr = ?query,
        query.len = len,
    ),
)]
#[ffi::specialize_for(
    types::<T>(
        u8  => u8,
        u16 => u16,
        u32 => u32,
        u64 => u64,
    ),
    name = "doublets_create_*",
    attributes(
        #[no_mangle]
    )
)]
pub unsafe extern "C" fn create<T: LinkType>(
    handle: &mut StoreHandle<T>,
    query: *const T,
    len: u32,
    ctx: FFICallbackContext,
    callback: CUDCallback<T>,
) -> DoubletsResult<T> {
    let query = query_from_raw(query, len);
    let handler = move |before, after| callback(ctx, before, after);
    handle
        .assume()
        .create_by_with(query, handler)
        .pipe(acquire_result)
}

#[tracing::instrument(
    skip_all,
    fields(
        query = ?thin_query_from_raw(query, len),
        query.ptr = ?query,
        query.len = len,
    ),
)]
#[ffi::specialize_for(
    types::<T>(
        u8  => u8,
        u16 => u16,
        u32 => u32,
        u64 => u64,
    ),
    name = "doublets_each_*",
    attributes(
        #[no_mangle]
    )
)]
pub unsafe extern "C" fn each<T: LinkType>(
    handle: &StoreHandle<T>,
    query: *const T,
    len: u32,
    ctx: FFICallbackContext,
    callback: EachCallback<T>,
) -> DoubletsResult<T> {
    let query = query_from_raw(query, len);
    let handler = move |link| callback(ctx, link);
    handle
        .assume_ref()
        .each_by(query, handler)
        .pipe(DoubletsResult::from_branch)
}

#[tracing::instrument(
    skip_all,
    fields(
        query = ?thin_query_from_raw(query, len),
        query.ptr = ?query,
        query.len = len,
    ),
)]
#[ffi::specialize_for(
    types::<T>(
        u8  => u8,
        u16 => u16,
        u32 => u32,
        u64 => u64,
    ),
    name = "doublets_count_*",
    attributes(
        #[no_mangle]
    )
)]
pub unsafe extern "C" fn count<T: LinkType>(
    handle: &mut StoreHandle<T>,
    query: *const T,
    len: u32,
) -> T {
    let query = query_from_raw(query, len);
    handle.assume().count_by(query)
}

#[tracing::instrument(
    skip_all,
    fields(
        query = ?thin_query_from_raw(query, len_q),
        query.ptr = ?query,
        query.len = len_q,

        change = ?thin_query_from_raw(query, len_q),
        change.ptr = ?change,
        change.len = len_c,
    ),
)]
#[ffi::specialize_for(
    types::<T>(
        u8  => u8,
        u16 => u16,
        u32 => u32,
        u64 => u64,
    ),
    name = "doublets_update_*",
    attributes(
        #[no_mangle]
    )
)]
pub unsafe extern "C" fn update<T: LinkType>(
    handle: &mut StoreHandle<T>,
    query: *const T,
    len_q: u32,
    change: *const T,
    len_c: u32,
    ctx: FFICallbackContext,
    callback: CUDCallback<T>,
) -> DoubletsResult<T> {
    let handler = move |before, after| callback(ctx, before, after);
    let query = query_from_raw(query, len_q);
    let change = query_from_raw(change, len_c);
    handle
        .assume()
        .update_by_with(query, change, handler)
        .pipe(acquire_result)
}

#[tracing::instrument(
    skip_all,
    fields(
        query = ?thin_query_from_raw(query, len),
        query.ptr = ?query,
        query.len = len,
    )
)]
#[ffi::specialize_for(
    types::<T>(
        u8  => u8,
        u16 => u16,
        u32 => u32,
        u64 => u64,
    ),
    name = "doublets_delete_*",
    attributes(
        #[no_mangle]
    )
)]
pub unsafe extern "C" fn delete<T: LinkType>(
    handle: &mut StoreHandle<T>,
    query: *const T,
    len: u32,
    ctx: FFICallbackContext,
    callback: CUDCallback<T>,
) -> DoubletsResult<T> {
    let handler = move |before, after| callback(ctx, before, after);
    let query = query_from_raw(query, len);
    handle
        .assume()
        .delete_by_with(query, handler)
        .pipe(acquire_result)
}
