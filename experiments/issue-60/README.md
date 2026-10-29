# Sequence compatibility investigation

Issue [#60](https://github.com/linksplatform/doublets-rs/issues/60) reports that
consumers have to carry their own sequence/Unicode pipeline. The public API
regression in `doublets/tests/sequences.rs` initially failed to compile with
`unresolved import doublets::sequences` before the module was implemented.

The reference implementation is
[`Platform.Data.Doublets.Sequences` 0.6.5](https://github.com/linksplatform/Data.Doublets.Sequences/tree/csharp_0.6.5).
The raw-number converters actually live in `Platform.Data.Numbers.Raw`, and
`CachingConverterDecorator` lives in `Platform.Converters`. Their Rust
counterparts are exposed alongside the sequence converters for convenience.

The key encoding rules are:

- Pair adjacent elements at every layer, carrying an unpaired tail unchanged.
  A midpoint split produces a different tree for odd lengths.
- Encode UTF-16 units, including surrogate pairs, rather than Unicode scalars.
- Encode raw zero as `MAX / 2 + 1`, and positive raw magnitudes as unsigned
  two's-complement negatives. `platform-data` 2.0.0's `Hybrid::external(0)`
  returns null, and its `abs` does not decode external references correctly.
  The new converters implement the C# representation without changing that
  dependency's public behavior.
- Store `(raw unit, symbol marker)` and `(balanced root, sequence marker)`.
  The sequence marker itself denotes the empty string.

To regenerate and check the golden triples against the actual NuGet package,
install the .NET 10 SDK and run from the repository root:

```bash
bash experiments/issue-60/verify-compatibility.sh
```

This runs a C# encode/decode round trip of `"A\0😀"` and compares the resulting
triples with `csharp/expected.txt`. The Rust regression asserts those same eight
triples and verifies decoding, repeated encoding and empty strings. Build
artifacts and logs remain under the ignored `target/` directory.

The C# experiment passed with package version 0.6.5. The Rust tests additionally
cover both backends (`u32` and `u64`), raw-number boundaries on five integer
types, shared subtrees, odd tails, cache invalidation/error retries, missing
links, wrong markers, oversized payloads and lone surrogates. UTF-16 methods
preserve lone surrogates; a Rust `String` conversion reports them as invalid.

The bounded deep-walk regression builds 2,048 links on a thread with a 128 KiB
stack. It yields 2,049 elements without recursive traversal. Its first draft
used a self-point as the sole leaf, which collapsed every `(leaf, leaf)` pair
back to that point via `get_or_create`. The test now uses a marked symbol,
matching the real string pipeline. Terminal criteria must distinguish leaves
from pairs; the same ambiguity exists in the C# representation.

Cycle detection tracks only the active path. A global visited set would
incorrectly discard repeated subtrees. Traversal errors terminate the iterator,
and each new walk owns an independent stack. Rust also rejects malformed
sequence wrappers with a null root, out-of-range raw magnitudes and code-unit
payloads instead of silently dropping or truncating content.

CI investigation found a toolchain difference: local stable was Rust 1.98.1,
while [run 37857084681](https://github.com/linksplatform/doublets-rs/actions/runs/37857084681)
used 1.99.0. Its new `clippy::assert_is_empty` lint rejected existing assertions
in `traits.rs` (lines 165, 355, 548, 985) and `query_tests.rs` (line 393).
The downloaded job log is preserved locally at
`ci-logs/lint-37857084681.log`, with the errors on lines 686–755; the full
workflow log is `ci-logs/rust-ci-37857084681.log` (lines 2719–2791).

Running strict Clippy with Rust 1.99.0 reproduced the error and also caught the
new walker's empty-result assertion. All six assertions now compare against
typed empty arrays, preserving their conditions and showing values on failure.
Strict Clippy passes without suppressing the lint or changing CI policy.
