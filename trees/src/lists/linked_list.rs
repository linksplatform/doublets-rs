use crate::LinkType;

pub trait LinkedList<T: LinkType> {
    fn get_previous(&self, element: T) -> T;
    fn get_next(&self, element: T) -> T;

    fn set_previous(&mut self, element: T, previous: T);
    fn set_next(&mut self, element: T, next: T);
}
