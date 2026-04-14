use std::ptr::NonNull;

use crate::mem::traits::LinksTree;

use crate::mem::{
    split::{
        generic::internal_recursion_less_base::{
            InternalRecursionlessSizeBalancedTreeBase,
            InternalRecursionlessSizeBalancedTreeBaseAbstract,
        },
        DataPart, IndexPart,
    },
    SplitUpdateMem,
};

use crate::{mem::SplitTree, Link};
use data::{Flow, LinkType, LinksConstants};
use trees::{IterativeSizeBalancedTree, RecursiveSizeBalancedTree};

pub struct InternalSourcesRecursionlessTree<T: LinkType + crate::TreesLinkType> {
    base: InternalRecursionlessSizeBalancedTreeBase<T>,
}

impl<T: LinkType + crate::TreesLinkType> InternalSourcesRecursionlessTree<T> {
    pub fn new(
        constants: LinksConstants<T>,
        data: NonNull<[DataPart<T>]>,
        indexes: NonNull<[IndexPart<T>]>,
    ) -> Self {
        Self {
            base: InternalRecursionlessSizeBalancedTreeBase::new(constants, data, indexes),
        }
    }
}

impl<T: LinkType + crate::TreesLinkType> RecursiveSizeBalancedTree<T> for InternalSourcesRecursionlessTree<T> {
    unsafe fn get_left_reference(&self, node: T) -> *const T {
        std::ptr::addr_of!(self.get_index_part(node).left_as_source)
    }

    unsafe fn get_right_reference(&self, node: T) -> *const T {
        std::ptr::addr_of!(self.get_index_part(node).right_as_source)
    }

    unsafe fn get_mut_left_reference(&mut self, node: T) -> *mut T {
        std::ptr::addr_of_mut!(self.get_mut_index_part(node).left_as_source)
    }

    unsafe fn get_mut_right_reference(&mut self, node: T) -> *mut T {
        std::ptr::addr_of_mut!(self.get_mut_index_part(node).right_as_source)
    }

    unsafe fn get_left(&self, node: T) -> T {
        self.get_index_part(node).left_as_source
    }

    unsafe fn get_right(&self, node: T) -> T {
        self.get_index_part(node).right_as_source
    }

    unsafe fn get_size(&self, node: T) -> T {
        self.get_index_part(node).size_as_source
    }

    unsafe fn set_left(&mut self, node: T, left: T) {
        self.get_mut_index_part(node).left_as_source = left;
    }

    unsafe fn set_right(&mut self, node: T, right: T) {
        self.get_mut_index_part(node).right_as_source = right;
    }

    unsafe fn set_size(&mut self, node: T, size: T) {
        self.get_mut_index_part(node).size_as_source = size;
    }

    unsafe fn first_is_to_the_left_of_second(&self, first: T, second: T) -> bool {
        self.get_key_part(first) < self.get_key_part(second)
    }

    unsafe fn first_is_to_the_right_of_second(&self, first: T, second: T) -> bool {
        self.get_key_part(first) > self.get_key_part(second)
    }

    unsafe fn clear_node(&mut self, node: T) {
        let link = self.get_mut_index_part(node);
        link.left_as_source = <T as data::FuntyPart>::funty(0);
        link.right_as_source = <T as data::FuntyPart>::funty(0);
        link.size_as_source = <T as data::FuntyPart>::funty(0);
    }
}

impl<T: LinkType + crate::TreesLinkType> IterativeSizeBalancedTree<T> for InternalSourcesRecursionlessTree<T> {}

fn each_usages_core<T: LinkType + crate::TreesLinkType, H: FnMut(Link<T>) -> Flow + ?Sized>(
    this: &InternalSourcesRecursionlessTree<T>,
    base: T,
    link: T,
    handler: &mut H,
) -> Flow {
    if link == <T as data::FuntyPart>::funty(0) {
        return Flow::Continue;
    }
    unsafe {
        if each_usages_core(this, base, this.get_left_or_default(link), handler).is_break() { return Flow::Break; }
        if handler(this.get_link_value(link)).is_break() { return Flow::Break; }
        if each_usages_core(this, base, this.get_right_or_default(link), handler).is_break() { return Flow::Break; }
        Flow::Continue
    }
}

impl<T: LinkType + crate::TreesLinkType> LinksTree<T> for InternalSourcesRecursionlessTree<T> {
    fn count_usages(&self, link: T) -> T {
        self.count_usages_core(link)
    }

    fn search(&self, source: T, target: T) -> T {
        self.search_core(self.get_tree_root(source), target)
    }

    fn each_usages<H: FnMut(Link<T>) -> Flow + ?Sized>(&self, root: T, handler: &mut H) -> Flow {
        each_usages_core(self, root, self.get_tree_root(root), handler)
    }

    fn detach(&mut self, root: &mut T, index: T) {
        unsafe { IterativeSizeBalancedTree::detach(self, root as *mut _, index) }
    }

    fn attach(&mut self, root: &mut T, index: T) {
        unsafe { IterativeSizeBalancedTree::attach(self, root as *mut _, index) }
    }
}

impl<T: LinkType + crate::TreesLinkType> SplitUpdateMem<T> for InternalSourcesRecursionlessTree<T> {
    fn update_mem(&mut self, data: NonNull<[DataPart<T>]>, index: NonNull<[IndexPart<T>]>) {
        self.base.data = data;
        self.base.indexes = index;
    }
}

impl<T: LinkType + crate::TreesLinkType> SplitTree<T> for InternalSourcesRecursionlessTree<T> {}

impl<T: LinkType + crate::TreesLinkType> InternalRecursionlessSizeBalancedTreeBaseAbstract<T>
    for InternalSourcesRecursionlessTree<T>
{
    fn get_index_part(&self, link: T) -> &IndexPart<T> {
        unsafe { &self.base.indexes.as_ref()[link.as_usize()] }
    }

    fn get_mut_index_part(&mut self, link: T) -> &mut IndexPart<T> {
        unsafe { &mut self.base.indexes.as_mut()[link.as_usize()] }
    }

    fn get_data_part(&self, link: T) -> &DataPart<T> {
        unsafe { &self.base.data.as_ref()[link.as_usize()] }
    }

    fn get_mut_data_part(&mut self, link: T) -> &mut DataPart<T> {
        unsafe { &mut self.base.data.as_mut()[link.as_usize()] }
    }

    fn get_tree_root(&self, link: T) -> T {
        self.get_index_part(link).root_as_source
    }

    fn get_base_part(&self, link: T) -> T {
        self.get_data_part(link).source
    }

    fn get_key_part(&self, link: T) -> T {
        self.get_data_part(link).target
    }
}
