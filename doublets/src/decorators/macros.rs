//! Internal macros that remove the boilerplate shared by every decorator.

/// Emits the struct, constructors and the [`Doublets`](crate::Doublets) forwarding
/// impl shared by every decorator that wraps a single store.
macro_rules! decorator_struct {
    (
        $(#[$attr:meta])*
        $name:ident
    ) => {
        $(#[$attr])*
        pub struct $name<T: LinkReference, L: Doublets<T>> {
            links: L,
            _marker: PhantomData<fn(T)>,
        }

        impl<T: LinkReference, L: Doublets<T>> $name<T, L> {
            #[doc = concat!("Wraps `links` in a [`", stringify!($name), "`].")]
            #[inline]
            pub const fn new(links: L) -> Self {
                Self {
                    links,
                    _marker: PhantomData,
                }
            }

            /// Returns a shared reference to the wrapped store.
            #[inline]
            #[must_use]
            pub const fn inner(&self) -> &L {
                &self.links
            }

            /// Returns a mutable reference to the wrapped store.
            #[inline]
            #[must_use]
            pub const fn inner_mut(&mut self) -> &mut L {
                &mut self.links
            }

            /// Unwraps the decorator and returns the wrapped store.
            #[inline]
            #[must_use]
            pub fn into_inner(self) -> L {
                self.links
            }
        }

        impl<T: LinkReference, L: Doublets<T> + Debug> Debug for $name<T, L> {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                f.debug_tuple(stringify!($name)).field(&self.links).finish()
            }
        }

        impl<T: LinkReference, L: Doublets<T>> Doublets<T> for $name<T, L> {
            #[inline]
            fn get_link(&self, index: T) -> Option<Link<T>> {
                self.links.get_link(index)
            }
        }
    };
}

/// Emits verbatim [`Links`](crate::Links) forwarding methods for every operation a
/// decorator does not intercept.
///
/// [`Links::constants`](crate::Links::constants) is always forwarded, because no
/// decorator changes the store's constants.
macro_rules! forward {
    ($($method:ident),* $(,)?) => {
        #[inline]
        fn constants(&self) -> &LinksConstants<T> {
            self.links.constants()
        }

        $(forward!(@method $method);)*
    };
    (@method count_links) => {
        #[inline]
        fn count_links(&self, query: &[T]) -> T {
            self.links.count_links(query)
        }
    };
    (@method each_links) => {
        #[inline]
        fn each_links(&self, query: &[T], handler: ReadHandler<'_, T>) -> Flow {
            self.links.each_links(query, handler)
        }
    };
    (@method create_links) => {
        #[inline]
        fn create_links(
            &mut self,
            query: &[T],
            handler: WriteHandler<'_, T>,
        ) -> Result<Flow, Error<T>> {
            self.links.create_links(query, handler)
        }
    };
    (@method update_links) => {
        #[inline]
        fn update_links(
            &mut self,
            query: &[T],
            change: &[T],
            handler: WriteHandler<'_, T>,
        ) -> Result<Flow, Error<T>> {
            self.links.update_links(query, change, handler)
        }
    };
    (@method delete_links) => {
        #[inline]
        fn delete_links(
            &mut self,
            query: &[T],
            handler: WriteHandler<'_, T>,
        ) -> Result<Flow, Error<T>> {
            self.links.delete_links(query, handler)
        }
    };
}

pub(crate) use {decorator_struct, forward};
