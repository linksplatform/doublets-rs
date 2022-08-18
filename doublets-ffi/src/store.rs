use crate::{c_char, c_void, constants::Constants, errors::DoubletsErrorKind, FFICallbackContext};
use doublets::{
    data::{query, Flow, LinkType, Query, ToQuery},
    mem::FileMapped,
    parts, unit, Doublets, Error, Link, Links,
};
use ffi_attributes as ffi;
use std::{error, ffi::CStr, marker::PhantomData, ptr, slice};
use tap::Pipe;
use tracing::{debug, error, warn};

// TODO: remove ::mem:: in doublets crate
type UnitedLinks<T> = unit::Store<T, FileMapped<parts::LinkPart<T>>>;

type EachCallback<T> = extern "C" fn(FFICallbackContext, Link<T>) -> Flow;

type CUDCallback<T> = extern "C" fn(FFICallbackContext, Link<T>, Link<T>) -> Flow;

#[repr(transparent)]
pub struct StoreHandle<T: LinkType> {
    pub(crate) ptr: *mut c_void, // dyn Doublets<T>
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
        Self {
            ptr: raw,
            marker: PhantomData,
        }
    }

    /// # Safety
    /// should not live more than what is allowed
    pub unsafe fn from_raw_assume<'a>(raw: *mut c_void) -> &'a mut Box<dyn Doublets<T>> {
        let leak = Self::from_raw(raw);
        // SAFETY: Guarantee by caller
        &mut *leak.ptr.cast()
    }

    pub fn assume(&mut self) -> &mut Box<dyn Doublets<T>> {
        // SAFETY: `StoreHandle` must be create from safe `new()`
        // or unsafe `Self::from_raw`
        // then it guarantee by `Self::from_raw()` caller
        unsafe { &mut *self.ptr.cast() }
    }

    pub fn invalid(err: Box<dyn error::Error>) -> Self {
        error!(err);
        // we not have access to self inner
        Self {
            ptr: ptr::null_mut(),
            marker: PhantomData,
        }
    }

    pub fn drop(mut handle: Self) {
        // SAFETY: `self.store` is valid `Store` ptr
        unsafe {
            // assume ptr is not null
            // guarantee by `from_raw` and `new`
            if !handle.ptr.is_null() {
                let _ = Box::from_raw(handle.assume());
            }
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
    // it not require `#[cfg(debug_assertions)]`,
    // because it is used in debug log mode only (llvm optimization:))
    if query.is_null() && len != 0 {
        warn!("if `query` is null then `len` must be 0");
    }

    thin_query_from_raw(query, len)
}

fn place_error<T: LinkType>(err: Error<T>) {
    // It can be very expensive to handle each error
    debug!(op_error = % err);
    super::errors::place_error(err);
}

impl DoubletsErrorKind {
    pub fn leak<T: LinkType>(err: &Error<T>) -> Self {
        match err {
            Error::NotExists(_) => DoubletsErrorKind::NotExists,
            Error::HasUsages(_) => DoubletsErrorKind::HasUsages,
            Error::AlreadyExists(_) => DoubletsErrorKind::AlreadyExists,
            Error::LimitReached(_) => DoubletsErrorKind::LimitReached,
            Error::AllocFailed(_) => DoubletsErrorKind::AllocFailed,
            Error::Other(_) => DoubletsErrorKind::Other,
        }
    }
}

fn acquire_result<T: LinkType>(result: Result<Flow, Error<T>>) -> DoubletsErrorKind {
    match result {
        Ok(_) => DoubletsErrorKind::None,
        Err(err) => {
            let ret = DoubletsErrorKind::leak(&err);
            place_error(err);
            ret
        }
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
    types = "u8",
    types = "u16",
    types = "u32",
    types = "u64",
    convention = "rust",
    name = "doublets_create_united_store_*"
)]
pub unsafe fn create_unit_store<T: LinkType>(
    path: *const c_char,
    constants: Constants<T>,
) -> StoreHandle<T> {
    let result: Result<_, Box<dyn error::Error>> = try {
        let path = CStr::from_ptr(path).to_str().unwrap();
        let mem = FileMapped::from_path(path)?;
        StoreHandle::new(Box::new(UnitedLinks::<T>::with_constants(
            mem,
            constants.into(),
        )?))
    };
    result.unwrap_or_else(StoreHandle::invalid)
}

#[ffi::specialize_for(
    types = "u8",
    types = "u16",
    types = "u32",
    types = "u64",
    convention = "rust",
    name = "doublets_free_store_*"
)]
pub unsafe fn free_store<T: LinkType>(this: *mut c_void) {
    StoreHandle::drop(StoreHandle::<T>::from_raw(this))
}

#[ffi::specialize_for(
    types = "u8",
    types = "u16",
    types = "u32",
    types = "u64",
    convention = "rust",
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
    types = "u8",
    types = "u16",
    types = "u32",
    types = "u64",
    convention = "rust",
    name = "doublets_create_*"
)]
pub unsafe fn create<T: LinkType>(
    this: *mut c_void,
    query: *const T,
    len: u32,
    ctx: FFICallbackContext,
    callback: CUDCallback<T>,
) -> DoubletsErrorKind {
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
    types = "u8",
    types = "u16",
    types = "u32",
    types = "u64",
    convention = "rust",
    name = "doublets_each_*"
)]
pub unsafe fn each<T: LinkType>(
    this: *mut c_void,
    query: *const T,
    len: u32,
    ctx: FFICallbackContext,
    callback: EachCallback<T>,
) -> T {
    let mut handle = StoreHandle::<T>::from_raw(this);
    let store = handle.assume();
    let constants = store.constants();
    let (cnt, brk) = (constants.r#continue, constants.r#break);

    let query = query_from_raw(query, len);
    let handler = move |link| callback(ctx, link);
    store
        .each_by(query, handler)
        // fixme: add `.is_break` for `Flow`
        .pipe(|flow| if let Flow::Continue = flow { cnt } else { brk })
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
    types = "u8",
    types = "u16",
    types = "u32",
    types = "u64",
    convention = "rust",
    name = "doublets_count_*"
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
    types = "u8",
    types = "u16",
    types = "u32",
    types = "u64",
    convention = "rust",
    name = "doublets_update_*"
)]
pub unsafe fn update<T: LinkType>(
    this: *mut c_void,
    query: *const T,
    len_q: u32,
    change: *const T,
    len_c: u32,
    ctx: FFICallbackContext,
    callback: CUDCallback<T>,
) -> DoubletsErrorKind {
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
    types = "u8",
    types = "u16",
    types = "u32",
    types = "u64",
    convention = "rust",
    name = "doublets_delete_*"
)]
pub unsafe fn delete<T: LinkType>(
    this: *mut c_void,
    query: *const T,
    len: u32,
    ctx: FFICallbackContext,
    callback: CUDCallback<T>,
) -> DoubletsErrorKind {
    let query = query_from_raw(query, len);
    let store = StoreHandle::<T>::from_raw_assume(this);
    let handler = move |before, after| callback(ctx, before, after);
    store.delete_by_with(query, handler).pipe(acquire_result)
}
