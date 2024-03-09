use {
    core::LinkType,
    std::fmt::{self, Debug, Display, Formatter},
};

#[derive(Default, Eq, PartialEq, Clone, Hash)]
#[repr(C)]
pub struct Link<T: LinkType> {
    pub index: T,
    pub source: T,
    pub target: T,
}

impl<T: LinkType> Link<T> {
    #[inline]
    pub const fn new(index: T, source: T, target: T) -> Self {
        Self { index, source, target }
    }

    #[inline]
    pub const fn point(val: T) -> Self {
        Self::new(val, val, val)
    }

    #[inline]
    pub const fn from_slice(slice: &[T]) -> Self {
        assert!(slice.len() >= 3);

        // SAFETY: slice has at least 3 elements.
        unsafe { Self::from_slice_unchecked(slice) }
    }

    #[inline]
    pub(crate) const unsafe fn from_slice_unchecked(slice: &[T]) -> Self {
        match slice {
            [index, source, target] => Self::new(*index, *source, *target),
            _ => std::hint::unreachable_unchecked(),
        }
    }

    #[inline]
    pub const fn as_slice(&self) -> &[T] {
        // SAFETY: Link is repr(C) and therefore is safe to transmute to a slice
        unsafe { &*(self as *const Self).cast::<[T; 3]>() }
    }
}

impl<T: LinkType + PartialEq> Link<T> {
    #[inline]
    pub fn is_full(&self) -> bool {
        self.index == self.source && self.index == self.target
    }

    #[inline]
    pub fn is_part(&self) -> bool {
        self.index == self.source || self.index == self.target
    }
}

impl<T: LinkType + Display> Debug for Link<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {} {}", self.index, self.source, self.target)
    }
}
