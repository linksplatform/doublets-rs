use std::{marker::PhantomData, ptr::NonNull};

use crate::{
    mem::{
        header::LinksHeader,
        unit::{generic::LinkAvlBalancedTreeBaseAbstract, raw_link::LinkPart},
        LinksTree,
    },
    Link,
};
use data::LinksConstants;
use methods::AvlTree;
use num::LinkType;

/// AVL balanced tree implementation for managing targets of links
pub struct LinksTargetsAvlBalancedTree<T: LinkType> {
    pub mem: NonNull<[LinkPart<T>]>,
    pub header: NonNull<LinksHeader<T>>,
    pub r#break: T,
    pub r#continue: T,

    _phantom: PhantomData<T>,
}

impl<T: LinkType> LinksTargetsAvlBalancedTree<T> {
    pub fn new(
        constants: LinksConstants<T>,
        mem: NonNull<[LinkPart<T>]>,
        header: NonNull<LinksHeader<T>>,
    ) -> Self {
        Self {
            mem,
            header,
            r#break: constants.r#break,
            r#continue: constants.r#continue,
            _phantom: PhantomData,
        }
    }
}

impl<T: LinkType> LinkAvlBalancedTreeBaseAbstract<T> for LinksTargetsAvlBalancedTree<T> {
    fn get_header(&self) -> &LinksHeader<T> {
        unsafe { self.header.as_ref() }
    }

    fn get_mut_header(&mut self) -> &mut LinksHeader<T> {
        unsafe { self.header.as_mut() }
    }

    fn get_link(&self, link: T) -> &LinkPart<T> {
        let index = link.as_usize();
        unsafe { &self.mem.as_ref()[index] }
    }

    fn get_mut_link(&mut self, link: T) -> &mut LinkPart<T> {
        let index = link.as_usize();
        unsafe { &mut self.mem.as_mut()[index] }
    }

    fn get_tree_root(&self) -> T {
        self.get_header().root_as_target
    }

    fn get_base_part(&self, link: T) -> T {
        self.get_link(link).target
    }

    fn first_is_to_the_left_of_second_4(
        &self,
        source: T,
        target: T,
        first_source: T,
        first_target: T,
    ) -> bool {
        // Compare by target first, then by source if targets are equal
        if first_target != target {
            first_target < target
        } else {
            first_source < source
        }
    }

    fn first_is_to_the_left_of_second_3(
        &self,
        first: T,
        first_source: T,
        first_target: T,
    ) -> bool {
        let link = self.get_link(first);
        self.first_is_to_the_left_of_second_4(
            link.source,
            link.target,
            first_source,
            first_target,
        )
    }
}

impl<T: LinkType> LinksTree<T> for LinksTargetsAvlBalancedTree<T> {
    fn get_tree_root(&self) -> T {
        LinkAvlBalancedTreeBaseAbstract::get_tree_root(self)
    }

    fn clear_node(&mut self, node: T) {
        let link = self.get_mut_link(node);
        link.left_as_target = T::funty_zero();
        link.right_as_target = T::funty_zero();
        link.size_as_target = T::funty_zero();
        link.balance_as_target = T::funty_zero();
    }

    fn get_left(&self, node: T) -> T {
        self.get_link(node).left_as_target
    }

    fn get_right(&self, node: T) -> T {
        self.get_link(node).right_as_target
    }

    fn get_size(&self, node: T) -> T {
        self.get_link(node).size_as_target
    }

    fn set_left(&mut self, node: T, left: T) {
        self.get_mut_link(node).left_as_target = left;
    }

    fn set_right(&mut self, node: T, right: T) {
        self.get_mut_link(node).right_as_target = right;
    }

    fn set_size(&mut self, node: T, size: T) {
        self.get_mut_link(node).size_as_target = size;
    }
}

impl<T: LinkType> AvlTree<T> for LinksTargetsAvlBalancedTree<T> {
    fn get_balance(&self, node: T) -> T {
        if node == T::funty_zero() {
            T::funty_zero()
        } else {
            self.get_link(node).balance_as_target
        }
    }

    fn set_balance(&mut self, node: T, balance: T) {
        if node != T::funty_zero() {
            self.get_mut_link(node).balance_as_target = balance;
        }
    }

    fn get_height(&self, node: T) -> T {
        if node == T::funty_zero() {
            T::funty_zero()
        } else {
            // Calculate height from balance and child heights
            let left_height = self.get_height(self.get_left(node));
            let right_height = self.get_height(self.get_right(node));
            if left_height > right_height {
                left_height + T::funty_one()
            } else {
                right_height + T::funty_one()
            }
        }
    }

    fn update_balance(&mut self, node: T) {
        if node != T::funty_zero() {
            let left_height = self.get_height(self.get_left(node));
            let right_height = self.get_height(self.get_right(node));
            // Balance factor = right_height - left_height
            let balance = if right_height >= left_height {
                right_height - left_height
            } else {
                // Handle underflow by using two's complement representation
                T::funty_zero() - (left_height - right_height)
            };
            self.set_balance(node, balance);
        }
    }
}