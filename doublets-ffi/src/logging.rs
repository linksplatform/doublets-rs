use super::{c_char, FFICallbackContext};
use crate::FFICallbackContextWrapper;
use crossbeam_channel::{self as mpsc, Sender};
use std::{ffi::CString, io, str::FromStr, thread};
use tap::Pipe;
use tracing::{dispatcher, error, subscriber, Dispatch, Subscriber};
use tracing_subscriber::{
    filter::{EnvFilter, LevelFilter},
    fmt::{format, FormatFields, MakeWriter, SubscriberBuilder},
    util::SubscriberInitExt,
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

#[repr(usize)]
pub enum Level {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
    // FFI binding can contain
    // Off = 5
    // But `tracing` must ignore it
}

#[repr(usize)]
pub enum Format {
    Virgin,
    Pretty,
    Json,
}

pub struct DoubletsFFILogHandle {/* opaque */}

impl DoubletsFFILogHandle {
    pub fn new(
        ctx: FFICallbackContext,
        callback: LogFFICallback,
        max_level: Level,
        format: Format,
        ansi: bool,
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
                } else {
                    break;
                }
                // });
            }
        });

        let filter = EnvFilter::from_default_env().add_directive(
            LevelFilter::from_level(match max_level {
                Level::Trace => tracing::Level::TRACE,
                Level::Debug => tracing::Level::DEBUG,
                Level::Info => tracing::Level::INFO,
                Level::Warn => tracing::Level::WARN,
                Level::Error => tracing::Level::ERROR,
            })
            .into(),
        );

        macro_rules! subscribe {
            ($($methods:tt)*) => {
                tracing_subscriber::fmt()
                    $($methods)*
                    .with_ansi(ansi)
                    .with_writer(ChannelWriter::new(sender))
                    .with_env_filter(filter)
                    .with_filter_reloading()
                    .finish()
            };
        }

        if match format {
            Format::Virgin => Box::new(subscribe!()) as Box<dyn Subscriber + Send + Sync>,
            Format::Pretty => Box::new(subscribe! { .pretty() }),
            Format::Json => Box::new(subscribe! { .json() }),
        }
        .pipe(subscriber::set_global_default)
        .is_err()
        {
            error!(
                "Log handler already set, cannot currently change: track issue \
                 `https://github.com/linksplatform/doublets-rs/issues/12`"
            );
        };

        Self {}
    }
}
