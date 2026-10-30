use super::resize_mem;
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

fn check_resize(mut memory: impl RawMem<Item = u64>) {
    assert_eq!(resize_mem(&mut memory, 0).unwrap(), []);
    resize_mem(&mut memory, 4)
        .unwrap()
        .copy_from_slice(&[11, 22, 33, 44]);
    assert_eq!(
        resize_mem(&mut memory, 7).unwrap(),
        [11, 22, 33, 44, 0, 0, 0]
    );
    resize_mem(&mut memory, 7).unwrap()[6] = 77;
    assert_eq!(
        resize_mem(&mut memory, 9).unwrap(),
        [11, 22, 33, 44, 0, 0, 77, 0, 0]
    );
    assert_eq!(resize_mem(&mut memory, 3).unwrap(), [11, 22, 33]);
    assert_eq!(resize_mem(&mut memory, 5).unwrap(), [11, 22, 33, 0, 0]);
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
fn resize_returns_whole_allocation_when_backend_returns_only_grown_region() {
    check_resize(GrownRegionOnly(Global::new()));
}
