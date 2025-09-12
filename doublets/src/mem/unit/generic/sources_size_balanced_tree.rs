use std::{marker::PhantomData, ptr::NonNull};

use crate::{
    mem::{
        header::LinksHeader,
        unit::{generic::LinkSizeBalancedTreeBaseAbstract, raw_link::LinkPart},
        LinksTree,
    },
    Link,
};
use data::LinksConstants;
use methods::SzbTree;
use num::LinkType;

/// Size balanced tree implementation for managing sources of links
pub struct LinksSourcesSizeBalancedTree<T: LinkType> {
    pub mem: NonNull<[LinkPart<T>]>,
    pub header: NonNull<LinksHeader<T>>,
    pub r#break: T,
    pub r#continue: T,

    _phantom: PhantomData<T>,
}

impl<T: LinkType> LinksSourcesSizeBalancedTree<T> {
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

impl<T: LinkType> LinkSizeBalancedTreeBaseAbstract<T> for LinksSourcesSizeBalancedTree<T> {
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
        self.get_header().root_as_source
    }

    fn get_base_part(&self, link: T) -> T {
        self.get_link(link).source
    }

    fn first_is_to_the_left_of_second_4(
        &self,
        source: T,
        target: T,
        first_source: T,
        first_target: T,
    ) -> bool {
        // Compare by source first, then by target if sources are equal
        if first_source != source {
            first_source < source
        } else {
            first_target < target
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

impl<T: LinkType> LinksTree<T> for LinksSourcesSizeBalancedTree<T> {
    fn get_tree_root(&self) -> T {
        LinkSizeBalancedTreeBaseAbstract::get_tree_root(self)
    }

    fn clear_node(&mut self, node: T) {
        let link = self.get_mut_link(node);
        link.left_as_source = T::funty_zero();
        link.right_as_source = T::funty_zero();
        link.size_as_source = T::funty_zero();
    }

    fn get_left(&self, node: T) -> T {
        self.get_link(node).left_as_source
    }

    fn get_right(&self, node: T) -> T {
        self.get_link(node).right_as_source
    }

    fn get_size(&self, node: T) -> T {
        self.get_link(node).size_as_source
    }

    fn set_left(&mut self, node: T, left: T) {
        self.get_mut_link(node).left_as_source = left;
    }

    fn set_right(&mut self, node: T, right: T) {
        self.get_mut_link(node).right_as_source = right;
    }

    fn set_size(&mut self, node: T, size: T) {
        self.get_mut_link(node).size_as_source = size;
    }
}

impl<T: LinkType> SzbTree<T> for LinksSourcesSizeBalancedTree<T> {
    fn maintain(&mut self, node: T, flag: bool) {
        if node == T::funty_zero() {
            return;
        }

        let left = self.get_left(node);
        let right = self.get_right(node);

        if !flag {
            // Left side was modified
            if left != T::funty_zero() {
                let left_left = self.get_left(left);
                let left_right = self.get_right(left);
                
                if self.get_size_or_zero(left_left) > self.get_size_or_zero(right) {
                    self.rotate_right(node);
                } else if self.get_size_or_zero(left_right) > self.get_size_or_zero(right) {
                    self.rotate_left(left);
                    self.rotate_right(node);
                } else {
                    return;
                }
            }
        } else {
            // Right side was modified
            if right != T::funty_zero() {
                let right_left = self.get_left(right);
                let right_right = self.get_right(right);
                
                if self.get_size_or_zero(right_right) > self.get_size_or_zero(left) {
                    self.rotate_left(node);
                } else if self.get_size_or_zero(right_left) > self.get_size_or_zero(left) {
                    self.rotate_right(right);
                    self.rotate_left(node);
                } else {
                    return;
                }
            }
        }

        // Update sizes after rotation
        self.update_size(node);
        if left != T::funty_zero() {
            self.maintain(left, false);
            self.maintain(left, true);
        }
        if right != T::funty_zero() {
            self.maintain(right, false);
            self.maintain(right, true);
        }
    }

    fn rotate_left(&mut self, node: T) {
        if node == T::funty_zero() {
            return;
        }
        
        let right = self.get_right(node);
        if right == T::funty_zero() {
            return;
        }

        let right_left = self.get_left(right);
        
        self.set_right(node, right_left);
        self.set_left(right, node);
        
        self.update_size(node);
        self.update_size(right);
    }

    fn rotate_right(&mut self, node: T) {
        if node == T::funty_zero() {
            return;
        }
        
        let left = self.get_left(node);
        if left == T::funty_zero() {
            return;
        }

        let left_right = self.get_right(left);
        
        self.set_left(node, left_right);
        self.set_right(left, node);
        
        self.update_size(node);
        self.update_size(left);
    }

    fn update_size(&mut self, node: T) {
        if node != T::funty_zero() {
            let left_size = self.get_size_or_zero(self.get_left(node));
            let right_size = self.get_size_or_zero(self.get_right(node));
            self.set_size(node, left_size + right_size + T::funty_one());
        }
    }
}