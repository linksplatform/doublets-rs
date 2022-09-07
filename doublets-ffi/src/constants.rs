use crate::utils::Maybe;
use doublets::data::{LinkType, LinksConstants};
use std::ops::RangeInclusive;

/// FFI repr to [`Inclusive Range`]
///
/// [`Inclusive Range`]: https://doc.rust-lang.org/std/ops/struct.RangeInclusive.html
#[derive(Eq, PartialEq)]
#[repr(C)]
pub struct Range<T> {
    start: T,
    end: T,
}

impl<T: Copy> From<RangeInclusive<T>> for Range<T> {
    fn from(range: RangeInclusive<T>) -> Self {
        Self {
            start: *range.start(),
            end: *range.end(),
        }
    }
}

impl<T: Copy> From<Range<T>> for RangeInclusive<T> {
    fn from(Range { start, end }: Range<T>) -> Self {
        RangeInclusive::new(start, end)
    }
}

#[repr(C)]
pub struct Constants<T: LinkType> {
    pub index_part: T,
    pub source_part: T,
    pub target_part: T,
    pub null: T,
    pub r#continue: T,
    pub r#break: T,
    pub skip: T,
    pub any: T,
    pub itself: T,
    pub error: T,
    pub internal_range: Range<T>,
    pub external_range: Maybe<Range<T>>,
}

impl<T: LinkType> From<LinksConstants<T>> for Constants<T> {
    fn from(c: LinksConstants<T>) -> Self {
        Self {
            index_part: c.index_part,
            source_part: c.source_part,
            target_part: c.target_part,
            null: c.null,
            r#continue: c.r#continue,
            r#break: c.r#break,
            skip: c.skip,
            any: c.any,
            itself: c.itself,
            error: c.error,
            internal_range: Range::from(c.internal_range),
            // external_range: c.external_range.map(|r| Range(*r.start(), *r.end())),
            external_range: c.external_range.map(Range::from).into(),
        }
    }
}

#[allow(clippy::from_over_into)]
impl<T: LinkType> Into<LinksConstants<T>> for Constants<T> {
    fn into(self) -> LinksConstants<T> {
        LinksConstants {
            index_part: self.index_part,
            source_part: self.source_part,
            target_part: self.target_part,
            r#break: self.r#break,
            null: self.null,
            r#continue: self.r#continue,
            skip: self.skip,
            any: self.any,
            itself: self.itself,
            error: self.error,
            internal_range: RangeInclusive::from(self.internal_range),
            external_range: Option::from(self.external_range).map(|range: Range<_>| range.into()),
        }
    }
}
