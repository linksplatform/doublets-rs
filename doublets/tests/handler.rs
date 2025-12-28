use data::Flow;
use doublets::{Fuse, Handler, Link};

#[test]
fn fuse_new() {
    let handler = |_before: Link<usize>, _after: Link<usize>| Flow::Continue;
    let _fuse: Fuse<usize, _, Flow> = Fuse::new(handler);
}

#[test]
fn fuse_from() {
    let handler = |_before: Link<usize>, _after: Link<usize>| Flow::Continue;
    let _fuse: Fuse<usize, _, Flow> = Fuse::from(handler);
}

#[test]
fn handler_fuse_method() {
    let handler = |_before: Link<usize>, _after: Link<usize>| Flow::Continue;
    let _fuse = handler.fuse();
}

#[test]
fn fuse_call_continue() {
    let handler = |_before: Link<usize>, _after: Link<usize>| Flow::Continue;
    let mut fuse: Fuse<usize, _, Flow> = Fuse::new(handler);

    let link1 = Link::<usize>::new(1, 1, 1);
    let link2 = Link::<usize>::new(2, 2, 2);

    let result = fuse(link1.clone(), link2.clone());
    assert!(matches!(result, Flow::Continue));

    let result = fuse(link1, link2);
    assert!(matches!(result, Flow::Continue));
}

#[test]
fn fuse_call_break() {
    let handler = |_before: Link<usize>, _after: Link<usize>| Flow::Break;
    let mut fuse: Fuse<usize, _, Flow> = Fuse::new(handler);

    let link1 = Link::<usize>::new(1, 1, 1);
    let link2 = Link::<usize>::new(2, 2, 2);

    let result = fuse(link1.clone(), link2.clone());
    assert!(matches!(result, Flow::Break));

    // After break, fuse should return Break immediately (done flag)
    let result = fuse(link1, link2);
    assert!(matches!(result, Flow::Break));
}

#[test]
fn fuse_call_once() {
    let handler = |_before: Link<usize>, _after: Link<usize>| Flow::Continue;
    let mut fuse: Fuse<usize, _, Flow> = Fuse::new(handler);

    let link1 = Link::<usize>::new(1, 1, 1);
    let link2 = Link::<usize>::new(2, 2, 2);

    let result = fuse(link1, link2);
    assert!(matches!(result, Flow::Continue));
}
