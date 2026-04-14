use crate::Link;
use data::{Flow, LinkType};
use std::marker::PhantomData;

pub struct Fuse<T: LinkType, H: FnMut(Link<T>, Link<T>) -> Flow> {
    handler: H,
    done: bool,
    _marker: PhantomData<fn(T)>,
}

impl<T: LinkType, F: FnMut(Link<T>, Link<T>) -> Flow> Fuse<T, F> {
    pub fn new(handler: F) -> Self {
        Self {
            handler,
            done: false,
            _marker: PhantomData,
        }
    }

    pub fn call(&mut self, before: Link<T>, after: Link<T>) -> Flow {
        if self.done {
            Flow::Break
        } else {
            let result = (self.handler)(before, after);
            if result.is_break() {
                self.done = true;
                Flow::Break
            } else {
                Flow::Continue
            }
        }
    }
}
