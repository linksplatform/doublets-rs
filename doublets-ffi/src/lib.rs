use std::{error::Error, ffi::CStr, fmt::Display, fs::File, ops::RangeInclusive, ptr::null_mut};

use doublets::{
    data::{
        query,
        Flow::{Break, Continue},
        LinkReference, LinksConstants, Query, ToQuery,
    },
    mem::FileMapped,
    Link, Links,
};
use libc::c_char;
use log::{error, warn};

#[allow(non_camel_case_types)]
type c_void = core::ffi::c_void;

use doublets::{parts, unit, Doublets};
use ffi_attributes as ffi;

fn result_into_log<R, E: Display>(result: Result<R, E>, default: R) -> R {
    result.unwrap_or_else(|e| {
        error!("{e}");
        default
    })
}

unsafe fn query_from_raw<'a, T: LinkReference>(query: *const T, len: usize) -> Query<'a, T> {
    // it not require `#[cfg(debug_assertions)]`,
    // because it is used in debug log mode only (llvm optimization:))
    if query.is_null() && len != 0 {
        warn!("if `query` is null then `len` must be 0");
    }

    if query.is_null() {
        query![]
    } else {
        std::slice::from_raw_parts(query, len).to_query()
    }
}

unsafe fn unnull_or_error<'a, P, R>(ptr: *mut P) -> &'a mut R {
    if ptr.is_null() {
        // todo: use std::Backtrace or crates/tracing
        error!("Null pointer");
        panic!("Null pointer");
    } else {
        &mut *(ptr as *mut _)
    }
}

type UnitedLinks<T> = unit::Store<T, FileMapped<parts::LinkPart<T>>>;

// The opaque C handle owns one concrete store allocated by Box::into_raw.
type WrappedLinks<T> = UnitedLinks<T>;

type EachCallback<T> = extern "C" fn(Link<T>) -> T;

type CUDCallback<T> = extern "C" fn(Link<T>, Link<T>) -> T;

#[derive(Eq, PartialEq)]
#[repr(C)]
pub struct Range<T: LinkReference>(pub T, pub T);

#[repr(C)]
pub struct Constants<T: LinkReference> {
    pub index_part: T,
    pub source_part: T,
    pub target_part: T,
    pub null: T,
    pub r#continue: T,
    pub r#break: T,
    pub skip: T,
    pub any: T,
    pub itself: T,
    pub error: T,
    pub internal_range: Range<T>,
    pub external_range: Range<T>,
    pub _opt_marker: bool,
}

impl<T: LinkReference> From<LinksConstants<T>> for Constants<T> {
    fn from(c: LinksConstants<T>) -> Self {
        Self {
            index_part: c.index_part,
            source_part: c.source_part,
            target_part: c.target_part,
            null: c.null,
            r#continue: c.r#continue,
            r#break: c.r#break,
            skip: c.skip,
            any: c.any,
            itself: c.itself,
            error: c.error,
            internal_range: Range(*c.internal_range.start(), *c.internal_range.end()),
            // external_range: c.external_range.map(|r| Range(*r.start(), *r.end())),
            external_range: c
                .clone()
                .external_range
                .map_or(Range(T::from_byte(0), T::from_byte(0)), |r| {
                    Range(*r.start(), *r.end())
                }),
            _opt_marker: c.external_range.is_some(),
        }
    }
}

#[allow(clippy::from_over_into)]
impl<T: LinkReference> Into<LinksConstants<T>> for Constants<T> {
    fn into(self) -> LinksConstants<T> {
        LinksConstants {
            index_part: self.index_part,
            source_part: self.source_part,
            target_part: self.target_part,
            r#break: self.r#break,
            null: self.null,
            r#continue: self.r#continue,
            skip: self.skip,
            any: self.any,
            itself: self.itself,
            error: self.error,
            internal_range: RangeInclusive::new(self.internal_range.0, self.internal_range.1),
            external_range: if self._opt_marker {
                Some(RangeInclusive::new(
                    self.external_range.0,
                    self.external_range.1,
                ))
            } else {
                None
            },
        }
    }
}

