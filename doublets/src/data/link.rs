use std::{
    fmt::{self, Debug, Formatter},
    mem::transmute,
};

use data::{LinkType, Query, ToQuery};

#[derive(Default, Eq, PartialEq, Clone, Hash)]
#[repr(C)]
pub struct Link<T: LinkType> {
    pub index: T,
    pub source: T,
    pub target: T,
}

impl<T: LinkType> Link<T> {
    pub fn nothing() -> Self {
        Self::default()
    }

    pub fn new(index: T, source: T, target: T) -> Self {
        Self {
            index,
            source,
            target,
        }
    }

    pub fn point(val: T) -> Self {
        Self::new(val, val, val)
    }

    pub fn from_slice(slice: &[T]) -> Self {
        assert!(slice.len() >= 3);

        // SAFETY: slice has at least 3 elements.
        unsafe { Self::from_slice_unchecked(slice) }
    }

    pub(crate) unsafe fn from_slice_unchecked(slice: &[T]) -> Self {
        match slice {
            [index, source, target] => Self::new(*index, *source, *target),
            _ => std::hint::unreachable_unchecked(),
        }
    }

    pub fn is_null(&self) -> bool {
        *self == Self::point(T::funty(0))
    }

    pub fn is_full(&self) -> bool {
        self.index == self.source && self.index == self.target
    }

    pub fn is_partial(&self) -> bool {
        self.index == self.source || self.index == self.target
    }

    pub fn as_slice(&self) -> &[T] {
        // SAFETY: Link is repr(C) and therefore is safe to transmute to a slice
        unsafe { transmute::<_, &[T; 3]>(self) }
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
