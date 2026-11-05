//! Iterative sequence traversal with configurable terminal-element criteria.

use std::{collections::HashSet, iter::FusedIterator};

use data::LinkReference;

use super::{CriterionMatcher, DefaultStack, SequenceError};
use crate::Doublets;

/// Walks source before target, yielding sequence elements in their original order.
///
/// The matcher identifies terminal elements; it can be a [`super::TargetMatcher`]
/// or an `Fn(T) -> bool` closure. Traversal uses a heap stack instead of recursion.
#[derive(Clone, Copy, Debug)]
pub struct RightSequenceWalker<M> {
    matcher: M,
}

impl<M> RightSequenceWalker<M> {
    /// Creates a walker with the terminal-element matcher.
    #[must_use]
    pub const fn new(matcher: M) -> Self {
        Self { matcher }
    }

    /// Lazily walks a sequence. A null root yields no elements.
    ///
    /// Missing nonterminal links and cycles yield one error, then terminate the
    /// iterator. Shared subtrees are traversed for each occurrence.
    pub fn iter<'a, T, L>(&'a self, links: &'a L, sequence: T) -> SequenceIter<'a, T, L, M>
    where
        T: LinkReference,
        L: Doublets<T> + ?Sized,
        M: CriterionMatcher<T>,
    {
        let mut stack = DefaultStack::new();
        if sequence != links.constants().null {
            stack.push((sequence, false));
        }
        SequenceIter {
            links,
            matcher: &self.matcher,
            stack,
            active: HashSet::new(),
        }
    }

    /// Collects all elements, propagating the first traversal error.
    pub fn walk<T, L>(&self, links: &L, sequence: T) -> Result<Vec<T>, SequenceError<T>>
    where
        T: LinkReference,
        L: Doublets<T> + ?Sized,
        M: CriterionMatcher<T>,
    {
        self.iter(links, sequence).collect()
    }
}

/// A lazy, stack-based walk returned by [`RightSequenceWalker::iter`].
pub struct SequenceIter<'a, T: LinkReference, L: ?Sized, M> {
    links: &'a L,
    matcher: &'a M,
    // An exit frame removes a node from the active path after both children.
    stack: DefaultStack<(T, bool)>,
    active: HashSet<T>,
}

impl<T, L, M> SequenceIter<'_, T, L, M>
where
    T: LinkReference,
    L: Doublets<T> + ?Sized,
    M: CriterionMatcher<T>,
{
    fn advance(&mut self) -> Result<Option<T>, SequenceError<T>> {
        while let Some((element, exiting)) = self.stack.pop() {
            if exiting {
                self.active.remove(&element);
                continue;
            }
            if self.matcher.is_matched(self.links, element) {
                return Ok(Some(element));
            }
            if !self.active.insert(element) {
                return Err(SequenceError::CyclicSequence(element));
            }
            let link = self.links.try_get_link(element)?;
            self.stack.push((element, true));
            self.stack.push((link.target, false));
            self.stack.push((link.source, false));
        }
        Ok(None)
    }
}

impl<T, L, M> Iterator for SequenceIter<'_, T, L, M>
where
    T: LinkReference,
    L: Doublets<T> + ?Sized,
    M: CriterionMatcher<T>,
{
    type Item = Result<T, SequenceError<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.advance() {
            Ok(element) => element.map(Ok),
            Err(error) => {
                self.stack.clear();
                self.active.clear();
                Some(Err(error))
            }
        }
    }
}

impl<T, L, M> FusedIterator for SequenceIter<'_, T, L, M>
where
    T: LinkReference,
    L: Doublets<T> + ?Sized,
    M: CriterionMatcher<T>,
{
}
