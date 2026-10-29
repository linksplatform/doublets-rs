//! Regressions for callback control through logging (issue #86).

use std::io;

use data::{Flow, LinksConstants};
use doublets::{
    data::{ReadHandler, WriteHandler},
    decorators::{DecoratorsExt, LoggingDecorator, Validate},
    split, unit, Doublets, Error, Link, Links,
};
use mem::Global;

fn creation_flow(mut store: impl Doublets<usize>) -> Result<(), Error<usize>> {
    let source = store.create_point()?;
    let target = store.create_point()?;

    // Break during creation, during update, or never. Initialization must finish.
    for break_at in [1, 2, usize::MAX] {
        let mut changes = Vec::new();
        let flow = store.create_link_with(source, target, |before, after| {
            changes.push((before, after));
            if changes.len() == break_at {
                Flow::Break
            } else {
                Flow::Continue
            }
        })?;

        assert_eq!(changes.len(), break_at.min(2));
        let index = changes[0].1.index;
        assert_eq!(changes[0], (Link::nothing(), Link::new(index, 0, 0)));
        assert_eq!(
            store.get_link(index),
            Some(Link::new(index, source, target))
        );
        if changes.len() == 2 {
            assert_eq!(
                changes[1],
                (Link::new(index, 0, 0), Link::new(index, source, target))
            );
        }
        assert_eq!(
            flow,
            if break_at <= 2 {
                Flow::Break
            } else {
                Flow::Continue
            }
        );
        store.delete(index)?;
    }
    Ok(())
}

fn logged_creation_flow(store: impl Doublets<usize>) -> Result<(), Error<usize>> {
    // The public constructor and null endpoints from the exact issue reproduction.
    let mut store = LoggingDecorator::new(store, Vec::<u8>::new());
    let mut changes = Vec::new();
    let flow = store.create_link_with(0, 0, |before, after| {
        changes.push((before, after));
        if changes.len() == 2 {
            Flow::Break
        } else {
            Flow::Continue
        }
    })?;
    assert_eq!(changes.len(), 2);
    let index = changes[1].1.index;
    assert_eq!(store.get_link(index), Some(Link::new(index, 0, 0)));
    assert_eq!(flow, Flow::Break);
    let (_, log) = store.into_inner();
    assert_eq!(
        String::from_utf8(log).unwrap(),
        format!(
            "Create. Before: (0->0). After: ({index}: 0->0)\n\
             Update. Before: ({index}: 0->0). After: ({index}: 0->0)\n"
        )
    );
    Ok(())
}

fn logged_mutations_preserve_break(store: impl Doublets<usize>) -> Result<(), Error<usize>> {
    let mut store = store.with_logging(Vec::new());
    let mut calls = 0;
    let flow = store.create_with(|_, _| {
        calls += 1;
        Flow::Break
    })?;
    assert_eq!(calls, 1);
    assert_eq!(flow, Flow::Break);
    let point = store.find([]).unwrap().index;
    assert_eq!(
        store.update_with(point, point, point, |_, _| Flow::Break)?,
        Flow::Break
    );
    assert_eq!(store.get_link(point), Some(Link::point(point)));
    assert_eq!(store.delete_with(point, |_, _| Flow::Break)?, Flow::Break);
    assert_eq!(store.get_link(point), None);
    let (_, log) = store.into_inner();
    assert_eq!(String::from_utf8(log).unwrap().lines().count(), 3);
    Ok(())
}

fn logged_cascade_finishes_after_break(store: impl Doublets<usize>) -> Result<(), Error<usize>> {
    let mut store = store
        .with_automatic_uniqueness_and_usages_resolution()
        .with_logging(Vec::new());
    let point = store.create_point()?;
    let usage = store.create_link(point, point + 1)?;
    store.writer_mut().clear();

    let mut calls = 0;
    let flow = store.delete_with(point, |_, _| {
        calls += 1;
        Flow::Break
    })?;
    assert_eq!(calls, 1);
    assert_eq!(store.count(), 0);
    assert_eq!(flow, Flow::Break);
    let (_, log) = store.into_inner();
    assert_eq!(
        String::from_utf8(log).unwrap(),
        format!(
            "Delete. Before: ({point}: {point}->{point}). After: ({point}: 0->0)\n\
             Delete. Before: ({usage}: {point}->{usage}). After: ({usage}: 0->0)\n\
             Delete. Before: ({usage}: 0->0). After: (0->0)\n\
             Delete. Before: ({point}: 0->0). After: (0->0)\n"
        )
    );
    Ok(())
}

fn primitive_break_and_errors_survive(store: impl Doublets<usize>) -> Result<(), Error<usize>> {
    let mut store = store.with_uniqueness(Validate).with_logging(Vec::new());
    let point = store.create_point()?;
    let result = store.create_link_with(point, point, |_, _| Flow::Break);
    assert!(matches!(result, Err(Error::AlreadyExists(_))));

    let mut store = store.with_no_exceptions().with_logging(Vec::new());
    // Only the creation calls the user handler. The rejected update returns Break.
    let mut calls = 0;
    let flow = store.create_link_with(point, point, |_, _| {
        calls += 1;
        Flow::Continue
    })?;
    assert_eq!(calls, 1);
    assert_eq!(flow, Flow::Break);
    assert_eq!(store.get_link(point), Some(Link::point(point)));
    Ok(())
}

fn automatic_creation_flow(store: impl Doublets<usize>) -> Result<(), Error<usize>> {
    creation_flow(store.with_automatic_uniqueness_and_usages_resolution())
}

