use std::fmt::{self, Debug, Formatter};

use data::{LinkReference, Query, ToQuery};

/// A link triple `(index, source, target)` that represents one node in the doublets graph.
///
/// Every link is uniquely identified by its `index` and connects a `source` to a `target`.
#[derive(Default, Eq, PartialEq, Clone, Hash)]
#[repr(C)]
pub struct Link<T: LinkReference> {
    /// Unique identifier of this link within the store.
    pub index: T,
    /// The source (left) endpoint of this link.
    pub source: T,
    /// The target (right) endpoint of this link.
    pub target: T,
}

impl<T: LinkReference> Link<T> {
    /// Returns a default/null link with all fields set to zero.
    #[inline]
    #[must_use]
    pub fn nothing() -> Self {
        Self::default()
    }

    /// Creates a new link with the given `index`, `source`, and `target`.
    #[inline]
    #[must_use]
    pub const fn new(index: T, source: T, target: T) -> Self {
        Self {
            index,
            source,
            target,
        }
    }

    /// Creates a self-referential (point) link where `index == source == target`.
    #[inline]
    #[must_use]
    pub const fn point(val: T) -> Self {
        Self::new(val, val, val)
    }

    /// Constructs a [`Link`] from the first three elements of `slice`.
    ///
    /// # Panics
    ///
    /// Panics if `slice.len() < 3`.
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

    /// Returns `true` if this is the null (zero) link.
    #[inline]
    #[must_use]
    pub fn is_null(&self) -> bool {
        *self == Self::point(T::from_byte(0))
    }

    /// Returns `true` if the link is a full point (`index == source == target`).
    #[inline]
    #[must_use]
    pub fn is_full(&self) -> bool {
        self.index == self.source && self.index == self.target
    }

    /// Returns `true` if `index` equals either `source` or `target`.
    #[inline]
    #[must_use]
    pub fn is_partial(&self) -> bool {
        self.index == self.source || self.index == self.target
    }

    /// Returns the link fields as a three-element slice `[index, source, target]`.
    #[inline]
    #[must_use]
    pub const fn as_slice(&self) -> &[T] {
        // SAFETY: Link is repr(C) and therefore is safe to transmute to a slice
        unsafe { &*(self as *const Self).cast::<[T; 3]>() }
    }
}

impl<T: LinkReference> Debug for Link<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {} {}", self.index, self.source, self.target)
    }
}

impl<T: LinkReference> ToQuery<T> for Link<T> {
    fn to_query(&self) -> Query<'_, T> {
        self.as_slice().to_query()
    }
}
