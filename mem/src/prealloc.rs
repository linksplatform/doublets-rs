use {
    crate::{Error::CapacityOverflow, RawMem, Result},
    std::{
        mem::{self, MaybeUninit},
        ops::{Deref, DerefMut},
    },
};
#[derive(Debug)]
pub struct PreAlloc<P> {
    place: P,
    used: usize,
}

impl<T, P: Deref<Target = [T]> + DerefMut> PreAlloc<P> {
    /// Constructs new `PreAlloc`
    pub fn new(place: P) -> Self {
        Self { place, used: 0 }
    }
}

impl<T, P: Deref<Target = [T]> + DerefMut> RawMem for PreAlloc<P> {
    type Item = T;

    fn allocated(&self) -> &[Self::Item] {
        &self.place[..self.used]
    }

    fn allocated_mut(&mut self) -> &mut [Self::Item] {
        &mut self.place[..self.used]
    }

    unsafe fn grow(
        &mut self,
        addition: usize,
        fill: impl FnOnce(&mut [MaybeUninit<Self::Item>]),
    ) -> Result<&mut [Self::Item]> {
        let cap = self.used.checked_add(addition).ok_or(CapacityOverflow)?;
        let available = self.place.len();

        if let Some(slice) = self.place.get_mut(self.used..cap) {
            fill(mem::transmute(&mut slice[..]));
            self.used = cap;
            Ok(slice)
        } else {
            Err(crate::Error::OverAlloc { available, to_alloc: cap })
        }
    }

    fn shrink(&mut self, cap: usize) -> Result<()> {
        self.used = self.used.checked_sub(cap).expect("Tried to shrink to a larger capacity");
        Ok(())
    }
}
