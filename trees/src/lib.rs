#![feature(try_blocks)]
// fixme: #![no_std]
#![deny(unused_must_use)]

// mod lists;
pub mod trees;

pub use {
    // lists::{
    //     AbsoluteCircularLinkedList, AbsoluteLinkedList, LinkedList, RelativeCircularLinkedList,
    //     RelativeLinkedList,
    // },
    trees::{Leaf, NoRecur, Tree},
};

macro_rules! named {
    ($($name:ident => $val:expr)*) => {
        $(
            fn $name() -> Self {
                Self::from_addr($val)
            }
        )*
    };
}
