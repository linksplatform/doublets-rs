macro_rules! tri {
    ($expr:expr) => {
        match $expr {
            Some(x) => x,
            None => return false,
        }
    };
}

mod leaf;

pub use leaf::Leaf;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Node<T> {
    pub size: usize,
    pub left: Option<T>,
    pub right: Option<T>,
}

impl<T> Default for Node<T> {
    fn default() -> Self {
        Self { size: 0, left: None, right: None }
    }
}

macro_rules! fn_set {
    ($($name:ident => $set:ident: $ty:ty)*) => {$(
        fn $name(&mut self, idx: T, $set: $ty) {
            if let Some(node) = self.get(idx) {
                self.set(idx, Node { $set, ..node });
            }
        }
    )*};
}

pub trait Tree<T: Leaf> {
    fn ptr_range(&self) -> std::ops::Range<*const u8>;

    fn get(&self, idx: T) -> Option<Node<T>>;
    fn set(&mut self, idx: T, node: Node<T>);

    fn left_mut(&mut self, idx: T) -> Option<&mut T>;
    fn right_mut(&mut self, idx: T) -> Option<&mut T>;

    fn is_left_of(&self, first: T, second: T) -> bool;

    fn is_right_of(&self, first: T, second: T) -> bool {
        !first.same(second) && !self.is_left_of(first, second)
    }

    fn size(&self, idx: T) -> Option<usize> {
        try { self.get(idx)?.size }
    }

    fn left(&self, idx: T) -> Option<T> {
        self.get(idx)?.left
    }

    fn right(&self, idx: T) -> Option<T> {
        self.get(idx)?.right
    }

    fn_set! {
        set_size => size: usize
        set_left => left: Option<T>
        set_right => right: Option<T>
    }

    fn left_size(&self, idx: T) -> Option<usize> {
        self.left(idx).and_then(|idx| self.size(idx))
    }

    fn right_size(&self, idx: T) -> Option<usize> {
        self.right(idx).and_then(|idx| self.size(idx))
    }

    fn rightest(&self, mut current: T) -> T {
        while let Some(next) = self.right(current) {
            current = next;
        }
        current
    }

    fn leftest(&self, mut current: T) -> T {
        while let Some(next) = self.left(current) {
            current = next;
        }
        current
    }

    fn next(&self, idx: T) -> Option<T> {
        self.right(idx).map(|idx| self.leftest(idx))
    }

    fn prev(&self, idx: T) -> Option<T> {
        self.left(idx).map(|idx| self.rightest(idx))
    }

    fn is_contains(&self, mut root: T, idx: T) -> bool {
        loop {
            if self.is_left_of(idx, root) {
                root = tri! { self.left(root) };
            } else if self.is_right_of(idx, root) {
                root = tri! { self.right(root) };
            } else {
                break true;
            }
        }
    }

    fn inc_size(&mut self, idx: T) {
        if let Some(size) = self.size(idx) {
            self.set_size(idx, size + 1)
        }
    }

    fn dec_size(&mut self, idx: T) {
        if let Some(size) = self.size(idx) {
            self.set_size(idx, size - 1)
        }
    }

    fn fix_size(&mut self, idx: T) {
        self.set_size(
            idx,
            self.left_size(idx).unwrap_or_default() + self.right_size(idx).unwrap_or_default() + 1,
        )
    }

    fn clear(&mut self, idx: T) {
        self.set(idx, Node { size: 0, left: None, right: None })
    }

    #[must_use]
    fn rotate_left(&mut self, root: T) -> Option<T> {
        let right = self.right(root)?;
        self.set_right(root, self.left(right));
        self.set_left(right, Some(root));
        self.set_size(right, self.size(root)?);
        self.fix_size(root);
        Some(right)
    }

    #[must_use]
    fn rotate_right(&mut self, root: T) -> Option<T> {
        let left = self.left(root)?;
        self.set_left(root, self.right(left));
        self.set_right(left, Some(root));
        self.set_size(left, self.size(root)?);
        self.fix_size(root);
        Some(left)
    }
}

/// # Safety
/// This relies on [`Leaf`] guarantees, hence it must provide guarantees of tree addr space
pub unsafe trait NoRecur<T: Leaf>: Tree<T> {
    fn attach(&mut self, root: Option<T>, idx: T) -> Option<T> {
        if let Some(mut root) = root {
            unsafe { attach_impl(self, &mut root, idx)? }
            Some(root)
        } else {
            self.set_size(idx, 1);
            Some(idx)
        }
    }

    fn detach(&mut self, root: Option<T>, idx: T) -> Option<T> {
        let mut root = root?;

        if unsafe { detach_impl(self, &mut root, idx) }.expect(UNCHECKED_MESSAGE) {
            None
        } else {
            Some(root)
        }
    }
}

const UNCHECKED_MESSAGE: &str = "unchecked...";

