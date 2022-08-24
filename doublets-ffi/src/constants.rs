use doublets::data::{LinkType, LinksConstants};
use std::{mem::MaybeUninit, ops::RangeInclusive};

/// FFI repr to [`Inclusive Range`]
///
/// [`Inclusive Range`]: https://doc.rust-lang.org/std/ops/struct.RangeInclusive.html
#[derive(Eq, PartialEq)]
#[repr(C)]
pub struct Range<T: LinkType>(pub T, pub T);

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
    // `MaybeUninit` is transparent - `Range<T>` is repr(C)
    pub external_range: MaybeUninit<Range<T>>,
    pub external_is_some: bool,
}

impl<T: LinkType> From<LinksConstants<T>> for Constants<T> {
    fn from(c: LinksConstants<T>) -> Self {
        let mut new = Self {
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
            internal_range: Range(*c.internal_range.start(), *c.internal_range.end()),
            // external_range: c.external_range.map(|r| Range(*r.start(), *r.end())),
            external_range: MaybeUninit::uninit(),
            external_is_some: false,
        };
        if let Some(r) = c.external_range {
            new.external_is_some = true;
            new.external_range.write(Range(*r.start(), *r.end()));
        }
        new
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
            internal_range: RangeInclusive::new(self.internal_range.0, self.internal_range.1),
            external_range: if self.external_is_some {
                // SAFETY: `self.external_range` is init
                unsafe {
                    let range = self.external_range.assume_init();
                    Some(RangeInclusive::new(range.0, range.1))
                }
            } else {
                None
            },
        }
    }
}
