use std::{default::default, marker::PhantomData, ptr::NonNull};

use crate::{
    mem::{header::LinksHeader, unit::raw_link::LinkPart, LinksTree},
    Link,
};
use data::LinksConstants;
use methods::SzbTree;
use num::LinkType;

/// Size Balanced Tree base implementation for links
pub struct LinksSizeBalancedTreeBase<T: LinkType> {
    pub mem: NonNull<[LinkPart<T>]>,
    pub r#break: T,
    pub r#continue: T,

    _phantom: PhantomData<T>,
}

impl<T: LinkType> LinksSizeBalancedTreeBase<T> {
    pub fn new(constants: LinksConstants<T>, mem: NonNull<[LinkPart<T>]>) -> Self {
        Self {
            mem,
            r#break: constants.r#break,
            r#continue: constants.r#continue,
            _phantom: default(),
        }
    }
}

/// Abstract trait for Size balanced tree operations on links
pub trait LinkSizeBalancedTreeBaseAbstract<T: LinkType>:
    SzbTree<T> + LinksTree<T>
{
    fn get_header(&self) -> &LinksHeader<T>;

    fn get_mut_header(&mut self) -> &mut LinksHeader<T>;

    fn get_link(&self, link: T) -> &LinkPart<T>;

    fn get_mut_link(&mut self, link: T) -> &mut LinkPart<T>;

    fn get_tree_root(&self) -> T;

    fn get_base_part(&self, link: T) -> T;

    /// Compare if first link should be to the left of second in the tree
    fn first_is_to_the_left_of_second_4(
        &self,
        source: T,
        target: T,
        first_source: T,
        first_target: T,
    ) -> bool;

    /// Compare if first link should be to the left of second in the tree (3 params)
    fn first_is_to_the_left_of_second_3(
        &self,
        first: T,
        first_source: T,
        first_target: T,
    ) -> bool;

    /// Get the size of a subtree rooted at the given node
    fn get_size_or_zero(&self, node: T) -> T {
        if node == T::funty_zero() {
            T::funty_zero()
        } else {
            self.get_size(node)
        }
    }

    /// Get left child or default (zero) if none
    fn get_left_or_default(&self, node: T) -> T {
        if node == T::funty_zero() {
            T::funty_zero()
        } else {
            self.get_left(node)
        }
    }

    /// Get right child or default (zero) if none
    fn get_right_or_default(&self, node: T) -> T {
        if node == T::funty_zero() {
            T::funty_zero()
        } else {
            self.get_right(node)
        }
    }
}