unsafe fn attach_impl<T: Leaf, Tree>(tree: &mut Tree, mut root: *mut T, idx: T) -> Option<()>
where
    Tree: NoRecur<T> + ?Sized,
{
    loop {
        if tree.is_left_of(idx, *root) {
            let Some(left) = tree.left_mut(*root) else {
                tree.inc_size(*root);
                tree.set_size(idx, 1);
                tree.set_left(*root, Some(idx));
                return Some(());
            };
            let left = left as *mut T;

            let left_size = tree.size(*left)?;
            let right_size = tree.right_size(*root).unwrap_or_default();

            if tree.is_left_of(idx, *left) {
                if left_size >= right_size {
                    *root = tree.rotate_right(*root)?;
                } else {
                    tree.inc_size(*root);
                    root = left;
                }
            } else {
                let lr_size = tree.right_size(*left).unwrap_or_default();
                if lr_size >= right_size {
                    if lr_size == 0 && right_size == 0 {
                        tree.set_left(idx, Some(*left));
                        tree.set_right(idx, Some(*root));
                        tree.set_size(idx, left_size + 2);
                        tree.set_left(*root, None);
                        tree.set_size(*root, 1);
                        *root = idx;
                        return Some(());
                    }
                    *left = tree.rotate_left(*left)?;
                    *root = tree.rotate_right(*root)?;
                } else {
                    tree.inc_size(*root);
                    root = left;
                }
            }
        } else {
            let Some(right) = tree.right_mut(*root) else {
                tree.inc_size(*root);
                tree.set_size(idx, 1);
                tree.set_right(*root, Some(idx));
                return Some(());
            };
            let right = right as *mut T;

            let right_size = tree.size(*right)?;
            let left_size = tree.left_size(*root).unwrap_or_default();

            if tree.is_right_of(idx, *right) {
                if right_size >= left_size {
                    *root = tree.rotate_left(*root)?;
                } else {
                    tree.inc_size(*root);
                    root = right;
                }
            } else {
                let rl_size = tree.left_size(*right).unwrap_or_default();
                if rl_size >= left_size {
                    if rl_size == 0 && left_size == 0 {
                        tree.set_left(idx, Some(*root));
                        tree.set_right(idx, Some(*right));
                        tree.set_size(idx, right_size + 2);
                        tree.set_right(*root, None);
                        tree.set_size(*root, 1);
                        *root = idx;
                        return Some(());
                    }
                    *right = tree.rotate_right(*right)?;
                    *root = tree.rotate_left(*root)?;
                } else {
                    tree.inc_size(*root);
                    root = right;
                }
            }
        }
    }
}

unsafe fn detach_impl<T: Leaf, Tree>(tree: &mut Tree, mut root: *mut T, idx: T) -> Option<bool>
where
    Tree: NoRecur<T> + ?Sized,
{
    #[rustfmt::skip]
    fn as_ptr<T>(t: &mut T) -> *mut T { t }

    loop {
        let left = tree.left_mut(*root).map(as_ptr);
        let right = tree.right_mut(*root).map(as_ptr);

        if tree.is_left_of(idx, *root) {
            let rl_size =
                tree.right(*root).and_then(|right| tree.left_size(right)).unwrap_or_default();
            let rr_size =
                tree.right(*root).and_then(|right| tree.right_size(right)).unwrap_or_default();
            let left_size = tree.left_size(*root).unwrap_or_default();

            if rr_size >= left_size {
                *root = tree.rotate_left(*root)?;
            } else if rl_size >= left_size {
                let right = right.expect(UNCHECKED_MESSAGE);

                *right = tree.rotate_right(*right)?;
                *root = tree.rotate_left(*root)?;
            } else {
                tree.dec_size(*root);
                root = left.expect(UNCHECKED_MESSAGE);
            }
        } else if tree.is_right_of(idx, *root) {
            let ll_size =
                tree.left(*root).and_then(|right| tree.left_size(right)).unwrap_or_default();
            let lr_size =
                tree.left(*root).and_then(|right| tree.right_size(right)).unwrap_or_default();
            let right_size = tree.right_size(*root).unwrap_or_default();

            if ll_size >= right_size {
                *root = tree.rotate_left(*root)?;
            } else if lr_size >= right_size {
                let left = left.expect(UNCHECKED_MESSAGE);

                *left = tree.rotate_left(*left)?;
                *root = tree.rotate_right(*root)?;
            } else {
                tree.dec_size(*root);
                root = right.expect(UNCHECKED_MESSAGE);
            }
        } else {
            match (left, right) {
                (Some(left), Some(right)) => {
                    let (left_size, right_size) = (tree.left_size(*root)?, tree.right_size(*root)?);

                    let new;
                    if left_size > right_size {
                        new = tree.rightest(*left);
                        let _ = detach_impl(tree, left, new);
                    } else {
                        new = tree.leftest(*right);
                        let _ = detach_impl(tree, right, new);
                    }
                    tree.set_left(new, tree.left(*root));
                    tree.set_right(new, tree.right(*root));
                    tree.set_size(new, left_size + right_size);
                    *root = new;
                }
                (Some(left), _) => *root = *left,
                (_, Some(right)) => *root = *right,
                _ => {
                    tree.clear(idx);
                    return Some(Leaf::remove_idx(root, tree.ptr_range()));
                }
            };
            tree.clear(idx);
            return Some(false);
        }
    }
}
