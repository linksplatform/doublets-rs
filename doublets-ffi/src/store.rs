#![allow(clippy::missing_safety_doc)]

use crate::{c_char, c_void, constants::Constants, errors::DoubletsResultKind, FFICallbackContext};
use doublets::{
    data::{query, Flow, LinkType, Query, ToQuery},
    mem::FileMapped,
    parts, unit, Doublets, Error, Link, Links,
};
use ffi_attributes as ffi;
use std::{ffi::CStr, marker::PhantomData, ptr::NonNull, slice};
use tap::Pipe;
use tracing::{debug, warn};

// TODO: remove ::mem:: in doublets crate
type UnitedLinks<T> = unit::Store<T, FileMapped<parts::LinkPart<T>>>;

type EachCallback<T> = extern "C" fn(FFICallbackContext, Link<T>) -> Flow;

type CUDCallback<T> = extern "C" fn(FFICallbackContext, Link<T>, Link<T>) -> Flow;

#[repr(transparent)]
pub struct StoreHandle<T: LinkType> {
    pub(crate) ptr: NonNull<c_void>, // thin ptr to dyn Doublets<T>
    marker: PhantomData<T>,
}

impl<T: LinkType> StoreHandle<T> {
    pub fn new(store: Box<dyn Doublets<T>>) -> Self {
        let raw = Box::into_raw(Box::new(store));
        // SAFETY: box contains valid ptr to store
        unsafe { Self::from_raw(raw.cast()) }
    }

    /// # Examples
    ///
    /// Safe usage:
    ///
    /// ```
    /// # use std::ffi::c_void;
    /// # use doublets_ffi::store::StoreHandle;
    /// extern "C" fn create_u64_store() -> *mut c_void {
    ///     todo!("todo: simple but full example")
    /// }
    ///
    /// // SAFETY: caller must guarantee `from_raw` invariants
    /// unsafe extern "C" fn free_u64_store(ptr: *mut c_void) {
    ///     StoreHandle::drop(StoreHandle::<u64>::from_raw(ptr))
    /// }
    /// ```
    ///
    /// Undefined Behaviour usage:
    /// ```no_run
    /// # use std::ffi::c_void;
    /// # use doublets_ffi::store::StoreHandle;
    ///
    /// unsafe extern "C" fn should_crush(ptr: *mut c_void) {
    ///     // two handle for one store is safe
    ///     let (mut a, mut b) = (
    ///         StoreHandle::<u64>::from_raw(ptr),
    ///         StoreHandle::<u64>::from_raw(ptr),
    ///     );
    ///     // but it is ub
    ///     let (a, b) = (a.assume(), b.assume());
    /// }
    /// ```
    ///
    /// # Safety
    /// `raw` must be valid ptr to `Box<dyn Doublets<T>>`
    /// allocated in `Box`
    /// without owner
    pub unsafe fn from_raw(raw: *mut c_void) -> StoreHandle<T> {
        debug_assert!(!raw.is_null());

        Self {
            ptr: NonNull::new_unchecked(raw),
            marker: PhantomData,
        }
    }

    /// # Safety
    /// should not live more than what is allowed
    pub unsafe fn from_raw_assume<'a>(raw: *mut c_void) -> &'a mut Box<dyn Doublets<T>> {
        let leak = Self::from_raw(raw);
        // SAFETY: Guarantee by caller
        leak.ptr.cast().as_mut()
    }

    pub fn assume(&mut self) -> &mut Box<dyn Doublets<T>> {
        // SAFETY: `StoreHandle` must be create from safe `new()`
        // or unsafe `Self::from_raw`
        // then it guarantee by `Self::from_raw()` caller
        unsafe { self.ptr.cast().as_mut() }
    }

    pub fn invalid(err: Error<T>) -> Self {
        acquire_error(err);
        // we not have access to self inner
        Self {
            ptr: NonNull::dangling(),
            marker: PhantomData,
        }
    }

    pub fn as_ptr(&self) -> *const c_void {
        self.ptr.as_ptr()
    }

    pub fn drop(mut handle: Self) {
        // SAFETY: `self.store` is valid `Store` ptr
        unsafe {
            let _ = Box::from_raw(handle.assume());
        }
    }
}

unsafe fn thin_query_from_raw<'a, T: LinkType>(query: *const T, len: u32) -> Query<'a, T> {
    // fixme: may be use `assert!(!query.is_null())`
    if query.is_null() {
        query![]
    } else {
        slice::from_raw_parts(query, len as usize).to_query()
    }
}

unsafe fn query_from_raw<'a, T: LinkType>(query: *const T, len: u32) -> Query<'a, T> {
    if query.is_null() && len != 0 {
        warn!("query ptr is null, but len is not null: this could be a potential mistake.");
    }

    thin_query_from_raw(query, len)
}

fn place_error<T: LinkType>(err: Error<T>) {
    // It can be very expensive to handle each error
    debug!(op_error = % err);
    super::errors::place_error(err);
}

impl DoubletsResultKind {
    pub fn branch(flow: Flow) -> Self {
        if let Flow::Continue = flow {
            DoubletsResultKind::Continue
        } else {
            DoubletsResultKind::Break
        }
    }

