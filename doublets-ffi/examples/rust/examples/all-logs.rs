#![feature(const_option_ext)]

use doublets_ffi::{
    export::{doublets_create_log_handle, doublets_free_log_handle},
    logging::{Format, Level},
    FFICallbackContext,
};
use std::{
    ffi::{c_char, CStr, CString},
    ptr,
};

unsafe extern "C" fn callback(_: FFICallbackContext, ptr: *const c_char) {
    let cstr = CStr::from_ptr(ptr);
    print!("{}", cstr.to_str().unwrap());
}

const FORMAT: &str = option_env!("RUST_EXAMPLES_FORMAT").unwrap_or("virgin");

fn main() {
    let ctx = ptr::null_mut();
    let level = Level::Trace;
    let use_ansi = true;

    let format = match &FORMAT.to_ascii_lowercase()[..] {
        "virgin" => Format::Virgin,
        "pretty" => Format::Pretty,
        "json" => Format::Json,
        _ => {
            panic!("allow only: `virgin`, `pretty`, `json`")
        }
    };

    unsafe {
        let handle = doublets_create_log_handle(ctx, callback, level, format, use_ansi);

        tracing::error!("SOMETHING IS SERIOUSLY WRONG!!!");
        tracing::warn!("important informational messages; might indicate an error");
        tracing::info!("general informational messages relevant to users");
        tracing::debug!("diagnostics used for internal debugging of a library or application");
        tracing::trace!("very verbose diagnostic events");

        doublets_free_log_handle(handle);
    }
}
