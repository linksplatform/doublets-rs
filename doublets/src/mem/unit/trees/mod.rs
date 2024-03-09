use {
    super::LinkRepr,
    crate::Link,
    core::{LinkType, Repr},
    std::{mem, ops::Range},
    trees::{trees::Node, Leaf, NoRecur, Tree as _},
};

pub trait Tree: NoRecur<<Self::Item as LinkType>::Repr> {
    type Item: LinkType;

    fn with(slice: &[LinkRepr<Self::Item>]) -> &Self;
    fn with_mut(slice: &mut [LinkRepr<Self::Item>]) -> &mut Self;

    fn each_usages<F>(&self, root: Option<<Self::Item as LinkType>::Repr>, link: Self::Item, f: F)
    where
        F: FnMut(Link<Self::Item>);

    fn count_usage(&self, root: <Self::Item as LinkType>::Repr, link: Self::Item) -> usize;

    fn find(
        &self,
        root: Option<<Self::Item as LinkType>::Repr>,
        source: Self::Item,
        target: Self::Item,
    ) -> Option<<Self::Item as LinkType>::Repr>;
}

macro_rules! impl_tree {
    ($name:ident: $part:ident -> $size:ident $left:ident $right: ident) => {
        #[repr(transparent)]
        pub struct $name<T: LinkType>([LinkRepr<T>]);

        impl<T: Repr, U: LinkType<Repr = T>> trees::Tree<T> for $name<U> {
            fn ptr_range(&self) -> Range<*const u8> {
                unsafe { mem::transmute(self.0.as_ptr_range()) }
            }

            fn get(&self, idx: T) -> Option<Node<T>> {
                self.0.get(idx.addr_of()).map(|&LinkRepr { $size, $left, $right, .. }| Node {
                    size: $size.addr(),
                    left: $left,
                    right: $right,
                })
            }

            fn set(&mut self, idx: T, Node { size, left, right }: Node<T>) {
                if let Some(repr) = self.0.get_mut(idx.addr_of()) {
                    repr.$size = U::from_addr(size);
                    repr.$left = left;
                    repr.$right = right;
                }
            }

            fn left_mut(&mut self, idx: T) -> Option<&mut T> {
                self.0.get_mut(idx.addr_of())?.$left.as_mut()
            }

            fn right_mut(&mut self, idx: T) -> Option<&mut T> {
                self.0.get_mut(idx.addr_of())?.$right.as_mut()
            }

            fn is_left_of(&self, first: T, second: T) -> bool {
                let (first, second) = (self.into_repr(first), self.into_repr(second));
                first < second
            }
        }

        unsafe impl<T: Repr, U: LinkType<Repr = T>> NoRecur<T> for $name<U> {}

        impl<T: LinkType> Tree for $name<T> {
            type Item = T;

            fn with(slice: &[LinkRepr<Self::Item>]) -> &$name<T> {
                unsafe { mem::transmute(slice) }
            }

            fn with_mut(slice: &mut [LinkRepr<Self::Item>]) -> &mut $name<T> {
                unsafe { mem::transmute(slice) }
            }

            fn each_usages<F>(&self, root: Option<T::Repr>, link: T, mut f: F)
            where
                F: FnMut(Link<T>),
            {
                if let Some(root) = root {
                    self.each_impl(Repr::repr(Some(root)), Repr::from_addr(link), &mut f)
                }
            }

            fn count_usage(&self, root: T::Repr, link: T) -> usize {
                let total = self.size(root).unwrap_or_default();

                let left = {
                    let mut total = 0;
                    let mut root = Some(root);

                    while let Some(idx) = root
                        && let Some(repr) = self.get_link(idx)
                        && let mark = repr.$part.addr()
                    {
                        if mark >= link.addr() {
                            root = self.left(idx);
                        } else {
                            total += 1 + self.left_size(idx).unwrap_or_default();
                        }
                    }
                    total
                };

                let right = {
                    let mut total = 0;
                    let mut root = Some(root);

                    while let Some(idx) = root
                        && let Some(repr) = self.get_link(idx)
                        && let mark = repr.$part.addr()
                    {
                        if mark <= link.addr() {
                            root = self.right(idx);
                        } else {
                            total += 1 + self.right_size(idx).unwrap_or_default();
                        }
                    }
                    total
                };

                total - left - right
            }

            fn find(&self, mut root: Option<T::Repr>, source: T, target: T) -> Option<T::Repr> {
                while let Some(idx) = root
                    && let repr = (source.addr(), target.addr())
                {
                    if repr < self.into_repr(idx) {
                        root = self.left(idx);
                    } else if repr > self.into_repr(idx) {
                        root = self.right(idx);
                    } else {
                        return root;
                    }
                }
                None
            }
        }

        impl<T: LinkType> $name<T> {
            fn get_repr(&self, idx: T::Repr) -> Option<LinkRepr<T>> {
                self.0.get(idx.addr_of()).copied()
            }

            fn get_link(&self, idx: T::Repr) -> Option<Link<T>> {
                let LinkRepr { source, target, .. } = self.get_repr(idx)?;
                Some(Link::new(T::from_addr(idx.addr_of()), source, target))
            }

            fn into_repr(&self, idx: T::Repr) -> (usize, usize) {
                self.0
                    .get(idx.addr_of())
                    .map(|&LinkRepr { source, target, .. }| (source.addr_of(), target.addr_of()))
                    .unwrap_or_default()
            }

            fn each_impl(&self, root: usize, link: Option<T::Repr>, f: &mut impl FnMut(Link<T>)) {
                if let Some(idx) = link
                    && let Some(link) = self.get_link(idx)
                    && let mark = link.$part.addr()
                {
                    println!("{}", link.index.addr());
                    if mark > root {
                        self.each_impl(root, self.left(idx), f);
                    } else if mark < root {
                        self.each_impl(root, self.right(idx), f);
                    } else {
                        f(link);
                        self.each_impl(root, self.left(idx), f);
                        self.each_impl(root, self.right(idx), f);
                    }
                }
            }
        }
    };
}

impl_tree!(Sources: source -> source_size source_left source_right);
impl_tree!(Targets: target -> target_size target_left target_right);