    pub fn leak<T: LinkType>(err: &Error<T>) -> Self {
        match err {
            Error::NotExists(_) => DoubletsResultKind::NotExists,
            Error::HasUsages(_) => DoubletsResultKind::HasUsages,
            Error::AlreadyExists(_) => DoubletsResultKind::AlreadyExists,
            Error::LimitReached(_) => DoubletsResultKind::LimitReached,
            Error::AllocFailed(_) => DoubletsResultKind::AllocFailed,
            Error::Other(_) => DoubletsResultKind::Other,
        }
    }
}

fn acquire_error<T: LinkType>(err: Error<T>) -> DoubletsResultKind {
    let ret = DoubletsResultKind::leak(&err);
    place_error(err);
    ret
}

fn acquire_result<T: LinkType>(result: Result<Flow, Error<T>>) -> DoubletsResultKind {
    match result {
        Ok(flow) => DoubletsResultKind::branch(flow),
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
)]
pub unsafe fn create_unit_store<T: LinkType>(
    path: *const c_char,
    constants: Constants<T>,
) -> StoreHandle<T> {
    let result: Result<_, Error<T>> = try {
        let path = CStr::from_ptr(path).to_str().unwrap();
        let mem = FileMapped::from_path(path)?;
        StoreHandle::new(Box::new(UnitedLinks::with_constants(
            mem,
            constants.into(),
        )?))
    };
    result.unwrap_or_else(StoreHandle::invalid)
}

#[ffi::specialize_for(
    types::<T>(
        u8  => u8,
        u16 => u16,
        u32 => u32,
        u64 => u64,
    ),
    name = "doublets_free_store_*",
)]
pub unsafe fn free_store<T: LinkType>(this: *mut c_void) {
    StoreHandle::drop(StoreHandle::<T>::from_raw(this))
}

#[ffi::specialize_for(
    types::<T>(
        u8  => u8,
        u16 => u16,
        u32 => u32,
        u64 => u64,
    )
    name = "doublets_constants_*"
)]
pub unsafe fn constants_for_store<T: LinkType>(this: *mut c_void) -> Constants<T> {
    StoreHandle::from_raw(this)
        .assume()
        .constants()
        .clone() // fixme: useless .clone
        .into()
}

#[tracing::instrument(
    skip_all,
    fields(
        query = ?&thin_query_from_raw(query, len)[..],
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
)]
pub unsafe fn create<T: LinkType>(
    this: *mut c_void,
    query: *const T,
    len: u32,
    ctx: FFICallbackContext,
    callback: CUDCallback<T>,
) -> DoubletsResultKind {
    let query = query_from_raw(query, len);
    let store = StoreHandle::<T>::from_raw_assume(this);
    let handler = move |before, after| callback(ctx, before, after);
    store.create_by_with(query, handler).pipe(acquire_result)
}

#[tracing::instrument(
    skip_all,
    fields(
        query = ?&thin_query_from_raw(query, len)[..],
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
)]
pub unsafe fn each<T: LinkType>(
    this: *mut c_void,
    query: *const T,
    len: u32,
    ctx: FFICallbackContext,
    callback: EachCallback<T>,
) -> DoubletsResultKind {
    let query = query_from_raw(query, len);
    let store = StoreHandle::<T>::from_raw_assume(this);
    let handler = move |link| callback(ctx, link);
    store
        .each_by(query, handler)
        .pipe(DoubletsResultKind::branch)
}

#[tracing::instrument(
    skip_all,
    fields(
        query = ?&thin_query_from_raw(query, len)[..],
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
)]
pub unsafe fn count<T: LinkType>(this: *mut c_void, query: *const T, len: u32) -> T {
    let mut handle = StoreHandle::<T>::from_raw(this);
    let query = query_from_raw(query, len);
    handle.assume().count_by(query)
}

#[tracing::instrument(
    skip_all,
    fields(
        query = ?&thin_query_from_raw(query, len_q)[..],
        query.ptr = ?query,
        query.len = len_q,

        change = ?&thin_query_from_raw(query, len_q)[..],
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
)]
pub unsafe fn update<T: LinkType>(
    this: *mut c_void,
    query: *const T,
    len_q: u32,
    change: *const T,
    len_c: u32,
    ctx: FFICallbackContext,
    callback: CUDCallback<T>,
) -> DoubletsResultKind {
    let query = query_from_raw(query, len_q);
    let change = query_from_raw(change, len_c);
    let store = StoreHandle::<T>::from_raw_assume(this);
    let handler = move |before, after| callback(ctx, before, after);
    store
        .update_by_with(query, change, handler)
        .pipe(acquire_result)
}

#[tracing::instrument(
    skip_all,
    fields(
        query = ?&thin_query_from_raw(query, len)[..],
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
)]
pub unsafe fn delete<T: LinkType>(
    this: *mut c_void,
    query: *const T,
    len: u32,
    ctx: FFICallbackContext,
    callback: CUDCallback<T>,
) -> DoubletsResultKind {
    let query = query_from_raw(query, len);
    let store = StoreHandle::<T>::from_raw_assume(this);
    let handler = move |before, after| callback(ctx, before, after);
    store.delete_by_with(query, handler).pipe(acquire_result)
}
