# Solver Example

This example demonstrates the fixed version of the solver code that was previously failing to compile with the errors mentioned in issue #17.

## Fixed Issues

1. **E0412: cannot find type `Link` in scope**
   - The `Link` type was properly imported from `doublets` crate
   - The import statement was already correct in the main.rs

2. **E0121: placeholder `_` not allowed in function signatures** 
   - Replaced `unit::Store<usize, _>` with `unit::Store<usize, mem::Global<unit::LinkPart<usize>>>`
   - Added proper generic parameter specification for the `Global` type

## Running the Example

To run this example with the specific nightly toolchain mentioned in the issue:

```bash
# Install the required nightly toolchain
rustup toolchain install nightly-2022-08-22

# Build the example
cargo +nightly-2022-08-22 build

# Run the example
cargo +nightly-2022-08-22 run
```

The example will generate sequences of boolean logic operations using NAND gates and demonstrate how to work with the doublets data structure.

## What This Example Does

This solver example:
- Creates a doublets store for managing link relationships
- Generates sequences of different lengths using x and y placeholders
- Creates variants of these sequences using the doublets data structure
- Applies NAND operations to boolean combinations
- Demonstrates deep formatting of link structures
- Outputs the results of boolean logic evaluations