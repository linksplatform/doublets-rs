//! Tests for the `before` / `after` links reported to write handlers.

use data::Flow;
use doublets::{split, unit, Doublets, Error, Link};
use mem::Global;

/// `update_links` must report the *previous* state of the link as `before`.
///
/// The unit store used to alias `old_source`/`old_target` to the incoming
/// change, so every handler saw `before == after`.
#[test]
fn unit_update_reports_the_previous_link() -> Result<(), Error<usize>> {
    let mut store = unit::Store::<usize, _>::new(Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;

    let mut seen = None;
    store.update_by_with([a], [a, a, b], |before, after| {
        seen = Some((before, after));
        Flow::Continue
    })?;

    let (before, after) = seen.expect("the handler must be called");
    assert_eq!(before, Link::new(a, a, a));
    assert_eq!(after, Link::new(a, a, b));

    Ok(())
}

#[test]
fn split_update_reports_the_previous_link() -> Result<(), Error<usize>> {
    let mut store = split::Store::<usize, _, _>::new(Global::new(), Global::new())?;

    let a = store.create_point()?;
    let b = store.create_point()?;

    let mut seen = None;
    store.update_by_with([a], [a, a, b], |before, after| {
        seen = Some((before, after));
        Flow::Continue
    })?;

    let (before, after) = seen.expect("the handler must be called");
    assert_eq!(before, Link::new(a, a, a));
    assert_eq!(after, Link::new(a, a, b));

    Ok(())
}
