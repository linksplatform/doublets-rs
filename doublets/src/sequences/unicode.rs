//! Unicode symbol and string converters using C# UTF-16 storage conventions.

use data::LinkReference;

use super::{
    AddressToRawNumberConverter, BalancedVariantConverter, CriterionMatcher,
    RawNumberToAddressConverter, RightSequenceWalker, SequenceError, TargetMatcher,
};
use crate::Doublets;

/// Converts a UTF-16 code unit (C# `char`) into `(raw number, symbol marker)`.
#[derive(Clone, Copy, Debug)]
pub struct CharToUnicodeSymbolConverter<T> {
    address_to_number: AddressToRawNumberConverter,
    symbol_marker: T,
}

impl<T: LinkReference> CharToUnicodeSymbolConverter<T> {
    /// Creates the converter with a raw-number encoder and caller-owned marker.
    #[must_use]
    pub const fn new(address_to_number: AddressToRawNumberConverter, symbol_marker: T) -> Self {
        Self {
            address_to_number,
            symbol_marker,
        }
    }

    fn raw_number<L: Doublets<T>>(&self, links: &L, code_unit: u16) -> Result<T, SequenceError<T>> {
        let address =
            T::try_from(code_unit).map_err(|_| SequenceError::CodeUnitDoesNotFit(code_unit))?;
        let raw = self.address_to_number.convert(address)?;
        if !links.constants().is_external(raw) {
            return Err(SequenceError::ExternalReferencesDisabled);
        }
        Ok(raw)
    }

    /// Stores or reuses a symbol, preserving every code unit including lone surrogates.
    /// Rejects code units outside the address range or stores without external support.
    pub fn convert<L: Doublets<T>>(
        &self,
        links: &mut L,
        code_unit: u16,
    ) -> Result<T, SequenceError<T>> {
        let raw = self.raw_number(links, code_unit)?;
        Ok(links.get_or_create(raw, self.symbol_marker)?)
    }
}

/// Converts a marked Unicode symbol back into a UTF-16 code unit.
#[derive(Clone, Copy, Debug)]
pub struct UnicodeSymbolToCharConverter<T, M = TargetMatcher<T>> {
    number_to_address: RawNumberToAddressConverter,
    matcher: M,
    address: std::marker::PhantomData<T>,
}

impl<T: LinkReference, M: CriterionMatcher<T>> UnicodeSymbolToCharConverter<T, M> {
    /// Creates a decoder with a raw-number decoder and symbol criterion.
    #[must_use]
    pub const fn new(number_to_address: RawNumberToAddressConverter, matcher: M) -> Self {
        Self {
            number_to_address,
            matcher,
            address: std::marker::PhantomData,
        }
    }

    /// Checks the symbol criterion and decodes its source, rejecting oversized payloads.
    pub fn convert<L: Doublets<T> + ?Sized>(
        &self,
        links: &L,
        symbol: T,
    ) -> Result<u16, SequenceError<T>> {
        let link = links.try_get_link(symbol)?;
        if !self.matcher.is_matched(links, symbol) {
            return Err(SequenceError::NotUnicodeSymbol(symbol));
        }
        let unit = self.number_to_address.convert(link.source);
        unit.try_into()
            .map_err(|_| SequenceError::CodeUnitOutOfRange(unit))
    }
}

/// Encodes strings as UTF-16 symbols in a marked balanced sequence.
#[derive(Clone, Copy, Debug)]
pub struct StringToUnicodeSequenceConverter<T> {
    symbol_converter: CharToUnicodeSymbolConverter<T>,
    balanced_converter: BalancedVariantConverter,
    sequence_marker: T,
}

impl<T: LinkReference> StringToUnicodeSequenceConverter<T> {
    /// Creates a converter from the symbol encoder, balancing strategy and marker.
    #[must_use]
    pub const fn new(
        symbol_converter: CharToUnicodeSymbolConverter<T>,
        balanced_converter: BalancedVariantConverter,
        sequence_marker: T,
    ) -> Self {
        Self {
            symbol_converter,
            balanced_converter,
            sequence_marker,
        }
    }

