// pub use generic::*;
pub use link::LinkRepr;
// pub use store::Store;

// mod generic;
mod link;
mod store;
mod trees;
// mod store;

use crate::mem::Header;
pub use trees::{Sources, Targets, Tree};

const _: () = {
    assert!(std::mem::size_of::<Header<u8>>() == 8);
    assert!(std::mem::size_of::<LinkRepr<u8>>() == 8);
};
