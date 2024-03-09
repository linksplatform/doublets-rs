use core::LinkType;

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone, Copy)]
#[repr(C)]
pub struct LinkRepr<T: LinkType> {
    pub(crate) source: T,
    pub(crate) target: T,
    pub(crate) source_size: T,
    pub(crate) target_size: T,
    pub(crate) source_left: Option<T::Repr>,
    pub(crate) source_right: Option<T::Repr>,
    pub(crate) target_left: Option<T::Repr>,
    pub(crate) target_right: Option<T::Repr>,
}
