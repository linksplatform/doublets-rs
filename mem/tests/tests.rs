macro_rules! define_impls {
    (impl RawMem: {
        $($ctor:expr /* -- */ $(=> in $cfg:meta)? ),+ $(,)?
    } for [
        $($test:path as $name:ident),* $(,)?
    ]) => {
        define_impls! { @loop
            [/* empty result */]
            [ $($ctor $(=> $cfg)? )*]
            [ $($test as $name |)* ]
        }
    };

   (@loop [ $($result:tt)* ] // result accumulation
           [ $($ctor:expr $(=> $cfg:meta)? )* ] // each ctor with our cfg `not(miri)`
           [ $test:path as $name:ident | $($tail:tt)* ] // match test with name + tail
    ) => {
        define_impls! { @loop
            [
                $($result)*

                #[test]
                fn $name() {
                    $( $(#[cfg($cfg)])? Terminate::report($test($ctor));)*
                }
            ]
            [$($ctor $(=> $cfg)? )*]
            [ $($tail)* ]
        }
    };

    (@loop [ $($result:tt)* ] [ $($_:tt)* ] [ /* tests still coming */ ] ) => {
        $($result)*
    };
}

trait Terminate {
    fn report(me: Self);
}

impl Terminate for () {
    fn report(_: Self) {}
}

impl<T, E: Debug> Terminate for Result<T, E> {
    fn report(me: Self) {
        me.unwrap();
    }
}

use {
    platform_mem::{Global, System, TempFile},
    std::fmt::Debug,
};

mod mem;
mod miri;
#[cfg(test)]
define_impls! {
    impl RawMem: {
        Global::new(),
        System::new(),
        TempFile::new().unwrap() => in not(miri),
    } for [
        miri::miri as miri,
        mem::grow_from_slice as grow_from_slice,
    ]
}
