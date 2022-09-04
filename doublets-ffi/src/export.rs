use crate::{
    logging::{DoubletsFFILogHandle, Format, Level, LogFFICallback},
    FFICallbackContext,
};

use tracing::error;

/// Basic logger. For advanced use [`doublets_create_log_handle`]
#[no_mangle]
pub extern "C" fn doublets_activate_env_logger() {
    if tracing_subscriber::fmt::try_init().is_err() {
        error!("Cannot re-init env logger, this should only be called once");
    }
}

#[no_mangle]
pub unsafe extern "C" fn doublets_create_log_handle(
    ctx: FFICallbackContext,
    callback: LogFFICallback,
    max_level: Level,
    format: Format,
    ansi: bool,
) -> Box<DoubletsFFILogHandle> {
    Box::new(DoubletsFFILogHandle::new(
        ctx, callback, max_level, format, ansi,
    ))
}

#[no_mangle]
pub unsafe extern "C" fn doublets_free_log_handle(log_handle: Box<DoubletsFFILogHandle>) {
    let _ = log_handle;
}
