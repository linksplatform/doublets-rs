use {platform_mem::RawMem, std::error::Error};

type Result = std::result::Result<(), Box<dyn Error>>;

pub fn miri(mut mem: impl RawMem<Item = String>) -> Result {
    const GROW: usize = if cfg!(miri) { 100 } else { 10_000 };

    let val = String::from("foo");

    for _ in 0..10 {
        mem.grow_filled(GROW, val.clone())?;
    }
    assert_eq!(mem.allocated(), &vec![val; GROW * 10][..]);

    for _ in 0..10 {
        mem.shrink(GROW)?;
    }
    assert_eq!(mem.allocated().len(), 0);

    Ok(())
}
