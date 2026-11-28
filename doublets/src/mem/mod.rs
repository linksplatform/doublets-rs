//! Memory layout and store implementations for doublets.
//!
//! Two store backends are available:
//! - [`mod@unit`] — a single contiguous memory region per store.
//! - [`split`] — separate data and index memory regions.

pub use header::LinksHeader;
pub use mutation::UncheckedMutation;
pub use release::Release;
pub use traits::{
    LinksList, LinksTree, SplitList, SplitTree, SplitUpdateMem, UnitTree, UnitUpdateMem,
};
mod header;
mod mutation;
mod release;
pub mod split;
mod traits;
pub mod unit;

#[cfg(test)]
#[path = "../../tests/internal/mem.rs"]
mod tests;

#[cfg(feature = "mem")]
pub use ::mem::*;

pub mod parts {
    pub use super::{
        split::{DataPart, IndexPart},
        unit::LinkPart,
    };
}

/// A record stored in a doublets memory backend.
///
/// # Safety
/// For a native unsigned `Reference`, the record must accept every bit pattern
/// and compare equal to `Self::default()` exactly when its bytes are all zero.
pub(crate) unsafe trait StorePart: Clone + Default + PartialEq + 'static {
    type Reference: data::LinkReference;
}

// SAFETY: Each part has a C layout, contains only fields of type T, and derives Default.
unsafe impl<T: data::LinkReference> StorePart for parts::LinkPart<T> {
    type Reference = T;
}

unsafe impl<T: data::LinkReference> StorePart for parts::DataPart<T> {
    type Reference = T;
}

unsafe impl<T: data::LinkReference> StorePart for parts::IndexPart<T> {
    type Reference = T;
}

fn zero_default<T: data::LinkReference>() -> bool {
    use std::any::TypeId;

    // These integer types accept every bit pattern and have an all-zero default.
    [
        TypeId::of::<u8>(),
        TypeId::of::<u16>(),
        TypeId::of::<u32>(),
        TypeId::of::<u64>(),
        TypeId::of::<u128>(),
        TypeId::of::<usize>(),
    ]
    .contains(&TypeId::of::<T>())
}

fn zero_extends_file<M: ::mem::RawMem<Item: 'static>>() -> bool {
    use ::mem::{FileMapped, TempFile};

    // Compare backend types without requiring stores to own their memory.
    let backend = typeid::of::<M>();
    backend == typeid::of::<FileMapped<M::Item>>()
        || backend == typeid::of::<&mut FileMapped<M::Item>>()
        || backend == typeid::of::<TempFile<M::Item>>()
        || backend == typeid::of::<&mut TempFile<M::Item>>()
}

pub(crate) fn resize_mem<M: ::mem::RawMem>(
    m: &mut M,
    new_cap: usize,
) -> ::mem::Result<&mut [M::Item]>
where
    M::Item: StorePart,
{
    let current = m.allocated().len();
    if new_cap > current {
        let addition = new_cap - current;
        if zero_default::<<M::Item as StorePart>::Reference>() && zero_extends_file::<M>() {
            // SAFETY: The store parts are repr(C) structs of native unsigned integers.
            // All mapped bytes represent valid parts. FileMapped (also used by TempFile)
            // zero-extends its files, so the remaining tail already holds default parts.
            unsafe {
                m.grow(addition, |initialized, (_, added)| {
                    // A pre-existing file can end inside the first item after the
                    // initialized prefix. Clear that item if it contains any old bytes.
                    if let Some(first) = added.get_mut(initialized) {
                        let default = M::Item::default();
                        if *first.assume_init_ref() != default {
                            first.write(default);
                        }
                    }
                })?;
            }
        } else {
            m.grow_filled(addition, M::Item::default())?;
        }
    } else if new_cap < current {
        m.shrink(current - new_cap)?;
    }
    // RawMem growth returns the added region; stores need the entire allocation.
    Ok(m.allocated_mut())
}
