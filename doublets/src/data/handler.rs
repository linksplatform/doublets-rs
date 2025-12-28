use crate::Link;
#[cfg(feature = "data")]
use data::{Flow, LinkType};
use std::{marker::PhantomData, mem::MaybeUninit};
#[cfg(feature = "nightly")]
use std::ops::Try;

pub trait Handler<T, R>: FnMut(Link<T>, Link<T>) -> R
where
    T: LinkType,
    R: Try<Output = ()>,
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
    R: Try<Output = ()>,
    All: FnMut(Link<T>, Link<T>) -> R,
{
}

pub struct Fuse<T, H, R>
where
    T: LinkType,
    H: Handler<T, R>,
    R: Try<Output = ()>,
{
    handler: H,
    done: bool,
    _marker: PhantomData<fn(T) -> R>,
}

impl<T, F, R> Fuse<T, F, R>
where
    T: LinkType,
    F: FnMut(Link<T>, Link<T>) -> R,
    R: Try<Output = ()>,
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
    R: Try<Output = ()>,
{
    fn from(handler: H) -> Self {
        Self::new(handler)
    }
}

impl<T, H, R> FnOnce<(Link<T>, Link<T>)> for Fuse<T, H, R>
where
    H: FnMut(Link<T>, Link<T>) -> R,
    R: Try<Output = ()>,
    T: LinkType,
{
    type Output = Flow;

    extern "rust-call" fn call_once(self, args: (Link<T>, Link<T>)) -> Self::Output {
        self.handler.call_once(args).branch().into()
    }
}

impl<T, H, R> FnMut<(Link<T>, Link<T>)> for Fuse<T, H, R>
where
    T: LinkType,
    H: Handler<T, R>,
    R: Try<Output = ()>,
{
    extern "rust-call" fn call_mut(&mut self, args: (Link<T>, Link<T>)) -> Self::Output {
        if self.done {
            Flow::Break
        } else {
            let result = self.handler.call_mut(args);
            if result.branch().is_break() {
                self.done = false;
                Flow::Break
            } else {
                Flow::Continue
            }
        }
    }
}
