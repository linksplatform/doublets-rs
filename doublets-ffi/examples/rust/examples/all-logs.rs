use doublets_ffi::{
    export::{doublets_create_log_handle, doublets_free_log_handle},
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

fn main() {
    let ctx = ptr::null_mut();
    let level = CString::new("trace").unwrap();
    let use_ansi = true;
    let use_json = false;
    unsafe {
        let handle = doublets_create_log_handle(ctx, callback, level.as_ptr(), use_ansi, use_json);

        tracing::error!("SOMETHING IS SERIOUSLY WRONG!!!");
        tracing::warn!("important informational messages; might indicate an error");
        tracing::info!("general informational messages relevant to users");
        tracing::debug!("diagnostics used for internal debugging of a library or application");
        tracing::trace!("very verbose diagnostic events");

        doublets_free_log_handle(handle);
    }
}
