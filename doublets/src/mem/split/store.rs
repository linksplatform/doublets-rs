use std::{mem::transmute, ptr::NonNull};

use crate::{
    mem::{
        split::{
            DataPart, ExternalSourcesRecursionlessTree, ExternalTargetsRecursionlessTree,
            IndexPart, InternalSourcesLinkedList, InternalSourcesRecursionlessTree,
            InternalTargetsRecursionlessTree, UnusedLinks,
        },
        LinksHeader, SplitList, SplitTree, SplitUpdateMem,
    },
    Doublets, Link, Links, LinksError, ReadHandler, WriteHandler,
};
use data::{Flow, LinkReference, LinksConstants, ToQuery};
use mem::RawMem;

const DEFAULT_PAGE_SIZE: usize = 8 * 1024;

mod mutation;
mod sources_list;

#[cfg(test)]
#[path = "../../../tests/internal/split_sources_list.rs"]
mod sources_list_tests;
#[cfg(test)]
#[path = "../../../tests/internal/split.rs"]
mod tests;

/// A split-memory doublets store that keeps link data and index metadata in separate regions.
///
/// `MD` holds the raw `(source, target)` data; `MI` holds the tree-index structures.
/// Use [`Store::new`] for default constants or [`Store::with_constants`] for custom ones.
/// References to absent IDs use external indexes. Creating those IDs moves the
/// entries to internal indexes within this store; link data stays the same.
/// Set `DOUBLETS_TRACE_SPLIT_INDEXES` to log this reindexing to stderr
/// (disabled by default).
/// `delete*` methods use a resetting convenience adapter. For release without
/// automatic reset, use [`crate::mem::Release`] or [`crate::decorators::CheckedRelease`].
///
/// `USE_LIST` selects how links with an internal source are indexed by that
/// source. With `true` (the default, like `useLinkedList: true` of the C#
/// `SplitMemoryLinks`) they form a circular linked list in insertion order:
/// attach and detach are O(1), and searching a `(source, target)` pair uses the
/// targets tree. With `false` they form a size-balanced tree of the `IS` type,
/// which is ordered by target. [`TreeStore`] names the store with `false`.
/// The two layouts store different index metadata, so open an index memory
/// only with the mode that wrote it.
pub struct Store<
    T: LinkReference,
    MD: RawMem<Item = DataPart<T>>,
    MI: RawMem<Item = IndexPart<T>>,
    IS: SplitTree<T> = InternalSourcesRecursionlessTree<T>,
    ES: SplitTree<T> = ExternalSourcesRecursionlessTree<T>,
    IT: SplitTree<T> = InternalTargetsRecursionlessTree<T>,
    ET: SplitTree<T> = ExternalTargetsRecursionlessTree<T>,
    UL: SplitList<T> = UnusedLinks<T>,
    const USE_LIST: bool = true,
> {
    data_mem: MD,
    index_mem: MI,

    data_ptr: NonNull<[DataPart<T>]>,
    index_ptr: NonNull<[IndexPart<T>]>,

    data_step: usize,
    index_step: usize,

    constants: LinksConstants<T>,

    internal_sources: IS,
    pub external_sources: ES,
    internal_targets: IT,
    pub external_targets: ET,

    sources_list: InternalSourcesLinkedList<T>,
    unused: UL,
    #[cfg(test)]
    update_calls: usize,
    /// Test-only count of contract lookups through [`Doublets::get_link`].
    #[cfg(test)]
    link_lookups: std::sync::atomic::AtomicUsize,
}

/// A split store that indexes links by internal source with a size-balanced
/// tree instead of the linked list (`USE_LIST = false`).
pub type TreeStore<T, MD, MI> = Store<
    T,
    MD,
    MI,
    InternalSourcesRecursionlessTree<T>,
    ExternalSourcesRecursionlessTree<T>,
    InternalTargetsRecursionlessTree<T>,
    ExternalTargetsRecursionlessTree<T>,
    UnusedLinks<T>,
    false,
>;

