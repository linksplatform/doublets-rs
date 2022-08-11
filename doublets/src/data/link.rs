use std::fmt::{self, Debug, Formatter};

use data::{LinkType, Query, ToQuery};

#[derive(Default, Eq, PartialEq, Clone, Hash)]
#[repr(C)]
pub struct Link<T: LinkType> {
    pub index: T,
    pub source: T,
    pub target: T,
}

impl<T: LinkType> Link<T> {
    #[inline]
    #[must_use]
    pub fn nothing() -> Self {
        Self::default()
    }

    #[inline]
    #[must_use]
    pub const fn new(index: T, source: T, target: T) -> Self {
        Self {
            index,
            source,
            target,
        }
    }

    #[inline]
    #[must_use]
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
    #[must_use]
    pub(crate) const unsafe fn from_slice_unchecked(slice: &[T]) -> Self {
        match slice {
            [index, source, target] => Self::new(*index, *source, *target),
            _ => std::hint::unreachable_unchecked(),
        }
    }

    #[inline]
    #[must_use]
    pub fn is_null(&self) -> bool {
        *self == Self::point(T::funty(0))
    }

    #[inline]
    #[must_use]
    pub fn is_full(&self) -> bool {
        self.index == self.source && self.index == self.target
    }

    #[inline]
    #[must_use]
    pub fn is_partial(&self) -> bool {
        self.index == self.source || self.index == self.target
    }

    #[inline]
    #[must_use]
    pub const fn as_slice(&self) -> &[T] {
        // SAFETY: Link is repr(C) and therefore is safe to transmute to a slice
        unsafe { &*(self as *const Self).cast::<[T; 3]>() }
    }
}

impl<T: LinkType> Debug for Link<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {} {}", self.index, self.source, self.target)
    }
}

impl<T: LinkType> ToQuery<T> for Link<T> {
    fn to_query(&self) -> Query<'_, T> {
        self.as_slice().to_query()
    }
}
