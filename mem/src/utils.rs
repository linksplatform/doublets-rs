use {
    crate::raw_place::RawPlace,
    std::{
        fmt,
        fmt::{DebugStruct, Formatter},
    },
};

pub fn debug_mem<'a, 'b: 'a, T>(
    f: &'a mut Formatter<'b>,
    buf: &RawPlace<T>,
    alt: &str,
) -> Result<DebugStruct<'a, 'b>, fmt::Error> {
    write!(f, "{:?} ", buf)?;
    Ok(f.debug_struct(alt))
}
