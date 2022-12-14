use std::ptr::NonNull;

use crate::mem::traits::LinksTree;

use crate::{
    mem::split::{DataPart, IndexPart},
    Link,
};
use data::{LinkType, LinksConstants};
use trees::NoRecurSzbTree;

// TODO: why is there so much duplication in OOP!!! FIXME
pub(crate) struct InternalRecursionlessSizeBalancedTreeBase<T: LinkType> {
    pub(crate) data: NonNull<[DataPart<T>]>,
    pub(crate) indexes: NonNull<[IndexPart<T>]>,
    pub(crate) r#break: T,
    pub(crate) r#continue: T,
}

impl<T: LinkType> InternalRecursionlessSizeBalancedTreeBase<T> {
    pub(crate) fn new(
        constants: LinksConstants<T>,
        data: NonNull<[DataPart<T>]>,
        indexes: NonNull<[IndexPart<T>]>,
    ) -> Self {
        Self {
            data,
            indexes,
            r#break: constants.r#break,
            r#continue: constants.r#continue,
        }
    }
}

pub(crate) trait InternalRecursionlessSizeBalancedTreeBaseAbstract<T: LinkType>:
    NoRecurSzbTree<T> + LinksTree<T>
{
    fn get_index_part(&self, link: T) -> &IndexPart<T>;

    fn get_mut_index_part(&mut self, link: T) -> &mut IndexPart<T>;

    fn get_data_part(&self, link: T) -> &DataPart<T>;

    fn get_mut_data_part(&mut self, link: T) -> &mut DataPart<T>;

    fn get_tree_root(&self, link: T) -> T;

    fn get_base_part(&self, link: T) -> T;

    fn get_key_part(&self, link: T) -> T;

    fn get_link_value(&self, index: T) -> Link<T> {
        let link = self.get_data_part(index);
        Link::new(index, link.source, link.target)
    }

    fn search_core(&self, mut root: T, key: T) -> T {
        unsafe {
            while root != T::funty(0) {
                let root_key = self.get_key_part(root);
                if key < root_key {
                    root = self.get_left_or_default(root);
                } else if key > root_key {
                    root = self.get_right_or_default(root);
                } else {
                    return root;
                }
            }
            T::funty(0)
        }
    }

    fn count_usages_core(&self, link: T) -> T {
        unsafe { self.get_size_or_zero(self.get_tree_root(link)) }
    }
}
