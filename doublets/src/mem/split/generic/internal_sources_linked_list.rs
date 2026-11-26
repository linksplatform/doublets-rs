use crate::Link;
use data::{Flow, LinksConstants};
use std::mem::transmute;

use std::ptr::NonNull;

use crate::mem::{
    header::LinksHeader,
    split::{DataPart, IndexPart},
    SplitUpdateMem,
};

use data::LinkReference;
use trees::{LinkedList, RelativeCircularLinkedList, RelativeLinkedList};

/// Circular doubly linked list of the links that share an internal source.
///
/// The head is the source link: its `root_as_source` is the first element and
/// its `size_as_source` the element count. Elements use `left_as_source` and
/// `right_as_source` as previous and next pointers.
///
/// A head that is itself a node of the external sources tree (its own source
/// is virtual) needs `size_as_source` for that tree. Disable size tracking
/// with [`Self::set_size_tracking`] while changing such a list and count it
/// with [`Self::count_by_traversal`] instead of [`Self::count_usages`].
pub struct InternalSourcesLinkedList<T: LinkReference> {
    data: NonNull<[DataPart<T>]>,
    indexes: NonNull<[IndexPart<T>]>,
    r#continue: T,
    r#break: T,
    track_size: bool,
}

impl<T: LinkReference> InternalSourcesLinkedList<T> {
    pub fn new(
        constants: LinksConstants<T>,
        data: NonNull<[DataPart<T>]>,
        indexes: NonNull<[IndexPart<T>]>,
    ) -> Self {
        Self {
            data,
            indexes,
            r#continue: constants.r#continue,
            r#break: constants.r#break,
            track_size: true,
        }
    }

    /// Selects whether `attach*`/`detach` update the stored size of the head.
    pub fn set_size_tracking(&mut self, track_size: bool) {
        self.track_size = track_size;
    }

    fn get_header(&self) -> &LinksHeader<T> {
        unsafe { transmute(&self.indexes.as_ref()[0]) }
    }

    fn get_mut_header(&mut self) -> &mut LinksHeader<T> {
        unsafe { transmute(&mut self.indexes.as_mut()[0]) }
    }

    fn get_data_part(&self, link: T) -> &DataPart<T> {
        unsafe { &self.data.as_ref()[link.as_()] }
    }

    fn get_mut_data_part(&mut self, link: T) -> &mut DataPart<T> {
        unsafe { &mut self.data.as_mut()[link.as_()] }
    }

    fn get_index_part(&self, link: T) -> &IndexPart<T> {
        unsafe { &self.indexes.as_ref()[link.as_()] }
    }

    fn get_mut_index_part(&mut self, link: T) -> &mut IndexPart<T> {
        unsafe { &mut self.indexes.as_mut()[link.as_()] }
    }

    fn get_link_value(&self, link: T) -> Link<T> {
        let data = self.get_data_part(link);
        Link::new(link, data.source, data.target)
    }

    /// Returns the stored element count of the list of `head`.
    pub fn count_usages(&self, head: T) -> T {
        self.get_size(head)
    }

    /// Counts the elements of the list of `head` by walking it.
    pub fn count_by_traversal(&self, head: T) -> T {
        let mut count = T::from_byte(0);
        self.each_usages(head, &mut |_| {
            count = count + T::from_byte(1);
            Flow::Continue
        });
        count
    }

    pub fn each_usages<H: FnMut(Link<T>) -> Flow + ?Sized>(
        &self,
        source: T,
        handler: &mut H,
    ) -> Flow {
        let mut current = self.get_first(source);
        let first = current;

        while current != T::from_byte(0) {
            if handler(self.get_link_value(current)).is_break() {
                return Flow::Break;
            }
            current = self.get_next(current);
            if current == first {
                return Flow::Continue;
            }
        }
        Flow::Continue
    }
}

impl<T: LinkReference> RelativeLinkedList<T> for InternalSourcesLinkedList<T> {
    fn get_first(&self, head: T) -> T {
        self.get_index_part(head).root_as_source
    }

    fn get_last(&self, head: T) -> T {
        let first = self.get_first(head);
        if first == T::from_byte(0) {
            first
        } else {
            self.get_previous(first)
        }
    }

    fn get_size(&self, head: T) -> T {
        self.get_index_part(head).size_as_source
    }

    fn set_first(&mut self, head: T, element: T) {
        self.get_mut_index_part(head).root_as_source = element;
    }

    fn set_last(&mut self, _head: T, _element: T) {
        // todo!("empty")
        // let first = self.get_index_part(head).root_as_source;
        // if first.is_zero() {
        //     self.set_first(head, element);
        // } else {
        //     self.set_previous(first, element);
        // }
    }

    fn set_size(&mut self, head: T, size: T) {
        self.get_mut_index_part(head).size_as_source = size;
    }

    fn inc_size(&mut self, head: T) {
        if self.track_size {
            self.set_size(head, self.get_size(head) + T::from_byte(1));
        }
    }

    fn dec_size(&mut self, head: T) {
        if self.track_size {
            self.set_size(head, self.get_size(head) - T::from_byte(1));
        }
    }
}

impl<T: LinkReference> LinkedList<T> for InternalSourcesLinkedList<T> {
    fn get_previous(&self, element: T) -> T {
        self.get_index_part(element).left_as_source
    }

    fn get_next(&self, element: T) -> T {
        self.get_index_part(element).right_as_source
    }

    fn set_previous(&mut self, element: T, previous: T) {
        self.get_mut_index_part(element).left_as_source = previous;
    }

    fn set_next(&mut self, element: T, next: T) {
        self.get_mut_index_part(element).right_as_source = next;
    }
}

impl<T: LinkReference> RelativeCircularLinkedList<T> for InternalSourcesLinkedList<T> {}

impl<T: LinkReference> SplitUpdateMem<T> for InternalSourcesLinkedList<T> {
    fn update_mem(&mut self, data: NonNull<[DataPart<T>]>, indexes: NonNull<[IndexPart<T>]>) {
        self.data = data;
        self.indexes = indexes;
    }
}
