use super::{c_char, FFICallbackContext};
use crate::FFICallbackContextWrapper;
use crossbeam_channel::{self as mpsc, Sender};
use std::{ffi::CString, io, str::FromStr, thread};
use tracing::{error};
use tracing_subscriber::{
    filter::{EnvFilter, LevelFilter},
    fmt::MakeWriter,
};

struct ChannelWriter {
    sender: Sender<Vec<u8>>,
}

impl ChannelWriter {
    pub fn new(sender: Sender<Vec<u8>>) -> Self {
        Self { sender }
    }
}

impl io::Write for ChannelWriter {
    fn write(&mut self, buf: &[u8]) -> Result<usize, io::Error> {
        let len = buf.len();
        let _ = self.sender.send(buf.to_vec());
        Ok(len)
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        Ok(())
    }
}

impl MakeWriter<'_> for ChannelWriter {
    type Writer = ChannelWriter;

    fn make_writer(&self) -> Self::Writer {
        ChannelWriter {
            sender: self.sender.clone(),
        }
    }
}

/// # Safety
/// This callback is safe if all the rules of Rust are followed
pub type LogFFICallback = unsafe extern "C" fn(FFICallbackContext, *const c_char);

pub struct DoubletsFFILogHandle {}

impl DoubletsFFILogHandle {
    pub fn new(
        ctx: FFICallbackContext,
        callback: LogFFICallback,
        max_level: &str,
        use_ansi: bool,
        use_json: bool,
    ) -> Self {
        log_panics::init();
        let wrapper = FFICallbackContextWrapper(ctx);
        let (sender, receiver) = mpsc::bounded(256);

        let callback = move |ctx: FFICallbackContextWrapper, ptr| {
            // SAFETY: caller must guarantee - we only delegate callback
            unsafe {
                callback(ctx.0, ptr);
            }
        };

        thread::spawn(move || {
            // We can't use `while let Ok(msg) = receiver.recv()`
            // here because the receiver will be blocked

            loop {
                // info_span!("Logging loop").in_scope(|| {
                if let Ok(msg) = receiver.recv() {
                    let str = CString::new(msg)
                        .expect("Only UTF-8 format strings are allowed in logging");
                    callback(wrapper, str.as_ptr());
                }
                // });
            }
        });

        let filter = EnvFilter::from_default_env()
            .add_directive(LevelFilter::from_str(max_level).unwrap().into());
        if use_json {
            if tracing_subscriber::fmt()
                .json()
                .with_ansi(use_ansi)
                .with_writer(ChannelWriter::new(sender))
                .with_env_filter(filter)
                .with_filter_reloading()
                .try_init()
                .is_err()
            {
                error!("Log handler already set, cannot currently change log levels.");
            }
        } else if tracing_subscriber::fmt()
            .with_ansi(use_ansi)
            .with_writer(ChannelWriter::new(sender))
            .with_env_filter(filter)
            .with_filter_reloading()
            .try_init()
            .is_err()
        {
            error!("Log handler already set, cannot currently change log levels.");
        };

        Self {}
    }
}
