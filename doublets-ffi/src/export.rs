use crate::{
    logging::{DoubletsFFILogHandle, Format, Level, LogFFICallback},
    FFIContext,
};

#[no_mangle]
pub unsafe extern "C" fn doublets_create_log_handle(
    ctx: FFIContext,
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
