#![feature(
    allocator_api,
    unchecked_math,
    maybe_uninit_slice,
    slice_ptr_get,
    ptr_as_uninit,
    inline_const,
    slice_range,
    maybe_uninit_write_slice,
    unboxed_closures,
    fn_traits
)]
// special lint
#![cfg_attr(not(test), forbid(clippy::unwrap_used))]
// rust compiler lints
#![deny(unused_must_use)]
#![warn(missing_debug_implementations)]

mod alloc;
mod file_mapped;
mod raw_mem;
mod raw_place;
mod utils;

pub(crate) use raw_place::RawPlace;
pub use {
    alloc::Alloc,
    file_mapped::FileMapped,
    raw_mem::{ErasedMem, Error, RawMem, Result},
};

fn _assertion() {
    fn assert_sync_send<T: Sync + Send>() {}

    assert_sync_send::<FileMapped<()>>();
    assert_sync_send::<Alloc<(), std::alloc::Global>>();
}

macro_rules! delegate_memory {
    ($($me:ident<$param:ident>($inner:ty) { $($body:tt)* } )*) => {$(
        pub struct $me<$param>($inner);

        impl<$param> $me<$param> {
            $($body)*
        }

        const _: () = {
            use std::{
                mem::MaybeUninit,
                fmt::{self, Formatter},
            };

            impl<$param> RawMem for $me<$param> {
                type Item = $param;

                fn allocated(&self) -> &[Self::Item] {
                    self.0.allocated()
                }

                fn allocated_mut(&mut self) -> &mut [Self::Item] {
                    self.0.allocated_mut()
                }

                unsafe fn grow(
                    &mut self,
                    addition: usize,
                    fill: impl FnOnce(usize, (&mut [Self::Item], &mut [MaybeUninit<Self::Item>])),
                ) -> Result<&mut [Self::Item]> {
                    self.0.grow(addition, fill)
                }

                fn shrink(&mut self, cap: usize) -> Result<()> {
                    self.0.shrink(cap)
                }

                fn size_hint(&self) -> Option<usize> {
                    self.0.size_hint()
                }
            }

            impl<T> fmt::Debug for $me<$param> {
                fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                    f.debug_tuple(stringify!($me)).field(&self.0).finish()
                }
            }

        };
    )*};
}

use std::{
    alloc::{Global as GlobalAlloc, System as SystemAlloc},
    fs::File,
    io,
    path::Path,
};

delegate_memory! {
    Global<T>(Alloc<T, GlobalAlloc>) {
        pub const fn new() -> Self {
            Self(Alloc::new(GlobalAlloc))
        }
    }
   System<T>(Alloc<T, SystemAlloc>) {
       pub const fn new() -> Self {
           Self(Alloc::new(SystemAlloc))
       }
   }
   TempFile<T>(FileMapped<T>) {
       pub fn new() -> io::Result<Self> {
           Self::from_temp(tempfile::tempfile())
       }

       pub fn new_in<P: AsRef<Path>>(path: P) -> io::Result<Self> {
           Self::from_temp(tempfile::tempfile_in(path))
       }

       fn from_temp(file: io::Result<File>) -> io::Result<Self> {
           file.and_then(FileMapped::new).map(Self)
       }
   }
}

// fixme: add flag when it needs in macro
impl<T> Default for Global<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Default for System<T> {
    fn default() -> Self {
        Self::new()
    }
}

fn _is_raw_mem() {
    fn check<T: RawMem>() {}

    check::<Box<dyn ErasedMem<Item = ()>>>();
    check::<Box<dyn ErasedMem<Item = ()> + Sync>>();
    check::<Box<dyn ErasedMem<Item = ()> + Sync + Send>>();

    fn elie() -> Box<Global<()>> {
        todo!()
    }

    let _: Box<dyn ErasedMem<Item = ()>> = elie();
    let _: Box<dyn ErasedMem<Item = ()> + Sync> = elie();
    let _: Box<dyn ErasedMem<Item = ()> + Sync + Send> = elie();
}
