#![feature(try_blocks)]

use doublets_ffi::{
    export::{doublets_create_log_handle, doublets_free_log_handle},
    logging::{Format, Level},
    FFICallbackContext,
};
use std::{
    ffi::{c_char, CStr, CString},
    io::{self, Write},
};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

unsafe extern "C" fn callback(ctx: FFICallbackContext, ptr: *const c_char) {
    let str = CStr::from_ptr(ptr).to_str().unwrap();
    let ctx = &mut *(ctx as *mut usize);

    let mut stdout = StandardStream::stdout(ColorChoice::Always);

    let _: io::Result<_> = try {
        match *ctx % 5 {
            0..=1 => stdout.set_color(
                ColorSpec::new()
                    .set_fg(Some(Color::Rgb(0, 0, 255)))
                    .set_bg(Some(Color::Rgb(255, 165, 0))),
            )?,
            2 => stdout.set_color(
                ColorSpec::new()
                    .set_fg(Some(Color::Rgb(255, 165, 0)))
                    .set_bg(Some(Color::Rgb(0, 0, 255))),
            )?,
            3..=5 => stdout.set_color(
                ColorSpec::new()
                    .set_fg(Some(Color::Rgb(0, 0, 255)))
                    .set_bg(Some(Color::Rgb(255, 165, 0))),
            )?,
            _ => unreachable!(),
        }

        write!(&mut stdout, "{str}")?;

        stdout.reset()?;
    };

    *ctx += 1;
}

fn main() {
    let ctx = &mut 0usize as *mut usize;
    unsafe {
        let handle =
            doublets_create_log_handle(ctx.cast(), callback, Level::Trace, Format::Virgin, false);

        tracing::error!("SOMETHING IS SERIOUSLY WRONG!!!");
        tracing::warn!("important informational messages; might indicate an error");
        tracing::info!("general informational messages relevant to users");
        tracing::debug!("diagnostics used for internal debugging of a library or application");
        tracing::trace!("very verbose diagnostic events");

        doublets_free_log_handle(handle);
    }
}
