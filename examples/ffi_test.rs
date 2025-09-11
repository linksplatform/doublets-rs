// Example demonstrating the improved ffi::specialize macros

extern crate ffi_attributes as ffi;

trait LinkType: Copy + Clone + std::fmt::Debug + PartialEq + Eq {}
impl LinkType for u8 {}
impl LinkType for u16 {}
impl LinkType for u32 {}
impl LinkType for u64 {}

// Example using the new simplified specialize macro (Option 2)
#[ffi::specialize("*_simple")]
unsafe fn simple_function<T: LinkType>(value: T) -> T {
    value
}

// Example using the existing specialize_for macro (backward compatible)
#[ffi::specialize_for(
    types = "u8",
    types = "u16", 
    types = "u32",
    types = "u64",
    convention = "csharp",
    name = "*_compatible"
)]
unsafe fn compatible_function<T: LinkType>(value: T) -> T {
    value
}

// Example showing convention-less usage (defaults to csharp)
#[ffi::specialize_for(
    types = "u8",
    types = "u16",
    types = "u32", 
    types = "u64",
    name = "*_default"
)]
unsafe fn default_convention_function<T: LinkType>(value: T) -> T {
    value
}

fn main() {
    println!("FFI specialization macros compiled successfully!");
}