use data::LinkReference;

use crate::Doublets;

/// A predicate that identifies links, for example terminal sequence elements.
///
/// Closures implementing `Fn(T) -> bool` can also serve as matchers.
pub trait CriterionMatcher<T: LinkReference> {
    /// Returns whether `link` satisfies this criterion in `links`.
    fn is_matched<L: Doublets<T> + ?Sized>(&self, links: &L, link: T) -> bool;
}

impl<T: LinkReference, F: Fn(T) -> bool> CriterionMatcher<T> for F {
    fn is_matched<L: Doublets<T> + ?Sized>(&self, _links: &L, link: T) -> bool {
        self(link)
    }
}

/// Identifies stored links whose target equals a marker address.
///
/// Counterpart of C# `Platform.Data.Doublets.CriterionMatchers.TargetMatcher`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TargetMatcher<T> {
    target: T,
}

impl<T: LinkReference> TargetMatcher<T> {
    /// Creates a matcher for links pointing to `target`.
    #[must_use]
    pub const fn new(target: T) -> Self {
        Self { target }
    }

    /// Returns the marker address.
    #[must_use]
    pub const fn target(&self) -> T {
        self.target
    }

    /// Returns false for missing links, otherwise compares the stored target.
    pub fn is_matched<L: Doublets<T> + ?Sized>(&self, links: &L, link: T) -> bool {
        links
            .get_link(link)
            .is_some_and(|link| link.target == self.target)
    }
}

impl<T: LinkReference> CriterionMatcher<T> for TargetMatcher<T> {
    fn is_matched<L: Doublets<T> + ?Sized>(&self, links: &L, link: T) -> bool {
        self.is_matched(links, link)
    }
}
