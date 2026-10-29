//! Store-backed sequences and UTF-16 strings compatible with
//! `Platform.Data.Doublets.Sequences` 0.6.5.
//!
//! A balanced variant pairs adjacent elements at each layer and carries an odd
//! final element to the next layer. A Unicode symbol is `(raw code unit, symbol
//! marker)`; a nonempty string is `(balanced root, sequence marker)`. The sequence
//! marker itself denotes the empty string. Markers are supplied by the caller;
//! these converters do not allocate application-specific type addresses.
//!
//! Pass the store to each operation so converters can share one mutable store
//! without interior mutability. All store writes use [`crate::Doublets::get_or_create`].
//! No additional dependencies or feature flags are needed.
//!
//! # Encoding and errors
//!
//! Raw numbers use C# `Platform.Data.Hybrid` encoding: zero is `MAX / 2 + 1`,
//! and positive values are their unsigned two's-complement negatives. This is
//! intentionally implemented here because `platform-data` 2.0's `Hybrid` does
//! not encode external zero correctly. Unicode stores must enable external
//! references with `data::LinksConstants::external()`.
//!
//! C# `char` is a UTF-16 code unit, so the character converters accept/return
//! `u16`, and [`StringToUnicodeSequenceConverter`] uses `str::encode_utf16`.
//! Raw UTF-16 conversion methods preserve lone surrogates; converting those to
//! a Rust `String` returns an error. Missing links, cycles, invalid symbol
//! payloads and wrong markers also return errors instead of silently losing data.
//! A shared subtree is visited once per occurrence, preserving repeated content.
//!
//! ```
//! use doublets::{mem::Global, split, Doublets, sequences::*};
//! use doublets::data::LinksConstants;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut store = split::Store::<u32, _, _>::with_constants(
//!     Global::new(), Global::new(), LinksConstants::external(),
//! )?;
//! let symbol_marker = store.create_point()?;
//! let sequence_marker = store.create_point()?;
//! let symbols = TargetMatcher::new(symbol_marker);
//! let encoder = StringToUnicodeSequenceConverter::new(
//!     CharToUnicodeSymbolConverter::new(AddressToRawNumberConverter::new(), symbol_marker),
//!     BalancedVariantConverter::new(), sequence_marker,
//! );
//! let decoder = UnicodeSequenceToStringConverter::new(
//!     TargetMatcher::new(sequence_marker), RightSequenceWalker::new(symbols),
//!     UnicodeSymbolToCharConverter::new(RawNumberToAddressConverter::new(), symbols),
//!     sequence_marker,
//! );
//! let address = encoder.convert(&mut store, "Hello, 世界 🌍")?;
//! assert_eq!(decoder.convert(&store, address)?, "Hello, 世界 🌍");
//! # Ok(()) }
//! ```

pub mod converters;
mod error;
mod stack;
pub mod unicode;
pub mod walkers;

pub use crate::data::{CriterionMatcher, TargetMatcher};
pub use converters::{
    AddressToRawNumberConverter, BalancedVariantConverter, CachingConverterDecorator,
    RawNumberToAddressConverter,
};
pub use error::SequenceError;
pub use stack::DefaultStack;
pub use unicode::{
    CharToUnicodeSymbolConverter, StringToUnicodeSequenceConverter,
    UnicodeSequenceToStringConverter, UnicodeSymbolToCharConverter,
};
pub use walkers::RightSequenceWalker;
