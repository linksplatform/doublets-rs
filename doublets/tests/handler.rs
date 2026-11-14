use data::Flow;
use doublets::{Fuse, Link};

#[test]
fn fuse_new() {
    let handler = |_before: Link<usize>, _after: Link<usize>| Flow::Continue;
    let _fuse = Fuse::new(handler);
}

#[test]
fn fuse_call_continue() {
    let handler = |_before: Link<usize>, _after: Link<usize>| Flow::Continue;
    let mut fuse = Fuse::new(handler);

    let link1 = Link::<usize>::new(1, 1, 1);
    let link2 = Link::<usize>::new(2, 2, 2);

    let result = fuse.call(link1.clone(), link2.clone());
    assert!(matches!(result, Flow::Continue));

    let result = fuse.call(link1, link2);
    assert!(matches!(result, Flow::Continue));
}

#[test]
fn fuse_call_break() {
    let handler = |_before: Link<usize>, _after: Link<usize>| Flow::Break;
    let mut fuse = Fuse::new(handler);

    let link1 = Link::<usize>::new(1, 1, 1);
    let link2 = Link::<usize>::new(2, 2, 2);

    let result = fuse.call(link1.clone(), link2.clone());
    assert!(matches!(result, Flow::Break));

    let result = fuse.call(link1, link2);
    assert!(matches!(result, Flow::Break));
}

#[test]
fn fuse_call_once() {
    let handler = |_before: Link<usize>, _after: Link<usize>| Flow::Continue;
    let mut fuse = Fuse::new(handler);

    let link1 = Link::<usize>::new(1, 1, 1);
    let link2 = Link::<usize>::new(2, 2, 2);

    let result = fuse.call(link1, link2);
    assert!(matches!(result, Flow::Continue));
}

#[test]
fn fuse_done_flag_behavior() {
    let mut call_count = 0;
    let handler = move |_before: Link<usize>, _after: Link<usize>| {
        call_count += 1;
        if call_count == 1 {
            Flow::Break
        } else {
            Flow::Continue
        }
    };
    let mut fuse = Fuse::new(handler);

    let link1 = Link::<usize>::new(1, 1, 1);
    let link2 = Link::<usize>::new(2, 2, 2);

    let result1 = fuse.call(link1.clone(), link2.clone());
    assert!(matches!(result1, Flow::Break));

    let result2 = fuse.call(link1, link2);
    assert!(matches!(result2, Flow::Break));
}
