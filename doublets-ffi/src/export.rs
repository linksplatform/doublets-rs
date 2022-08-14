use crate::{
    c_char,
    logging::{DoubletsFFILogHandle, LogFFICallback},
    FFICallbackContext,
};
use std::ffi::CStr;
use tracing::error;

#[no_mangle]
pub extern "C" fn doublets_activate_env_logger() {
    if tracing_subscriber::fmt::try_init().is_err() {
        error!("Cannot re-init env logger, this should only be called once");
    }
}

#[no_mangle]
pub unsafe extern "C" fn doublets_create_log_handle(
    callback: LogFFICallback,
    ctx: FFICallbackContext,
    max_level: *const c_char,
    use_ansi: bool,
    use_json: bool,
) -> *mut DoubletsFFILogHandle {
    assert!(!max_level.is_null());
    // if str isn't utf-8 just panic
    let max_level_str = CStr::from_ptr(max_level).to_str().unwrap();
    Box::into_raw(Box::new(DoubletsFFILogHandle::new(
        callback,
        ctx,
        max_level_str,
        use_ansi,
        use_json,
    )))
}

#[no_mangle]
pub unsafe extern "C" fn doublets_free_log_handle(ptr: *mut DoubletsFFILogHandle) {
    if !ptr.is_null() {
        let _ = Box::from_raw(ptr);
    }
}
