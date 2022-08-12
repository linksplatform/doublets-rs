use data::LinkType;
use doublets::{unit, Doublets, DoubletsExt, Link};
use mem::Global;
use std::error::Error;
use tap::Pipe;

fn rebase_impl<T: LinkType>(mut store: impl Doublets<T>) -> Result<(), Box<dyn Error>> {
    let a = store.create_point()?;
    let b = store.create_point()?;

    let c = store.create_point()?.pipe(|x| store.update(x, x, a))?;
    let d = store.create_point()?.pipe(|x| store.update(x, a, x))?;

    assert_eq!(
        store.iter().collect::<Vec<_>>(),
        vec![
            Link::new(a, a, a),
            Link::new(b, b, b),
            Link::new(c, c, a),
            Link::new(d, a, d)
        ]
    );

    store.rebase(a, b)?;

    assert_eq!(
        store.iter().collect::<Vec<_>>(),
        vec![
            Link::new(a, a, a),
            Link::new(b, b, b),
            Link::new(c, c, b),
            Link::new(d, b, d)
        ]
    );

    Ok(())
}

#[test]
fn rebase() {
    rebase_impl(unit::Store::<usize, _>::new(Global::new()).unwrap()).unwrap();
}
