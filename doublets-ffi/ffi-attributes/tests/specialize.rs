use ffi_attributes::specialize_for;

#[specialize_for(
    types = "u32",
    types = "u64",
    convention = "csharp",
    name = "*Identity"
)]
unsafe fn identity<T>(value: T) -> T {
    value
}

#[test]
fn specializes_arguments_returns_and_export_names() {
    // SAFETY: the generated identity functions have no safety preconditions.
    unsafe {
        assert_eq!(UInt32Identity(42), 42_u32);
        assert_eq!(UInt64Identity(u64::MAX), u64::MAX);
    }
}
