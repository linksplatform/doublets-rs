use crate::{
    mem::unit::LinkPart,
    split::{DataPart, IndexPart},
    Link,
};
use data::{Flow, LinkReference};
use std::ptr::NonNull;

pub trait LinksTree<T: LinkReference> {
    fn count_usages(&self, root: T) -> T;

    fn search(&self, source: T, target: T) -> T;

    fn each_usages<H: FnMut(Link<T>) -> Flow + ?Sized>(&self, root: T, handler: &mut H) -> Flow;

    fn detach(&mut self, root: &mut T, index: T);

    fn attach(&mut self, root: &mut T, index: T);
}

pub trait UnitUpdateMem<T: LinkReference> {
    fn update_mem(&mut self, mem: NonNull<[LinkPart<T>]>);
}

pub trait UnitTree<T: LinkReference>: LinksTree<T> + UnitUpdateMem<T> {}

pub trait SplitUpdateMem<T: LinkReference> {
    fn update_mem(&mut self, data: NonNull<[DataPart<T>]>, index: NonNull<[IndexPart<T>]>);
}

pub trait SplitTree<T: LinkReference>: LinksTree<T> + SplitUpdateMem<T> {}

pub trait LinksList<T: LinkReference> {
    fn detach(&mut self, link: T);

    fn attach_as_first(&mut self, link: T);
}

pub trait UnitList<T: LinkReference>: LinksList<T> + UnitUpdateMem<T> {}

pub trait SplitList<T: LinkReference>: LinksList<T> + SplitUpdateMem<T> {}