impl<
        T: LinkReference,
        MD: RawMem<Item = DataPart<T>>,
        MI: RawMem<Item = IndexPart<T>>,
        const USE_LIST: bool,
    >
    Store<
        T,
        MD,
        MI,
        InternalSourcesRecursionlessTree<T>,
        ExternalSourcesRecursionlessTree<T>,
        InternalTargetsRecursionlessTree<T>,
        ExternalTargetsRecursionlessTree<T>,
        UnusedLinks<T>,
        USE_LIST,
    >
{
    /// Creates a split store with the given data memory, index memory, and `constants`.
    pub fn with_constants(
        data_mem: MD,
        index_mem: MI,
        constants: LinksConstants<T>,
    ) -> Result<Self, LinksError<T>> {
        let dangling_data = NonNull::slice_from_raw_parts(NonNull::dangling(), 0);
        let dangling_index = NonNull::slice_from_raw_parts(NonNull::dangling(), 0);

        let internal_sources =
            InternalSourcesRecursionlessTree::new(constants.clone(), dangling_data, dangling_index);
        let external_sources =
            ExternalSourcesRecursionlessTree::new(constants.clone(), dangling_data, dangling_index);

        let internal_targets =
            InternalTargetsRecursionlessTree::new(constants.clone(), dangling_data, dangling_index);
        let external_targets =
            ExternalTargetsRecursionlessTree::new(constants.clone(), dangling_data, dangling_index);

        let sources_list =
            InternalSourcesLinkedList::new(constants.clone(), dangling_data, dangling_index);
        let unused = UnusedLinks::new(dangling_data, dangling_index);

        let mut new = Self {
            data_mem,
            index_mem,
            data_ptr: dangling_data,
            index_ptr: dangling_index,
            data_step: Self::SIZE_STEP,
            index_step: Self::SIZE_STEP,
            constants,
            internal_sources,
            external_sources,
            internal_targets,
            external_targets,
            sources_list,
            unused,
            #[cfg(test)]
            update_calls: 0,
            #[cfg(test)]
            link_lookups: std::sync::atomic::AtomicUsize::new(0),
        };

        // SAFETY: Without this, the code will become unsafe
        unsafe {
            new.init()?;
        }
        Ok(new)
    }

    /// Creates a split store using the given memories and default [`LinksConstants`].
    pub fn new(data_mem: MD, index_mem: MI) -> Result<Self, LinksError<T>> {
        Self::with_constants(data_mem, index_mem, LinksConstants::default())
    }
}

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
    #[cfg(not(miri))]
    const SIZE_STEP: usize = 2_usize.pow(20);
    #[cfg(miri)]
    const SIZE_STEP: usize = 2_usize.pow(10);

    fn mut_from_mem<'a, U>(mut ptr: NonNull<[U]>, index: usize) -> Option<&'a mut U> {
        if index < ptr.len() {
            // SAFETY: `ptr` is non-dangling slice
            Some(unsafe {
                let slice = ptr.as_mut();
                &mut slice[index]
            })
        } else {
            None
        }
    }

    fn get_from_mem<'a, U>(ptr: NonNull<[U]>, index: usize) -> Option<&'a U> {
        Self::mut_from_mem(ptr, index).map(|v| &*v)
    }

    fn get_header(&self) -> &LinksHeader<T> {
        // SAFETY: `LinksHeader` and `IndexPart` layout are equivalent
        unsafe {
            Self::get_from_mem(self.index_ptr, 0)
                .map(|x| transmute(x))
                .expect("Header should be in index memory")
        }
    }

    fn mut_header(&mut self) -> &mut LinksHeader<T> {
        // SAFETY: `LinksHeader` and `IndexPart` layout are equivalent
        unsafe {
            Self::mut_from_mem(self.index_ptr, 0)
                .map(|x| transmute(x))
                .expect("Header should be in index memory")
        }
    }

    /// Returns the raw data part (source/target) for the link at `index`.
    pub fn get_data_part(&self, index: T) -> &DataPart<T> {
        Self::get_from_mem(self.data_ptr, index.as_()).expect("Data part should be in data memory")
    }

    unsafe fn get_data_unchecked(&self, index: T) -> &DataPart<T> {
        Self::get_from_mem(self.data_ptr, index.as_()).unwrap_unchecked()
    }

    fn mut_data_part(&mut self, index: T) -> &mut DataPart<T> {
        Self::mut_from_mem(self.data_ptr, index.as_()).expect("Data part should be in data memory")
    }

    /// Returns the raw index part (tree metadata) for the link at `index`.
    pub fn get_index_part(&self, index: T) -> &IndexPart<T> {
        Self::get_from_mem(self.index_ptr, index.as_())
            .expect("Index part should be in index memory")
    }

    fn mut_index_part(&mut self, index: T) -> &mut IndexPart<T> {
        Self::mut_from_mem(self.index_ptr, index.as_())
            .expect("Index part should be in index memory")
    }

    unsafe fn mut_source_header_root(&mut self) -> *mut T {
        &mut self.mut_header().root_as_source
    }

    unsafe fn mut_target_header_root(&mut self) -> *mut T {
        &mut self.mut_header().root_as_target
    }

    unsafe fn mut_source_root(&mut self, link: T) -> *mut T {
        &mut self.mut_index_part(link).root_as_source
    }

    unsafe fn mut_target_root(&mut self, link: T) -> *mut T {
        &mut self.mut_index_part(link).root_as_target
    }

    unsafe fn detach_internal_source_unchecked(&mut self, root: *mut T, index: T) {
        self.internal_sources.detach(&mut *root, index);
    }

    unsafe fn detach_internal_target_unchecked(&mut self, root: *mut T, index: T) {
        self.internal_targets.detach(&mut *root, index);
    }

    unsafe fn attach_internal_source_unchecked(&mut self, root: *mut T, index: T) {
        self.internal_sources.attach(&mut *root, index);
    }

    unsafe fn attach_internal_target_unchecked(&mut self, root: *mut T, index: T) {
        self.internal_targets.attach(&mut *root, index);
    }

    unsafe fn detach_external_source_unchecked(&mut self, root: *mut T, index: T) {
        self.external_sources.detach(&mut *root, index);
    }

    unsafe fn detach_external_target_unchecked(&mut self, root: *mut T, index: T) {
        self.external_targets.detach(&mut *root, index);
    }

    unsafe fn attach_external_source_unchecked(&mut self, root: *mut T, index: T) {
        self.external_sources.attach(&mut *root, index);
    }

    unsafe fn attach_external_target_unchecked(&mut self, root: *mut T, index: T) {
        self.external_targets.attach(&mut *root, index);
    }

    unsafe fn detach_internal_source(&mut self, root: T, index: T) {
        let root = self.mut_source_root(root);
        self.detach_internal_source_unchecked(root, index);
    }

    unsafe fn detach_internal_target(&mut self, root: T, index: T) {
        let root = self.mut_target_root(root);
        self.detach_internal_target_unchecked(root, index);
    }

    unsafe fn attach_internal_source(&mut self, root: T, index: T) {
        let root = self.mut_source_root(root);
        self.attach_internal_source_unchecked(root, index);
    }

    unsafe fn attach_internal_target(&mut self, root: T, index: T) {
        let root = self.mut_target_root(root);
        self.attach_internal_target_unchecked(root, index);
    }

    unsafe fn detach_external_source(&mut self, index: T) {
        let root = self.mut_source_header_root();
        self.detach_external_source_unchecked(root, index);
    }

    unsafe fn detach_external_target(&mut self, index: T) {
        let root = self.mut_target_header_root();
        self.detach_external_target_unchecked(root, index);
    }

    unsafe fn attach_external_source(&mut self, index: T) {
        let root = self.mut_source_header_root();
        self.attach_external_source_unchecked(root, index);
    }

    unsafe fn attach_external_target(&mut self, index: T) {
        let root = self.mut_target_header_root();
        self.attach_external_target_unchecked(root, index);
    }

    fn update_mem(&mut self, data: NonNull<[DataPart<T>]>, index: NonNull<[IndexPart<T>]>) {
        self.data_ptr = data;
        self.index_ptr = index;

        self.internal_sources.update_mem(data, index);
        self.external_sources.update_mem(data, index);
        self.internal_targets.update_mem(data, index);
        self.external_targets.update_mem(data, index);
        self.sources_list.update_mem(data, index);
        self.unused.update_mem(data, index);
    }

    fn align(mut to_align: usize, target: usize) -> usize {
        debug_assert!(to_align >= target);

        // TODO: optimize this `if`
        if to_align % target != 0 {
            to_align = ((to_align / target) * target) + target;
        }
        to_align
    }

    unsafe fn init(&mut self) -> Result<(), LinksError<T>> {
        let data = NonNull::from(crate::mem::resize_mem(
            &mut self.data_mem,
            DEFAULT_PAGE_SIZE,
        )?);
        let index = NonNull::from(crate::mem::resize_mem(
            &mut self.index_mem,
            DEFAULT_PAGE_SIZE,
        )?);
        self.update_mem(data, index);

        let header = self.get_header().clone();
        let allocated = header.allocated.as_();

        let mut data_capacity = allocated;
        data_capacity = data_capacity.max(self.data_mem.allocated().len());
        data_capacity = data_capacity.max(self.data_step);

        let mut index_capacity = allocated;
        index_capacity = index_capacity.max(self.index_mem.allocated().len());
        index_capacity = index_capacity.max(self.index_step);

        data_capacity = Self::align(data_capacity, self.data_step);
        index_capacity = Self::align(index_capacity, self.index_step);

        let data = NonNull::from(crate::mem::resize_mem(&mut self.data_mem, data_capacity)?);
        let index = NonNull::from(crate::mem::resize_mem(&mut self.index_mem, index_capacity)?);
        self.update_mem(data, index);

        self.mut_header().reserved =
            T::try_from(self.data_mem.allocated().len() - 1).expect("always ok");
        Ok(())
    }

    fn total(&self) -> T {
        let header = self.get_header();
        header.allocated - header.free
    }

    /// Returns `true` if the slot at `link` is in the free-list (deleted but not yet reused).
    pub fn is_unused(&self, link: T) -> bool {
        let header = self.get_header();
        if link <= header.allocated && header.first_free != link {
            let index = self.get_index_part(link);
            let data = self.get_data_part(link);
            // Free-list slots have no endpoint-index entries; their data holds
            // circular-list pointers. A live (source, null) link has no target
            // entry either, but is still present in its source tree or, in
            // list mode, in its circular source list (a nonzero next pointer).
            index.size_as_source == T::from_byte(0)
                && index.size_as_target == T::from_byte(0)
                && !(USE_LIST && index.right_as_source != T::from_byte(0))
                && data.source != T::from_byte(0)
        } else {
            true
        }
    }

    //fn is_non_

    /// Returns `true` if `link` is a virtual (external/unused) reference.
    pub fn is_virtual(&self, link: T) -> bool {
        self.is_unused(link)
    }

    /// Returns `true` if `link` is an allocated, non-deleted internal link.
    pub fn exists(&self, link: T) -> bool {
        let constants = self.constants();
        let header = self.get_header();

        // TODO: use attributes expressions feature
        // TODO: use `Range::contains`
        link >= *constants.internal_range.start()
            && link <= header.allocated
            && !self.is_unused(link)
    }

    // SAFETY: must be link exists
    unsafe fn get_link_unchecked(&self, index: T) -> Link<T> {
        debug_assert!(self.exists(index));

        let raw = self.get_data_unchecked(index);
        Link::new(index, raw.source, raw.target)
    }

    /// Finds the link `(source, target)` with the index that the endpoint
    /// kinds and the source mode select; returns `null` if there is none.
    fn search_pair(&self, source: T, target: T, virtual_source: bool, virtual_target: bool) -> T {
        if virtual_source && virtual_target {
            self.external_sources.search(source, target)
        } else if virtual_source {
            self.internal_targets.search(source, target)
        } else if virtual_target {
            // An internal source with a virtual target: in list mode the
            // sources are not a search tree, so use the external targets tree.
            if USE_LIST {
                self.external_targets.search(source, target)
            } else {
                self.internal_sources.search(source, target)
            }
        } else if USE_LIST
            || self.internal_sources.count_usages(source)
                > self.internal_targets.count_usages(target)
        {
            self.internal_targets.search(source, target)
        } else {
            self.internal_sources.search(source, target)
        }
    }

    fn try_each_by_core(&self, handler: ReadHandler<'_, T>, query: &[T]) -> Flow {
        let query = query.to_query();

        if query.is_empty() {
            let mut index = T::from_byte(1);
            let allocated = self.get_header().allocated;
            while index <= allocated {
                if let Some(link) = self.get_link(index) {
                    if handler(link).is_break() {
                        return Flow::Break;
                    }
                }
                index = index + T::from_byte(1);
            }
            return Flow::Continue;
        }

        let constants = self.constants.clone();
        let any = constants.any;
        let index = query[constants.index_part.as_()];
        if query.len() == 1 {
            return if index == any {
                self.try_each_by_core(handler, &[])
            } else if let Some(link) = self.get_link(index) {
                handler(link)
            } else {
                Flow::Continue
            };
        }
        //
        if query.len() == 2 {
            let value = query[1];
            return if index == any {
                if value == any {
                    self.try_each_by_core(handler, &[])
                } else {
                    if self
                        .try_each_by_core(handler, &[index, value, any])
                        .is_break()
                    {
                        return Flow::Break;
                    }
                    self.try_each_by_core(handler, &[index, any, value])
                }
            } else if let Some(link) = self.get_link(index) {
                if value == any || link.source == value || link.target == value {
                    handler(link)
                } else {
                    Flow::Continue
                }
            } else {
                Flow::Continue
            };
        }
        //
        if query.len() == 3 {
            let source = query[constants.source_part.as_()];
            let target = query[constants.target_part.as_()];
            let is_virtual_source = self.is_virtual(source);
            let is_virtual_target = self.is_virtual(target);

            return if index == any {
                if (source, target) == (any, any) {
                    self.try_each_by_core(handler, &[])
                } else if source == any {
                    if is_virtual_target {
                        self.external_targets.each_usages(target, handler)
                    } else {
                        self.internal_targets.each_usages(target, handler)
                    }
                } else if target == any {
                    if is_virtual_source {
                        self.external_sources.each_usages(source, handler)
                    } else if USE_LIST {
                        self.sources_list.each_usages(source, handler)
                    } else {
                        self.internal_sources.each_usages(source, handler)
                    }
                } else {
                    let link =
                        self.search_pair(source, target, is_virtual_source, is_virtual_target);
                    return if link == constants.null {
                        Flow::Continue
                    } else {
                        // SAFETY: link 100% exists
                        let link = unsafe { self.get_link(link).unwrap_unchecked() };
                        handler(link)
                    };
                }
            } else if let Some(link) = self.get_link(index) {
                if (source, target) == (any, any) {
                    handler(link)
                } else if source != any && target != any {
                    if (link.source, link.target) == (source, target) {
                        handler(link)
                    } else {
                        Flow::Continue
                    }
                } else if source == any {
                    if link.target == target {
                        handler(link)
                    } else {
                        Flow::Continue
                    }
                } else if target == any {
                    if link.source == source {
                        handler(link)
                    } else {
                        Flow::Continue
                    }
                } else {
                    Flow::Continue
                }
            } else {
                Flow::Continue
            };
        }
        todo!()
    }
}

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
    > Links<T> for Store<T, MD, MI, IS, ES, IT, ET, UL, USE_LIST>
{
    fn constants(&self) -> &LinksConstants<T> {
        &self.constants
    }

    fn count_links(&self, query: &[T]) -> T {
        if query.is_empty() {
            return self.total();
        }

        let constants = self.constants();
        let any = constants.any;
        let index = query[constants.index_part.as_()];
        if query.len() == 1 {
            return if index == any {
                self.total()
            } else if self.exists(index) {
                T::from_byte(1)
            } else {
                T::from_byte(0)
            };
        }

        if query.len() == 2 {
            let value = query[1];
            let is_virtual_val = self.is_virtual(value);

            return if index == any {
                if value == any {
                    self.total()
                } else if is_virtual_val {
                    self.external_sources.count_usages(value)
                        + self.external_targets.count_usages(value)
                } else if USE_LIST {
                    self.count_list_usages(value) + self.internal_targets.count_usages(value)
                } else {
                    self.internal_sources.count_usages(value)
                        + self.internal_targets.count_usages(value)
                }
            } else if !self.exists(index) {
                T::from_byte(0)
            } else if value == any {
                T::from_byte(1)
            } else {
                let stored = self.get_data_part(index);
                if (stored.source, stored.target) == (value, value) {
                    T::from_byte(1)
                } else {
                    T::from_byte(0)
                }
            };
        }

        if query.len() == 3 {
            let source = query[constants.source_part.as_()];
            let target = query[constants.target_part.as_()];

            let is_virtual_source = self.is_virtual(source);
            let is_virtual_target = self.is_virtual(target);

            return if index == any {
                if (source, target) == (any, any) {
                    self.total()
                } else if source == any {
                    if is_virtual_target {
                        self.external_targets.count_usages(target)
                    } else {
                        self.internal_targets.count_usages(target)
                    }
                } else if target == any {
                    if is_virtual_source {
                        self.external_sources.count_usages(source)
                    } else if USE_LIST {
                        self.count_list_usages(source)
                    } else {
                        self.internal_sources.count_usages(source)
                    }
                } else {
                    let link =
                        self.search_pair(source, target, is_virtual_source, is_virtual_target);
                    return if link == constants.null {
                        T::from_byte(0)
                    } else {
                        T::from_byte(1)
                    };
                }
            } else if !self.exists(index) {
                T::from_byte(0)
            } else if (source, target) == (any, any) {
                T::from_byte(1)
            } else {
                let link = unsafe { self.get_link_unchecked(index) };
                if source != any && target != any {
                    if (link.source, link.target) == (source, target) {
                        T::from_byte(1)
                    } else {
                        T::from_byte(0)
                    }
                } else if source == any {
                    if link.target == target {
                        T::from_byte(1)
                    } else {
                        T::from_byte(0)
                    }
                } else if target == any {
                    if link.source == source {
                        T::from_byte(1)
                    } else {
                        T::from_byte(0)
                    }
                } else {
                    T::from_byte(0)
                }
            };
        }

        todo!()
    }

    fn create_links(
        &mut self,
        _query: &[T],
        handler: WriteHandler<'_, T>,
    ) -> Result<Flow, LinksError<T>> {
        let constants = self.constants().clone();
        let header = self.get_header();
        let mut free = header.first_free;
        if free == constants.null {
            let max_inner = *constants.internal_range.end();
            if header.allocated >= max_inner {
                return Err(LinksError::LimitReached(max_inner));
            }

            if header.allocated >= header.reserved - T::from_byte(1) {
                let new_data_cap = self.data_mem.allocated().len() + self.data_step;
                let new_index_cap = self.index_mem.allocated().len() + self.index_step;
                let data = NonNull::from(crate::mem::resize_mem(&mut self.data_mem, new_data_cap)?);
                let index =
                    NonNull::from(crate::mem::resize_mem(&mut self.index_mem, new_index_cap)?);
                self.update_mem(data, index);
                let reserved = self.index_mem.allocated().len();
                let header = self.mut_header();
                header.reserved = T::try_from(reserved).expect("always ok");
            }
            let header = self.mut_header();
            header.allocated = header.allocated + T::from_byte(1);
            free = header.allocated;
        } else {
            self.unused.detach(free);
        }

        self.resolve_dangling_external(free);

        Ok(handler(
            Link::nothing(),
            Link::new(free, T::from_byte(0), T::from_byte(0)),
        ))
    }

    fn each_links(&self, query: &[T], handler: ReadHandler<'_, T>) -> Flow {
        self.try_each_by_core(handler, query)
    }

    fn update_links(
        &mut self,
        query: &[T],
        change: &[T],
        handler: WriteHandler<'_, T>,
    ) -> Result<Flow, LinksError<T>> {
        crate::decorators::CheckedMutation::new(self).update_links(query, change, handler)
    }

    fn delete_links(
        &mut self,
        query: &[T],
        handler: WriteHandler<'_, T>,
    ) -> Result<Flow, LinksError<T>> {
        crate::decorators::CheckedMutation::new(self).delete_links(query, handler)
    }
}

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
    > Doublets<T> for Store<T, MD, MI, IS, ES, IT, ET, UL, USE_LIST>
{
    fn get_link(&self, index: T) -> Option<Link<T>> {
        #[cfg(test)]
        self.link_lookups
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if self.exists(index) {
            Some(unsafe { self.get_link_unchecked(index) })
        } else {
            None
        }
    }
}

unsafe impl<
        T: LinkReference,
        MD: RawMem<Item = DataPart<T>>,
        MI: RawMem<Item = IndexPart<T>>,
        IS: SplitTree<T>,
        ES: SplitTree<T>,
        IT: SplitTree<T>,
        ET: SplitTree<T>,
        UL: SplitList<T>,
        const USE_LIST: bool,
    > Sync for Store<T, MD, MI, IS, ES, IT, ET, UL, USE_LIST>
{
}

unsafe impl<
        T: LinkReference,
        MD: RawMem<Item = DataPart<T>>,
        MI: RawMem<Item = IndexPart<T>>,
        IS: SplitTree<T>,
        ES: SplitTree<T>,
        IT: SplitTree<T>,
        ET: SplitTree<T>,
        UL: SplitList<T>,
        const USE_LIST: bool,
    > Send for Store<T, MD, MI, IS, ES, IT, ET, UL, USE_LIST>
{
}
