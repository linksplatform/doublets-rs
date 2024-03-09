use {
    platform_mem::{FileMapped, RawMem},
    std::{error, fs::File, result},
};

type Result = result::Result<(), Box<dyn error::Error>>;

pub fn grow_from_slice(mut mem: impl RawMem<Item = u8>) {
    assert_eq!(b"hello world", mem.grow_from_slice(b"hello world").unwrap());
}

#[test]
fn yet() -> Result {
    use std::{fs, io::Write, str};

    // may be use tempfile???
    const FILE: &str = "tmp.file";
    const TAIL_SIZE: usize = 4 * 1024;

    let _ = fs::remove_file(FILE);
    {
        let mut file = File::options() // `create_new` feature
            .write(true)
            .create_new(true)
            .open(FILE)?;
        file.write_all(b"hello world")?;
        file.write_all(&[b'\0'; TAIL_SIZE])?;
    }

    unsafe {
        let mut mem = FileMapped::from_path(FILE)?;

        assert_eq!(b"hello world", mem.grow_assumed(5 + 1 + 5)?); // is size of `hello world`

        mem.grow(10_000, |inited, (_, uninit)| {
            assert_eq!(inited, TAIL_SIZE);
            assert_eq!(10_000, uninit.len());
        })?;
    }

    Ok(())
}
