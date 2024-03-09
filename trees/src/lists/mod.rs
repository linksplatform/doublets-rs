mod absolute_circular_linked_list;
mod absolute_linked_list;
mod linked_list;
mod relative_circular_linked_list;
mod relative_doubly_linked_list;

// TODO: use human names
pub use {
    absolute_circular_linked_list::AbsoluteCircularLinkedList,
    absolute_linked_list::AbsoluteLinkedList, linked_list::LinkedList,
    relative_circular_linked_list::RelativeCircularLinkedList,
    relative_doubly_linked_list::RelativeLinkedList,
};
