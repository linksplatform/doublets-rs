use super::{parts::DataPart, resize_mem};
use mem::{FileMapped, Global, RawMem};
use std::mem::MaybeUninit;

pub(super) fn check_links(store: &impl crate::Doublets<usize>, count: usize) {
    assert_eq!(store.count(), count);
    for index in 1..=count {
        assert_eq!(store.get_link(index), Some(crate::Link::point(index)));
        assert_eq!(store.search(index, index), Some(index));
    }
}

// Preserve the grown-region contract even if a future backend returns more.
struct GrownRegionOnly<M>(M);

impl<M: RawMem> RawMem for GrownRegionOnly<M> {
    type Item = M::Item;

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
    ) -> mem::Result<&mut [Self::Item]> {
        let old_len = self.allocated().len();
        // SAFETY: the caller supplies the initializer required by the backend.
        unsafe { self.0.grow(addition, fill)? };
        Ok(&mut self.allocated_mut()[old_len..])
    }

    fn shrink(&mut self, count: usize) -> mem::Result<()> {
        self.0.shrink(count)
    }
}

fn part(source: u64) -> DataPart<u64> {
    DataPart { source, target: 0 }
}

fn check_resize(mut memory: impl RawMem<Item = DataPart<u64>>) {
    assert_eq!(resize_mem(&mut memory, 0).unwrap(), []);
    resize_mem(&mut memory, 4)
        .unwrap()
        .clone_from_slice(&[11, 22, 33, 44].map(part));
    assert_eq!(
        resize_mem(&mut memory, 7).unwrap(),
        [11, 22, 33, 44, 0, 0, 0].map(part)
    );
    resize_mem(&mut memory, 7).unwrap()[6] = part(77);
    assert_eq!(
        resize_mem(&mut memory, 9).unwrap(),
        [11, 22, 33, 44, 0, 0, 77, 0, 0].map(part)
    );
    assert_eq!(resize_mem(&mut memory, 3).unwrap(), [11, 22, 33].map(part));
    assert_eq!(
        resize_mem(&mut memory, 5).unwrap(),
        [11, 22, 33, 0, 0].map(part)
    );
    assert_eq!(
        resize_mem(&mut memory, 5).unwrap().as_ptr(),
        memory.allocated().as_ptr()
    );
}

#[test]
fn resize_global() {
    check_resize(Global::new());
}

#[test]
fn resize_file_mapped() {
    check_resize(FileMapped::new(tempfile::tempfile().unwrap()).unwrap());
}

#[test]
fn resize_file_mapped_preserves_complete_parts_and_clears_partial_part() {
    use std::io::Write;

    let mut file = tempfile::tempfile().unwrap();
    file.write_all(&11_u64.to_ne_bytes()).unwrap();
    file.write_all(&0_u64.to_ne_bytes()).unwrap();
    file.write_all(&[0xff; 5]).unwrap();
    let mut memory = FileMapped::<DataPart<u64>>::new(file).unwrap();

    assert_eq!(resize_mem(&mut memory, 4).unwrap(), [11, 0, 0, 0].map(part));
}

#[cfg(all(target_os = "linux", not(miri)))]
#[test]
#[allow(
    clippy::mut_mut,
    reason = "exercise the RawMem implementation for a borrowed mapping"
)]
fn second_file_mapped_resize_stays_sparse() {
    use std::os::unix::fs::MetadataExt;

    let file = tempfile::tempfile().unwrap();
    let mut mapped = FileMapped::new(file.try_clone().unwrap()).unwrap();
    let mut memory = &mut mapped;
    resize_mem(&mut memory, 16).unwrap()[15] = part(77);

    let capacity = 1 << 20;
    let grown = resize_mem(&mut memory, capacity).unwrap();
    assert_eq!(grown[15], part(77));
    assert_eq!(grown[16], part(0));
    assert_eq!(grown[capacity - 1], part(0));

    drop(mapped);
    file.sync_all().unwrap();
    let metadata = file.metadata().unwrap();
    let allocated_bytes = metadata.blocks() * 512;
    assert_eq!(metadata.len(), u64::try_from(capacity).unwrap() * 16);
    assert!(
        allocated_bytes < 1024 * 1024,
        "allocated {allocated_bytes} bytes"
    );
}

#[test]
fn resize_returns_whole_allocation_when_backend_returns_only_grown_region() {
    check_resize(GrownRegionOnly(Global::new()));
}