#[ffi::specialize_for(
    types = "u8",
    types = "u16",
    types = "u32",
    types = "u64",
    convention = "csharp",
    name = "*Links_New"
)]
unsafe fn new_united_links<T: LinkReference>(path: *const c_char) -> *mut c_void {
    new_with_constants_united_links::<T>(path, LinksConstants::external().into())
}

#[ffi::specialize_for(
    types = "u8",
    types = "u16",
    types = "u32",
    types = "u64",
    convention = "csharp",
    name = "*Links_NewWithConstants"
)]
unsafe fn new_with_constants_united_links<T: LinkReference>(
    path: *const c_char,
    constants: Constants<T>,
) -> *mut c_void {
    let result: Result<_, Box<dyn Error>> = (|| {
        let path = CStr::from_ptr(path).to_str()?;
        let file = File::options()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(path)?;
        let mem = FileMapped::new(file)?;
        let links = UnitedLinks::<T>::with_constants(mem, constants.into())?;
        Ok(Box::into_raw(Box::new(links)).cast::<c_void>())
    })();
    result_into_log(result, null_mut())
}

#[ffi::specialize_for(
    types = "u8",
    types = "u16",
    types = "u32",
    types = "u64",
    convention = "csharp",
    name = "*Links_Drop"
)]
unsafe fn drop_united_links<T: LinkReference>(this: *mut c_void) {
    if !this.is_null() {
        // SAFETY: this is the pointer returned by new_with_constants_united_links,
        // with the same integer specialization, and ownership returns exactly once.
        drop(Box::from_raw(this.cast::<WrappedLinks<T>>()));
    }
}

#[ffi::specialize_for(
    types = "u8",
    types = "u16",
    types = "u32",
    types = "u64",
    convention = "csharp",
    name = "*Links_GetConstants"
)]
unsafe fn get_constants_united_links<T: LinkReference>(this: *mut c_void) -> Constants<T> {
    let links: &mut WrappedLinks<T> = unnull_or_error(this);
    links.constants().clone().into()
}

#[ffi::specialize_for(
    types = "u8",
    types = "u16",
    types = "u32",
    types = "u64",
    convention = "csharp",
    name = "*Links_Create"
)]
unsafe fn create_united<T: LinkReference>(
    this: *mut c_void,
    query: *const T,
    len: usize,
    callback: CUDCallback<T>,
) -> T {
    let links: &mut WrappedLinks<T> = unnull_or_error(this);
    let continue_ = links.constants().r#continue;
    let break_ = links.constants().r#break;
    let result = {
        let query = query_from_raw(query, len);
        let handler = |before: Link<_>, after: Link<_>| {
            if callback(before, after) == continue_ {
                Continue
            } else {
                Break
            }
        };
        links.create_by_with(query, handler)
    };
    result_into_log(
        result.map(|flow| if !flow.is_break() { continue_ } else { break_ }),
        links.constants().error,
    )
}

#[ffi::specialize_for(
    types = "u8",
    types = "u16",
    types = "u32",
    types = "u64",
    convention = "csharp",
    name = "*Links_SmartCreate"
)]
unsafe fn smart_create_united<T: LinkReference>(this: *mut c_void) -> T {
    let links: &mut WrappedLinks<T> = unnull_or_error(this);
    let result = links.create();
    result_into_log(result, links.constants().error)
}

#[ffi::specialize_for(
    types = "u8",
    types = "u16",
    types = "u32",
    types = "u64",
    convention = "csharp",
    name = "*Links_SmartUpdate"
)]
unsafe fn smart_update_united<T: LinkReference>(
    this: *mut c_void,
    index: T,
    source: T,
    target: T,
) -> T {
    let links: &mut WrappedLinks<T> = unnull_or_error(this);
    let result = links.update(index, source, target);
    result_into_log(result, links.constants().error)
}

