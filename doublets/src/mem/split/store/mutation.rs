//! Index maintenance of [`Store`] mutations: update, release, and moving
//! entries between the internal and external indexes when IDs are created or
//! released.

use std::cmp::Ordering;

use super::Store;
use crate::{
    mem::{
        split::{DataPart, IndexPart},
        SplitList, SplitTree,
    },
    DoubletsExt, Link, WriteHandler,
};
use data::{Flow, LinkReference};
use mem::RawMem;

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
    pub(super) fn resolve_dangling_internal(&mut self, index: T) {
        let any = self.constants.any;
        let links: Vec<_> = self
            .each_iter([any, index, any])
            .filter(|link| link.index != index)
            .collect();
        let sources = links.len();
        let sized = USE_LIST && self.list_head_stores_size(index);
        for link in links {
            unsafe {
                if USE_LIST {
                    self.detach_list_source(index, link.index, sized);
                } else {
                    self.detach_internal_source(index, link.index);
                }
                self.enter_external_sources(link.index);
            }
        }

        let links: Vec<_> = self
            .each_iter([any, any, index])
            .filter(|link| link.index != index)
            .filter(|link| !link.is_full())
            .collect();
        let targets = links.len();
        for link in links {
            unsafe {
                self.detach_internal_target(index, link.index);
                self.attach_external_target(link.index);
            }
        }
        if (sources != 0 || targets != 0)
            && std::env::var_os("DOUBLETS_TRACE_SPLIT_INDEXES").is_some()
        {
            eprintln!("[doublets split] ID {index}: moved {sources} source and {targets} target entries from internal to external indexes");
        }
    }

    pub(super) fn resolve_dangling_external(&mut self, free: T) {
        // `free` is already live. Normal queries now select its internal trees,
        // but references written before allocation still live in the external
        // trees. Collect from those trees before changing their structure.
        let mut links = Vec::new();
        self.external_sources.each_usages(free, &mut |link| {
            links.push(link.index);
            Flow::Continue
        });
        let sources = links.len();
        let sized = USE_LIST && self.list_head_stores_size(free);
        for link in links {
            unsafe {
                self.leave_external_sources(link);
                if USE_LIST {
                    self.attach_list_source(free, link, sized);
                } else {
                    self.attach_internal_source(free, link);
                }
            }
        }

        let mut links = Vec::new();
        self.external_targets.each_usages(free, &mut |link| {
            links.push(link.index);
            Flow::Continue
        });
        let targets = links.len();
        for link in links {
            unsafe {
                self.detach_external_target(link);
                self.attach_internal_target(free, link);
            }
        }
        debug_assert_eq!(self.external_sources.count_usages(free), T::from_byte(0));
        debug_assert_eq!(self.external_targets.count_usages(free), T::from_byte(0));
        if (sources != 0 || targets != 0)
            && std::env::var_os("DOUBLETS_TRACE_SPLIT_INDEXES").is_some()
        {
            eprintln!("[doublets split] ID {free}: moved {sources} source and {targets} target entries from external to internal indexes");
        }
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
    > crate::mem::Release<T> for Store<T, MD, MI, IS, ES, IT, ET, UL, USE_LIST>
{
    #[inline]
    unsafe fn release_unchecked(&mut self, index: T, handler: WriteHandler<'_, T>) -> Flow {
        self.resolve_dangling_internal(index);

        let header = self.get_header();

        match index.cmp(&header.allocated) {
            Ordering::Less => self.unused.attach_as_first(index),
            Ordering::Greater => unreachable!(),
            Ordering::Equal => {
                let allocated = self.get_header().allocated;
                let header = self.mut_header();
                header.allocated = allocated - T::from_byte(1);

                loop {
                    let allocated = self.get_header().allocated;
                    if !(allocated > T::from_byte(0) && self.is_unused(allocated)) {
                        break;
                    }
                    self.unused.detach(allocated);
                    self.mut_header().allocated = allocated - T::from_byte(1);
                }
            }
        }
        handler(
            Link::new(index, T::from_byte(0), T::from_byte(0)),
            Link::nothing(),
        )
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
    > crate::mem::UncheckedMutation<T> for Store<T, MD, MI, IS, ES, IT, ET, UL, USE_LIST>
{
    #[inline]
    unsafe fn update_unchecked(
        &mut self,
        index: T,
        source: T,
        target: T,
        handler: WriteHandler<'_, T>,
    ) -> Flow {
        #[cfg(test)]
        {
            self.update_calls += 1;
        }
        let null = T::from_byte(0);
        let old = self.get_data_part(index).clone();
        // Detaching a self reference temporarily clears this link's tree
        // sizes. is_unused uses those sizes, so classify the endpoints and the
        // list heads while the original indexes still describe the link as live.
        let virtual_source = self.is_virtual(source);
        let virtual_target = self.is_virtual(target);
        let old_virtual_source = old.source != null && self.is_virtual(old.source);
        let old_virtual_target = old.target != null && self.is_virtual(old.target);
        // After the update `index` is its own source and stores its list size.
        let old_head_sized = USE_LIST
            && old.source != null
            && !old_virtual_source
            && self.list_head_stores_size(old.source);
        let new_head_sized = USE_LIST
            && source != null
            && !virtual_source
            && (source == index || self.list_head_stores_size(source));
        // SAFETY: the caller guarantees that `index` is live and the indexes are
        // consistent, so it is a member of the indexes of its current nonzero
        // endpoints. Once detached, it is attached to those of the new ones.
        unsafe {
            if old.source != null {
                if old_virtual_source {
                    self.leave_external_sources(index);
                } else if USE_LIST {
                    self.detach_list_source(old.source, index, old_head_sized);
                } else {
                    self.detach_internal_source(old.source, index);
                }
            }
            if old.target != null {
                if old_virtual_target {
                    self.detach_external_target(index);
                } else {
                    self.detach_internal_target(old.target, index);
                }
            }
            let place = self.mut_data_part(index);
            (place.source, place.target) = (source, target);
            if source != null {
                if virtual_source {
                    self.enter_external_sources(index);
                } else if USE_LIST {
                    self.attach_list_source(source, index, new_head_sized);
                } else {
                    self.attach_internal_source(source, index);
                }
            }
            if target != null {
                if virtual_target {
                    self.attach_external_target(index);
                } else {
                    self.attach_internal_target(target, index);
                }
            }
        }
        handler(
            Link::new(index, old.source, old.target),
            Link::new(index, source, target),
        )
    }
}