    /// Encodes a Rust string using UTF-16, including surrogate pairs for non-BMP text.
    /// An empty string returns the sequence marker without creating any links.
    pub fn convert<L: Doublets<T>>(
        &self,
        links: &mut L,
        content: &str,
    ) -> Result<T, SequenceError<T>> {
        self.convert_utf16(links, &content.encode_utf16().collect::<Vec<_>>())
    }

    /// Encodes raw UTF-16 units, preserving unpaired surrogates from C# strings.
    /// Validates numeric/external ranges before creating symbols; store failures
    /// can still leave already-created links, as store writes are not transactional.
    pub fn convert_utf16<L: Doublets<T>>(
        &self,
        links: &mut L,
        units: &[u16],
    ) -> Result<T, SequenceError<T>> {
        if units.is_empty() {
            return Ok(self.sequence_marker);
        }
        let raw_numbers = units
            .iter()
            .map(|&unit| self.symbol_converter.raw_number(links, unit))
            .collect::<Result<Vec<_>, _>>()?;
        let symbols = raw_numbers
            .into_iter()
            .map(|raw| links.get_or_create(raw, self.symbol_converter.symbol_marker))
            .collect::<Result<Vec<_>, _>>()?;
        let sequence = self.balanced_converter.convert(links, &symbols)?;
        Ok(links.get_or_create(sequence, self.sequence_marker)?)
    }
}

/// Decodes a marked Unicode sequence into UTF-16 units or a Rust string.
#[derive(Clone, Copy, Debug)]
pub struct UnicodeSequenceToStringConverter<
    T,
    S = TargetMatcher<T>,
    E = TargetMatcher<T>,
    M = TargetMatcher<T>,
> {
    sequence_matcher: S,
    walker: RightSequenceWalker<E>,
    symbol_converter: UnicodeSymbolToCharConverter<T, M>,
    sequence_marker: T,
}

impl<T, S, E, M> UnicodeSequenceToStringConverter<T, S, E, M>
where
    T: LinkReference,
    S: CriterionMatcher<T>,
    E: CriterionMatcher<T>,
    M: CriterionMatcher<T>,
{
    /// Creates a decoder with sequence/symbol criteria, a walker and empty marker.
    #[must_use]
    pub const fn new(
        sequence_matcher: S,
        walker: RightSequenceWalker<E>,
        symbol_converter: UnicodeSymbolToCharConverter<T, M>,
        sequence_marker: T,
    ) -> Self {
        Self {
            sequence_matcher,
            walker,
            symbol_converter,
            sequence_marker,
        }
    }

    /// Decodes a sequence, returning an error for unpaired UTF-16 surrogates.
    pub fn convert<L: Doublets<T> + ?Sized>(
        &self,
        links: &L,
        sequence: T,
    ) -> Result<String, SequenceError<T>> {
        Ok(String::from_utf16(&self.convert_utf16(links, sequence)?)?)
    }

    /// Decodes the exact stored UTF-16 units, including unpaired surrogates.
    /// The configured empty marker returns no units without a store lookup.
    pub fn convert_utf16<L: Doublets<T> + ?Sized>(
        &self,
        links: &L,
        sequence: T,
    ) -> Result<Vec<u16>, SequenceError<T>> {
        if sequence == self.sequence_marker {
            return Ok(Vec::new());
        }
        let link = links.try_get_link(sequence)?;
        if !self.sequence_matcher.is_matched(links, sequence) {
            return Err(SequenceError::NotUnicodeSequence(sequence));
        }
        if link.source == links.constants().null {
            return Err(crate::Error::NotExists(link.source).into());
        }
        self.walker
            .iter(links, link.source)
            .map(|symbol| self.symbol_converter.convert(links, symbol?))
            .collect()
    }
}
