use {
    crate::{raw_place::RawPlace, utils, Error::CapacityOverflow, RawMem, Result},
    memmap2::{MmapMut, MmapOptions},
    std::{
        alloc::Layout,
        fmt::{self, Formatter},
        fs::File,
        io,
        mem::{self, MaybeUninit},
        path::Path,
        ptr::{self, NonNull},
    },
};

pub struct FileMapped<T> {
    buf: RawPlace<T>,
    mmap: Option<MmapMut>,
    pub(crate) file: File,
}

impl<T> FileMapped<T> {
    // todo: say about mapping, read-write guarantees, and `MIN_PAGE_SIZE`
    pub fn new(file: File) -> io::Result<Self> {
        const MIN_PAGE_SIZE: u64 = 4096;

        if file.metadata()?.len() < MIN_PAGE_SIZE {
            file.set_len(MIN_PAGE_SIZE)?;
        }

        Ok(Self { file, buf: RawPlace::dangling(), mmap: None })
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        File::options().create(true).read(true).write(true).open(path).and_then(Self::new)
    }

    fn map_yet(&mut self, cap: u64) -> io::Result<MmapMut> {
        unsafe { MmapOptions::new().len(cap as usize).map_mut(&self.file) }
    }

    unsafe fn assume_mapped(&mut self) -> &mut [u8] {
        self.mmap.as_mut().unwrap_unchecked()
    }
}

impl<T> RawMem for FileMapped<T> {
    type Item = T;

    fn allocated(&self) -> &[Self::Item] {
        unsafe { self.buf.as_slice() }
    }

    fn allocated_mut(&mut self) -> &mut [Self::Item] {
        unsafe { self.buf.as_slice_mut() }
    }

    unsafe fn grow(
        &mut self,
        addition: usize,
        fill: impl FnOnce(usize, (&mut [T], &mut [MaybeUninit<T>])),
    ) -> Result<&mut [T]> {
        let cap = self.buf.cap().checked_add(addition).ok_or(CapacityOverflow)?;
        // use layout to prevent all capacity bugs
        let layout = Layout::array::<T>(cap).map_err(|_| CapacityOverflow)?;
        let new_size = layout.size() as u64;

        // unmap the file by calling `Drop` of `mmap`
        let _ = self.mmap.take();

        let old_size = self.file.metadata()?.len();

        #[rustfmt::skip]
        let inited = if old_size < new_size {
            self.file.set_len(new_size)?;
            (old_size as usize / mem::size_of::<T>()) // more flexible without `rustfmt`
                .unchecked_sub(self.buf.cap())
        } else {
            addition // all place is available as initialized
        };

        let ptr = unsafe {
            let mmap = self.map_yet(new_size)?;
            self.mmap.replace(mmap);
            // we set it now: ^^^
            NonNull::from(self.assume_mapped()) // it assume that `mmap` is some
        };

        Ok(self.buf.handle_fill((ptr.cast(), cap), inited, fill))
    }

    fn shrink(&mut self, cap: usize) -> Result<()> {
        let cap = self.buf.cap().checked_sub(cap).expect("Tried to shrink to a larger capacity");
        self.buf.shrink_to(cap);

        let _ = self.mmap.take();

        let ptr = unsafe {
            // we can skip this checks because this memory layout is valid
            // then smaller layout will also be valid
            let new_size = mem::size_of::<T>().unchecked_mul(cap) as u64;
            self.file.set_len(new_size)?;

            let mmap = self.map_yet(new_size)?;
            self.mmap.replace(mmap);

            self.assume_mapped().into()
        };

        self.buf.set_ptr(ptr);

        Ok(())
    }
}

impl<T> Drop for FileMapped<T> {
    fn drop(&mut self) {
        unsafe {
            ptr::drop_in_place(self.buf.as_slice_mut());
        }

        let _ = self.file.sync_all();
    }
}

impl<T> fmt::Debug for FileMapped<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        utils::debug_mem(f, &self.buf, "FileMapped")?
            .field("mmap", &self.mmap)
            .field("file", &self.file)
            .finish()
    }
}
