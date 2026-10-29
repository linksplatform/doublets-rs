//! Store and retrieve Unicode strings without an application-local sequence port.
use doublets::{
    data::LinksConstants,
    mem::Global,
    sequences::{
        AddressToRawNumberConverter, BalancedVariantConverter, CachingConverterDecorator,
        CharToUnicodeSymbolConverter, RawNumberToAddressConverter, RightSequenceWalker,
        StringToUnicodeSequenceConverter, TargetMatcher, UnicodeSequenceToStringConverter,
        UnicodeSymbolToCharConverter,
    },
    split, Doublets,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut store = split::Store::<u32, _, _>::with_constants(
        Global::new(),
        Global::new(),
        LinksConstants::external(),
    )?;
    let symbol_marker = store.create_point()?;
    let sequence_marker = store.create_point()?;
    let symbols = TargetMatcher::new(symbol_marker);
    let encoder = StringToUnicodeSequenceConverter::new(
        CharToUnicodeSymbolConverter::new(AddressToRawNumberConverter::new(), symbol_marker),
        BalancedVariantConverter::new(),
        sequence_marker,
    );
    let decoder = UnicodeSequenceToStringConverter::new(
        TargetMatcher::new(sequence_marker),
        RightSequenceWalker::new(symbols),
        UnicodeSymbolToCharConverter::new(RawNumberToAddressConverter::new(), symbols),
        sequence_marker,
    );
    let mut cache = CachingConverterDecorator::new();
    for text in ["", "Hello, 世界 🌍", "A\0😀", "Hello, 世界 🌍"] {
        let address =
            cache.convert_with(text.to_owned(), |input| encoder.convert(&mut store, input))?;
        let recovered = decoder.convert(&store, address)?;
        assert_eq!(text, recovered);
        println!("{address}: {recovered:?}");
    }
    Ok(())
}
