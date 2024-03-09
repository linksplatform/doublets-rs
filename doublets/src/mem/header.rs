use {core::LinkType, std::mem::MaybeUninit};

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Header<T: LinkType> {
    pub allocated: T,
    pub reserved: T,
    pub free: T,
    pub first_free: T,
    pub last_free: T,
    pub sources_root: Option<T::Repr>,
    pub targets_root: Option<T::Repr>,
    __pad_to_8: MaybeUninit<T>,
}
