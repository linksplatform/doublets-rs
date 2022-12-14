use std::{default::default, marker::PhantomData, ptr::NonNull};

use crate::{
    mem::{header::LinksHeader, unit::raw_link::LinkPart, LinksTree},
    Link,
};
use data::{LinkType, LinksConstants};
use trees::NoRecurSzbTree;

// TODO: why is there so much duplication in OOP!!! FIXME
pub struct LinksRecursionlessSizeBalancedTreeBase<T: LinkType> {
    pub mem: NonNull<[LinkPart<T>]>,
    pub r#break: T,
    pub r#continue: T,

    _phantom: PhantomData<T>,
}

impl<T: LinkType> LinksRecursionlessSizeBalancedTreeBase<T> {
    pub fn new(constants: LinksConstants<T>, mem: NonNull<[LinkPart<T>]>) -> Self {
        Self {
            mem,
            r#break: constants.r#break,
            r#continue: constants.r#continue,
            _phantom: default(),
        }
    }
}

pub trait LinkRecursionlessSizeBalancedTreeBaseAbstract<T: LinkType>:
    NoRecurSzbTree<T> + LinksTree<T>
{
    fn get_header(&self) -> &LinksHeader<T>;

    fn get_mut_header(&mut self) -> &mut LinksHeader<T>;

    fn get_link(&self, link: T) -> &LinkPart<T>;

    fn get_mut_link(&mut self, link: T) -> &mut LinkPart<T>;

    fn get_tree_root(&self) -> T;

    fn get_base_part(&self, link: T) -> T;

    // TODO: rename
    fn first_is_to_the_left_of_second_4(
        &self,
        source: T,
        target: T,
        root_source: T,
        root_target: T,
    ) -> bool;

    fn first_is_to_the_right_of_second_4(
        &self,
        source: T,
        target: T,
        root_source: T,
        root_target: T,
    ) -> bool;

    fn get_link_value(&self, index: T) -> Link<T> {
        let link = self.get_link(index);
        Link::new(index, link.source, link.target)
    }
}
