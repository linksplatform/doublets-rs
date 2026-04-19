use crate::Link;
use data::{Flow, LinkReference};
use std::marker::PhantomData;

/// A write-handler wrapper that stops calling the inner handler once it returns [`Flow::Break`].
///
/// After the first `Break`, every subsequent call to [`Fuse::call`] also returns `Break`
/// without invoking the wrapped closure.
pub struct Fuse<T: LinkReference, H: FnMut(Link<T>, Link<T>) -> Flow> {
    handler: H,
    done: bool,
    _marker: PhantomData<fn(T)>,
}

impl<T: LinkReference, F: FnMut(Link<T>, Link<T>) -> Flow> Fuse<T, F> {
    /// Wraps `handler` in a [`Fuse`].
    pub fn new(handler: F) -> Self {
        Self {
            handler,
            done: false,
            _marker: PhantomData,
        }
    }

    /// Calls the inner handler unless it has already returned [`Flow::Break`].
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
