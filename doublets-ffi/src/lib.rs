#![cfg_attr(unstable_backtrace, feature(error_generic_member_access))]

pub mod constants;
pub mod errors;
pub mod export;
pub mod logging;
pub mod store;
mod utils;

pub(crate) use utils::stable_try;

// It is not useless: CLion highlight
// `c_char` as alias - italic
// `c_void` as type or alias - non-italic
#[allow(non_camel_case_types)]
type c_void = std::ffi::c_void;
#[allow(non_camel_case_types)]
type c_char = std::ffi::c_char;

pub type FFICallbackContext = *mut c_void;

/// [`Send`] and [`Sync`] wrapper on [`FFICallbackContext`]
///
/// WARNING: value of `FFICallbackContext` Context must be transferred across thread boundaries
/// and safe to share references between threads.
///
/// Otherwise value not use in multithreading context
#[derive(Clone, Copy)]
pub struct FFICallbackContextWrapper(FFICallbackContext);

/// Guarantee by caller side
unsafe impl Send for FFICallbackContextWrapper {}
unsafe impl Sync for FFICallbackContextWrapper {}
