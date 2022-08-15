#![feature(try_blocks)]
#![feature(box_syntax)]
#![feature(try_trait_v2)]

pub mod constants;
pub mod export;
pub mod logging;
pub mod store;

// It is not useless: CLion highlight
// `c_char` as alias - italic
// `c_void` as type or alias - non-italic
#[allow(non_camel_case_types)]
type c_void = std::ffi::c_void;
#[allow(non_camel_case_types)]
type c_char = std::ffi::c_char;

pub type FFICallbackContext = *mut c_void;

#[derive(Clone, Copy)]
pub struct FFICallbackContextWrapper(FFICallbackContext);

unsafe impl Send for FFICallbackContextWrapper {}
unsafe impl Sync for FFICallbackContextWrapper {}
