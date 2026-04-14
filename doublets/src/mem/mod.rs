pub use header::LinksHeader;
pub use traits::{
    LinksList, LinksTree, SplitList, SplitTree, SplitUpdateMem, UnitTree, UnitUpdateMem,
};
mod header;
pub mod split;
mod traits;
pub mod unit;

#[cfg(feature = "mem")]
pub use ::mem::*;

pub mod parts {
    pub use super::{
        split::{DataPart, IndexPart},
        unit::LinkPart,
    };
}

pub(crate) fn resize_mem<M: ::mem::RawMem>(
    m: &mut M,
    new_cap: usize,
) -> ::mem::Result<&mut [M::Item]>
where
    M::Item: Default + Clone,
{
    let current = m.allocated().len();
    if new_cap > current {
        m.grow_filled(new_cap - current, M::Item::default())
    } else if new_cap < current {
        m.shrink(current - new_cap)?;
        Ok(m.allocated_mut())
    } else {
        Ok(m.allocated_mut())
    }
}
