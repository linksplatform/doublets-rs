use crate::{LinkType, LinkedList};

pub trait AbsoluteLinkedList<T: LinkType + funty::Unsigned>: LinkedList<T> {
    fn get_first(&self) -> T;
    fn get_last(&self) -> T;
    fn get_size(&self) -> T;

    fn set_first(&mut self, element: T);
    fn set_last(&mut self, element: T);
    fn set_size(&mut self, size: T);

    fn inc_size(&mut self) {
        self.set_size(self.get_size() + T::one())
    }
    fn dec_size(&mut self) {
        self.set_size(self.get_size() - T::one())
    }
}
