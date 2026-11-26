//! Maintenance of the internal sources linked list (`USE_LIST = true`).
//!
//! A link `X` can be both the head of the list of links whose source is `X`
//! and a node of the external sources tree (when its own source is virtual).
//! Both roles use `X.size_as_source`: the tree keeps the subtree size there, so
//! the list of such a head does not store its size and is counted by walking
//! it. Every other head stores its list size, as in the C# implementation.

use super::Store;
use crate::mem::{
    split::{DataPart, IndexPart},
    SplitList, SplitTree,
};
use data::LinkReference;
use mem::RawMem;
use trees::RelativeCircularLinkedList;

impl<
        T: LinkReference,
        MD: RawMem<Item = DataPart<T>>,
        MI: RawMem<Item = IndexPart<T>>,
        IS: SplitTree<T>,
        ES: SplitTree<T>,
        IT: SplitTree<T>,
        ET: SplitTree<T>,
        UL: SplitList<T>,
        const USE_LIST: bool,
    > Store<T, MD, MI, IS, ES, IT, ET, UL, USE_LIST>
{
    /// Returns `true` if the live link `head` stores the size of its sources
    /// list, i.e. it is not a node of the external sources tree.
    pub(super) fn list_head_stores_size(&self, head: T) -> bool {
        let source = self.get_data_part(head).source;
        source == T::from_byte(0) || source == head || !self.is_virtual(source)
    }

    /// Returns the number of links in the sources list of the live link `head`.
    pub(super) fn count_list_usages(&self, head: T) -> T {
        if self.list_head_stores_size(head) {
            self.sources_list.count_usages(head)
        } else {
            self.sources_list.count_by_traversal(head)
        }
    }

    /// Appends `index` to the sources list of `head`. `sized` tells whether
    /// `head` stores its list size (see [`Self::list_head_stores_size`]).
    pub(super) fn attach_list_source(&mut self, head: T, index: T, sized: bool) {
        self.sources_list.set_size_tracking(sized);
        self.sources_list.attach_as_last(head, index);
        self.sources_list.set_size_tracking(true);
    }

    /// Removes `index` from the sources list of `head`. `sized` is as in
    /// [`Self::attach_list_source`].
    pub(super) fn detach_list_source(&mut self, head: T, index: T, sized: bool) {
        self.sources_list.set_size_tracking(sized);
        self.sources_list.detach(head, index);
        self.sources_list.set_size_tracking(true);
    }

    /// Attaches `index` to the external sources tree. In list mode its
    /// `size_as_source` stops holding its own list size first.
    pub(super) unsafe fn enter_external_sources(&mut self, index: T) {
        if USE_LIST {
            self.mut_index_part(index).size_as_source = T::from_byte(0);
        }
        self.attach_external_source(index);
    }

    /// Detaches `index` from the external sources tree. In list mode its
    /// `size_as_source` then holds its own list size again.
    pub(super) unsafe fn leave_external_sources(&mut self, index: T) {
        self.detach_external_source(index);
        if USE_LIST {
            let size = self.sources_list.count_by_traversal(index);
            self.mut_index_part(index).size_as_source = size;
        }
    }
}
