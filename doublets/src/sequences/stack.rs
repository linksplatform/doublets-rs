/// A growable LIFO stack, corresponding to C# `DefaultStack<TElement>`.
#[derive(Clone, Debug)]
pub struct DefaultStack<T> {
    items: Vec<T>,
}

impl<T> Default for DefaultStack<T> {
    fn default() -> Self {
        Self { items: Vec::new() }
    }
}

impl<T> DefaultStack<T> {
    /// Creates an empty stack.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an element to the top.
    pub fn push(&mut self, item: T) {
        self.items.push(item);
    }

    /// Removes the top element, or returns `None` when empty.
    pub fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }

    /// Borrows the top element without removing it.
    #[must_use]
    pub fn peek(&self) -> Option<&T> {
        self.items.last()
    }

    /// Removes all elements, retaining the allocated capacity.
    pub fn clear(&mut self) {
        self.items.clear();
    }

    /// Returns the number of elements.
    #[must_use]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns whether the stack is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}
