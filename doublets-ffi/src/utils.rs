/// Stable `try` block only for `Result`
macro_rules! stable_try {
    ($($block:tt)*) => {
        (|| {
            let ret = { $($block)* };
            core::result::Result::Ok(ret)
        })()
    };
}

pub(crate) use stable_try;