fn logged_creation_phases(store: impl Doublets<usize>) -> Result<(), Error<usize>> {
    creation_flow(store.with_logging(Vec::new()))
}

fn logging_outside_automatic(store: impl Doublets<usize>) -> Result<(), Error<usize>> {
    creation_flow(
        store
            .with_automatic_uniqueness_and_usages_resolution()
            .with_logging(Vec::new()),
    )
}

fn logging_inside_automatic(store: impl Doublets<usize>) -> Result<(), Error<usize>> {
    creation_flow(
        store
            .with_logging(Vec::new())
            .with_automatic_uniqueness_and_usages_resolution(),
    )
}

fn logged_uniqueness_preserves_survivor(store: impl Doublets<usize>) -> Result<(), Error<usize>> {
    let mut store = store
        .with_automatic_uniqueness_and_usages_resolution()
        .with_logging(Vec::new());
    let point = store.create_point()?;
    for break_at in [1, 2, 3, usize::MAX] {
        store.writer_mut().clear();
        let mut calls = 0;
        let flow = store.create_link_with(point, point, |_, _| {
            calls += 1;
            if calls == break_at {
                Flow::Break
            } else {
                Flow::Continue
            }
        })?;
        assert_eq!(calls, break_at.min(3));
        assert_eq!(store.count(), 1);
        assert_eq!(store.get_link(point), Some(Link::point(point)));
        assert_eq!(
            flow,
            if break_at <= 3 {
                Flow::Break
            } else {
                Flow::Continue
            }
        );
        assert_eq!(
            String::from_utf8(store.writer_mut().clone()).unwrap(),
            "Create. Before: (0->0). After: (2: 0->0)\n\
             Update. Before: (2: 0->0). After: (0->0)\n\
             Update. Before: (1: 1->1). After: (1: 1->1)\n"
        );
    }
    assert_eq!(store.create_link(point, point)?, point);
    Ok(())
}

#[derive(Debug)]
struct BrokenWriter;

impl io::Write for BrokenWriter {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("log write failed"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn logging_error_takes_precedence(store: impl Doublets<usize>) {
    let mut store = store.with_logging(BrokenWriter);
    let mut created = 0;
    let result = store.create_with(|_, after| {
        created = after.index;
        Flow::Break
    });
    assert!(matches!(result, Err(Error::AllocFailed(_))));
    assert_eq!(store.get_link(created), Some(Link::new(created, 0, 0)));
}

// Model an inner layer that completes an update but discards callback control.
// This independently verifies the helper even if LoggingDecorator is repaired.
struct ContinuingUpdate<L>(L);

impl<L: Doublets<usize>> Doublets<usize> for ContinuingUpdate<L> {
    fn get_link(&self, index: usize) -> Option<Link<usize>> {
        self.0.get_link(index)
    }
}

impl<L: Doublets<usize>> Links<usize> for ContinuingUpdate<L> {
    fn constants(&self) -> &LinksConstants<usize> {
        self.0.constants()
    }

    fn count_links(&self, query: &[usize]) -> usize {
        self.0.count_links(query)
    }

    fn each_links(&self, query: &[usize], handler: ReadHandler<'_, usize>) -> Flow {
        self.0.each_links(query, handler)
    }

    fn create_links(
        &mut self,
        query: &[usize],
        handler: WriteHandler<'_, usize>,
    ) -> Result<Flow, Error<usize>> {
        self.0.create_links(query, handler)
    }

    fn update_links(
        &mut self,
        query: &[usize],
        change: &[usize],
        handler: WriteHandler<'_, usize>,
    ) -> Result<Flow, Error<usize>> {
        self.0.update_links(query, change, handler)?;
        Ok(Flow::Continue)
    }

    fn delete_links(
        &mut self,
        query: &[usize],
        handler: WriteHandler<'_, usize>,
    ) -> Result<Flow, Error<usize>> {
        self.0.delete_links(query, handler)
    }
}

fn creation_preserves_discarded_break(store: impl Doublets<usize>) -> Result<(), Error<usize>> {
    creation_flow(ContinuingUpdate(store))
}

fn logged_update_preserves_discarded_break(
    store: impl Doublets<usize>,
) -> Result<(), Error<usize>> {
    let mut store = ContinuingUpdate(store).with_logging(Vec::new());
    let point = store.create_point()?;
    assert_eq!(
        store.update_with(point, 0, 0, |_, _| Flow::Break)?,
        Flow::Break
    );
    assert_eq!(store.get_link(point), Some(Link::new(point, 0, 0)));
    Ok(())
}

macro_rules! test_stores {
    ($($test:ident),+ $(,)?) => {
        $(
            mod $test {
                use super::*;

                #[test]
                fn unit() -> Result<(), Error<usize>> {
                    $test(unit::Store::<usize, _>::new(Global::new())?)
                }

                #[test]
                fn split() -> Result<(), Error<usize>> {
                    $test(split::Store::<usize, _, _>::new(Global::new(), Global::new())?)
                }
            }
        )+
    };
}

test_stores!(
    creation_flow,
    logged_creation_flow,
    logged_mutations_preserve_break,
    logged_cascade_finishes_after_break,
    primitive_break_and_errors_survive,
    automatic_creation_flow,
    logged_creation_phases,
    logging_outside_automatic,
    logging_inside_automatic,
    logged_uniqueness_preserves_survivor,
    creation_preserves_discarded_break,
    logged_update_preserves_discarded_break,
);

#[test]
fn logging_write_error() {
    logging_error_takes_precedence(unit::Store::<usize, _>::new(Global::new()).unwrap());
}