#[ffi::specialize_for(
    types = "u8",
    types = "u16",
    types = "u32",
    types = "u64",
    convention = "csharp",
    name = "*Links_Each"
)]
unsafe fn each_united<T: LinkReference>(
    this: *mut c_void,
    query: *const T,
    len: usize,
    callback: EachCallback<T>,
) -> T {
    let links: &mut WrappedLinks<T> = unnull_or_error(this);
    let query = query_from_raw(query, len);
    let r#continue = links.constants().r#continue;
    let r#break = links.constants().r#break;
    let result = links.each_by(query, move |link| {
        if callback(link) == r#continue {
            Continue
        } else {
            Break
        }
    });
    match result {
        Continue => r#continue,
        Break => r#break,
    }
}

#[ffi::specialize_for(
    types = "u8",
    types = "u16",
    types = "u32",
    types = "u64",
    convention = "csharp",
    name = "*Links_Count"
)]
unsafe fn count_united<T: LinkReference>(this: *mut c_void, query: *const T, len: usize) -> T {
    let links: &mut WrappedLinks<T> = unnull_or_error(this);
    let query = query_from_raw(query, len);
    links.count_by(query)
}

#[ffi::specialize_for(
    types = "u8",
    types = "u16",
    types = "u32",
    types = "u64",
    convention = "csharp",
    name = "*Links_Update"
)]
unsafe fn update_united<T: LinkReference>(
    this: *mut c_void,
    restrictions: *const T,
    len_r: usize,
    substitutuion: *const T,
    len_s: usize,
    callback: CUDCallback<T>,
) -> T {
    let restrictions = query_from_raw(restrictions, len_r);
    let substitutuion = query_from_raw(substitutuion, len_s);
    let links: &mut WrappedLinks<T> = unnull_or_error(this);
    let continue_ = links.constants().r#continue;
    let break_ = links.constants().r#break;
    let result = {
        let handler = move |before: Link<T>, after: Link<T>| {
            if callback(before, after) == continue_ {
                Continue
            } else {
                Break
            }
        };
        links.update_by_with(restrictions, substitutuion, handler)
    };
    result_into_log(
        result.map(|flow| if !flow.is_break() { continue_ } else { break_ }),
        links.constants().error,
    )
}

#[ffi::specialize_for(
    types = "u8",
    types = "u16",
    types = "u32",
    types = "u64",
    convention = "csharp",
    name = "*Links_Delete"
)]
unsafe fn delete_united<T: LinkReference>(
    this: *mut c_void,
    query: *const T,
    len: usize,
    callback: CUDCallback<T>,
) -> T {
    let query = query_from_raw(query, len);
    let links: &mut WrappedLinks<T> = unnull_or_error(this);
    let continue_ = links.constants().r#continue;
    let break_ = links.constants().r#break;
    let result = {
        let handler = move |before: Link<_>, after: Link<_>| {
            if callback(before, after) == break_ {
                Break
            } else {
                Continue
            }
        };
        links.delete_by_with(query, handler)
    };
    result_into_log(
        result.map(|flow| if !flow.is_break() { continue_ } else { break_ }),
        links.constants().error,
    )
}

#[repr(C)]
pub struct SharedLogger {
    formatter: for<'a> extern "C" fn(&'a log::Record<'_>),
}

impl log::Log for SharedLogger {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }
    fn log(&self, record: &log::Record) {
        (self.formatter)(record)
    }
    fn flush(&self) {}
}

pub fn build_shared_logger() -> SharedLogger {
    extern "C" fn formatter(r: &log::Record<'_>) {
        tracing_log::format_trace(r).unwrap()
    }
    SharedLogger { formatter }
}

#[no_mangle]
pub extern "C" fn setup_shared_logger(logger: SharedLogger) {
    log::set_max_level(log::STATIC_MAX_LEVEL);

    let subscriber = tracing_subscriber::fmt::fmt()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    if let Err(err) = tracing::subscriber::set_global_default(subscriber) {
        warn!("subscriber error: {}", err)
    }

    if let Err(err) = log::set_boxed_logger(Box::new(logger)) {
        warn!("{}", err)
    }
}

#[no_mangle]
pub extern "C" fn init_fmt_logger() {
    let logger = build_shared_logger();
    setup_shared_logger(logger);
}

#[cfg(test)]
mod tests;
