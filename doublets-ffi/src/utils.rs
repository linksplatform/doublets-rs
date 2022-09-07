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
use std::mem::MaybeUninit;

#[repr(C)]
pub struct Maybe<T> {
    // `MaybeUninit` is transparent - `Range<T>` is repr(C)
    value: MaybeUninit<T>,
    some: bool,
}

impl<T> Maybe<T> {
    pub fn some(value: T) -> Self {
        Self {
            value: MaybeUninit::new(value),
            some: true,
        }
    }

    pub fn none() -> Self {
        Self {
            value: MaybeUninit::uninit(),
            some: false,
        }
    }

    pub fn unwrap(self) -> T {
        if self.some {
            // SAFETY: value is some
            unsafe { self.value.assume_init() }
        } else {
            panic!("called `Maybe::unwrap()` on a `none` value")
        }
    }
}

impl<T> From<Option<T>> for Maybe<T> {
    fn from(opt: Option<T>) -> Self {
        match opt {
            Some(val) => Maybe::some(val),
            None => Maybe::none(),
        }
    }
}

impl<T> From<Maybe<T>> for Option<T> {
    fn from(maybe: Maybe<T>) -> Self {
        if maybe.some {
            // SAFETY: value is some
            unsafe { Some(maybe.value.assume_init()) }
        } else {
            None
        }
    }
}
