//! Choose checked, ignore-missing or caller-guaranteed mutation on either layout.

use data::Flow;
use doublets::{
    decorators::{CheckedMutation, DecoratorsExt},
    mem::UncheckedMutation,
    split, unit, Doublets, Error, Link,
};
use mem::Global;

fn demonstrate(mut store: impl UncheckedMutation<usize>) -> Result<(), Error<usize>> {
    let a = store.create_point()?;
    let b = store.create()?;

    // Safe routes validate the requested address and reject without mutating.
    let missing = b + 1;
    assert!(matches!(store.update(missing, a, a), Err(Error::NotExists(i)) if i == missing));
    assert!(matches!(store.delete(missing), Err(Error::NotExists(i)) if i == missing));
    assert_eq!(store.count(), 2);

    // A caller that guarantees `b` is live omits that lookup.
    let mut events = Vec::new();
    // SAFETY: `b` was created above and only the store API mutated the indexes.
    unsafe {
        store.update_unchecked(b, a, b, &mut |before, after| {
            events.push((before, after));
            Flow::Continue
        });
    }
    assert_eq!(events, [(Link::new(b, 0, 0), Link::new(b, a, b))]);
    assert_eq!(store.search(a, b), Some(b));

    // Ignoring a missing deletion is an explicit policy that forwards errors.
    {
        let mut ignoring = CheckedMutation::new(&mut store).with_non_existent_deletion_resolution();
        assert_eq!(ignoring.delete(b)?, b);
        assert_eq!(ignoring.delete(b)?, 0);
    }

    // Unchecked deletion is the caller's reset followed by release.
    // SAFETY: `a` is live; the reset detaches its outgoing index memberships.
    unsafe {
        store.update_unchecked(a, 0, 0, &mut |_, _| Flow::Continue);
        store.release_unchecked(a, &mut |_, _| Flow::Continue);
    }
    assert_eq!(store.count(), 0);
    Ok(())
}

fn main() -> Result<(), Error<usize>> {
    demonstrate(unit::Store::<usize, _>::new(Global::new())?)?;
    demonstrate(split::Store::<usize, _, _>::new(
        Global::new(),
        Global::new(),
    )?)?;
    println!("checked, ignore-missing and unchecked mutation routes behave as documented");
    Ok(())
}
