use doublets_ffi::{
    export::{doublets_create_log_handle, doublets_free_log_handle},
    FFICallbackContext,
};
use std::ffi::{c_char, CStr, CString};

unsafe extern "C" fn callback(ctx: FFICallbackContext, ptr: *const c_char) {
    let str = CStr::from_ptr(ptr).to_str().unwrap();
    let ctx = &mut *(ctx as *mut usize);
    *ctx += 1;

    if *ctx % 2 == 0 {
        print!("{str}");
    } else {
        eprint!("{str}");
    }
}

fn main() {
    let ctx = &mut 0usize as *mut usize;
    let level = CString::new("trace").unwrap();
    unsafe {
        let handle = doublets_create_log_handle(ctx.cast(), callback, level.as_ptr(), true, false);

        tracing::error!("SOMETHING IS SERIOUSLY WRONG!!!");
        tracing::warn!("important informational messages; might indicate an error");
        tracing::info!("general informational messages relevant to users");
        tracing::debug!("diagnostics used for internal debugging of a library or application");
        tracing::trace!("very verbose diagnostic events");

        doublets_free_log_handle(handle);
    }
}
