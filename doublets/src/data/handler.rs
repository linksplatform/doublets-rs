use crate::Link;
use data::{Flow, LinkType};
use std::{marker::PhantomData, mem::MaybeUninit, ops::Try};

/// Trait for handler return types that can be converted to `Try<Output = ()>`.
/// This allows handlers to return `()` (implying `Flow::Continue`) or explicit `Flow` values.
pub trait HandlerResult {
    type Try: Try<Output = ()>;

    fn try_it(self) -> Self::Try;
}

impl HandlerResult for () {
    type Try = Flow;

    fn try_it(self) -> Self::Try {
        Flow::Continue
    }
}

impl<T: Try<Output = ()>> HandlerResult for T {
    type Try = T;

    fn try_it(self) -> Self::Try {
        self
    }
}

pub trait Handler<T, R>: FnMut(Link<T>, Link<T>) -> R
where
    T: LinkType,
    R: HandlerResult,
{
    fn fuse(self) -> Fuse<T, Self, R>
    where
        Self: Sized,
    {
        Fuse::new(self)
    }
}

impl<T, R, All> Handler<T, R> for All
where
    T: LinkType,
    R: HandlerResult,
    All: FnMut(Link<T>, Link<T>) -> R,
{
}

pub struct Fuse<T, H, R>
where
    T: LinkType,
    H: Handler<T, R>,
    R: HandlerResult,
{
    handler: H,
    done: bool,
    _marker: PhantomData<fn(T) -> R>,
}

impl<T, F, R> Fuse<T, F, R>
where
    T: LinkType,
    F: FnMut(Link<T>, Link<T>) -> R,
    R: HandlerResult,
{
    pub fn new(handler: F) -> Self {
        Self {
            handler,
            done: false,
            _marker: PhantomData,
        }
    }
}

impl<T, H, R> From<H> for Fuse<T, H, R>
where
    T: LinkType,
    H: Handler<T, R>,
    R: HandlerResult,
{
    fn from(handler: H) -> Self {
        Self::new(handler)
    }
}

impl<T, H, R> FnOnce<(Link<T>, Link<T>)> for Fuse<T, H, R>
where
    H: FnMut(Link<T>, Link<T>) -> R,
    R: HandlerResult,
    T: LinkType,
{
    type Output = Flow;

    extern "rust-call" fn call_once(self, args: (Link<T>, Link<T>)) -> Self::Output {
        self.handler.call_once(args).try_it().branch().into()
    }
}

impl<T, H, R> FnMut<(Link<T>, Link<T>)> for Fuse<T, H, R>
where
    T: LinkType,
    H: Handler<T, R>,
    R: HandlerResult,
{
    extern "rust-call" fn call_mut(&mut self, args: (Link<T>, Link<T>)) -> Self::Output {
        if self.done {
            Flow::Break
        } else {
            let result = self.handler.call_mut(args).try_it();
            if result.branch().is_break() {
                self.done = false;
                Flow::Break
            } else {
                Flow::Continue
            }
        }
    }
}
