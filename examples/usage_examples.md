# FFI Specialization Macros Usage Examples

This demonstrates the improvements to the `ffi::specialize_for` macro and the new `ffi::specialize` macro.

## Before (Current Verbose Syntax)

```rust
#[ffi::specialize_for(
    types = "u8",
    types = "u16",
    types = "u32",
    types = "u64",
    convention = "csharp",
    name = "doublets_constants_*"
)]
unsafe fn get_constants<T: LinkType>() -> Constants<T> {
    // implementation
}
```

## After - Option 1: Flexible Type Mapping (IMPLEMENTED)

The existing syntax is still supported for backward compatibility, and now:

- The `convention` parameter is optional (defaults to csharp)
- More concise usage

```rust
#[ffi::specialize_for(
    types = "u8",
    types = "u16", 
    types = "u32",
    types = "u64",
    name = "doublets_constants_*"
)]
unsafe fn get_constants<T: LinkType>() -> Constants<T> {
    // implementation - convention defaults to csharp
}
```

## After - Option 2: Simplified Embedded Annotations (IMPLEMENTED)

```rust
#[ffi::specialize("doublets_constants_*")]
unsafe fn get_constants<T: LinkType>() -> Constants<T> {
    // implementation
    // Automatically generates for u8, u16, u32, u64 with C# naming
}
```

## Generated Functions

Both approaches generate the same FFI functions:

- `doublets_constants_Byte`    (for u8)
- `doublets_constants_UInt16`  (for u16)  
- `doublets_constants_UInt32`  (for u32)
- `doublets_constants_UInt64`  (for u64)

## Benefits

1. **Less repetition** - No need to repeat `types = "..."` for each type
2. **Simpler syntax** - The new `specialize` macro requires just the name pattern
3. **Backward compatibility** - Existing code continues to work unchanged
4. **Default convention** - No need to specify convention for common C# usage

## Implementation Status

✅ Both macros are implemented and working
✅ Backward compatibility maintained  
✅ Code compiles and tests pass
✅ Ready for use in production