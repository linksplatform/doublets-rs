//! Regression and compatibility tests for issue #60.

use data::{LinkReference, LinksConstants};
use doublets::{
    mem::Global,
    sequences::{
        AddressToRawNumberConverter, BalancedVariantConverter, CachingConverterDecorator,
        CharToUnicodeSymbolConverter, DefaultStack, RawNumberToAddressConverter,
        RightSequenceWalker, SequenceError, StringToUnicodeSequenceConverter, TargetMatcher,
        UnicodeSequenceToStringConverter, UnicodeSymbolToCharConverter,
    },
    split, unit, Doublets, Error, Link,
};

#[test]
fn csharp_unicode_pipeline_stores_exact_utf16_links() {
    let mut store = split::Store::<u32, _, _>::with_constants(
        Global::new(),
        Global::new(),
        LinksConstants::external(),
    )
    .unwrap();
    let symbol_marker = store.create_point().unwrap();
    let sequence_marker = store.create_point().unwrap();
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

    // C# chars are UTF-16 units: A, NUL, high surrogate, low surrogate.
    let sequence = encoder.convert(&mut store, "A\0😀").unwrap();
    let expected = [
        Link::new(3, 0xffff_ffbf, 1),
        Link::new(4, 0x8000_0000, 1),
        Link::new(5, 0xffff_27c3, 1),
        Link::new(6, 0xffff_2200, 1),
        Link::new(7, 3, 4),
        Link::new(8, 5, 6),
        Link::new(9, 7, 8),
        Link::new(10, 9, 2),
    ];
    assert_eq!(sequence, 10);
    for link in expected {
        assert_eq!(store.get_link(link.index), Some(link));
    }
    assert_eq!(decoder.convert(&store, sequence).unwrap(), "A\0😀");
    let count = store.count();
    assert_eq!(encoder.convert(&mut store, "A\0😀").unwrap(), sequence);
    assert_eq!(store.count(), count);
    assert_eq!(encoder.convert(&mut store, "").unwrap(), sequence_marker);
    assert_eq!(decoder.convert(&store, sequence_marker).unwrap(), "");
}

fn encoder<T: LinkReference>(symbol: T, sequence: T) -> StringToUnicodeSequenceConverter<T> {
    StringToUnicodeSequenceConverter::new(
        CharToUnicodeSymbolConverter::new(AddressToRawNumberConverter::new(), symbol),
        BalancedVariantConverter::new(),
        sequence,
    )
}

fn decoder<T: LinkReference>(symbol: T, sequence: T) -> UnicodeSequenceToStringConverter<T> {
    let symbols = TargetMatcher::new(symbol);
    UnicodeSequenceToStringConverter::new(
        TargetMatcher::new(sequence),
        RightSequenceWalker::new(symbols),
        UnicodeSymbolToCharConverter::new(RawNumberToAddressConverter::new(), symbols),
        sequence,
    )
}

fn check_raw_numbers<T: LinkReference>() {
    let encode = AddressToRawNumberConverter::new();
    let decode = RawNumberToAddressConverter::new();
    let half = T::MAX / T::from_byte(2);
    let zero = T::from_byte(0);
    let external_zero = half + T::from_byte(1);
    assert_eq!(encode.convert(zero).unwrap(), external_zero);
    assert_eq!(encode.convert(T::from_byte(1)).unwrap(), T::MAX);
    assert_eq!(
        encode.convert(half).unwrap(),
        external_zero + T::from_byte(1)
    );
    for magnitude in [zero, T::from_byte(1), T::from_byte(65), half] {
        assert_eq!(
            decode.convert(encode.convert(magnitude).unwrap()),
            magnitude
        );
    }
    assert!(matches!(
        encode.convert(external_zero),
        Err(SequenceError::RawNumberOutOfRange(_))
    ));
    assert!(matches!(
        encode.convert(T::MAX),
        Err(SequenceError::RawNumberOutOfRange(_))
    ));
    assert_eq!(decode.convert(T::from_byte(65)), T::from_byte(65));
}

#[test]
fn csharp_raw_number_encoding_is_generic_and_checked() {
    check_raw_numbers::<u8>();
    check_raw_numbers::<u16>();
    check_raw_numbers::<u32>();
    check_raw_numbers::<u64>();
    check_raw_numbers::<usize>();
}

#[test]
fn balanced_variants_carry_odd_tails_and_reuse_shared_subtrees() {
    let mut store = unit::Store::<u32, _>::new(Global::new()).unwrap();
    let elements: Vec<_> = (0..7).map(|_| store.create_point().unwrap()).collect();
    let converter = BalancedVariantConverter::new();
    let walker = RightSequenceWalker::new(|address| (1..=7).contains(&address));
    for length in 0..=7 {
        let root = converter.convert(&mut store, &elements[..length]).unwrap();
        assert_eq!(walker.walk(&store, root).unwrap(), elements[..length]);
        let count = store.count();
        assert_eq!(
            converter.convert(&mut store, &elements[..length]).unwrap(),
            root
        );
        assert_eq!(store.count(), count);
    }
    // Pairwise layering for five leaves is (((a,b),(c,d)),e), rather than
    // splitting the input at its midpoint into ((a,b),(c,(d,e))).
    let ab = store.search(1, 2).unwrap();
    let cd = store.search(3, 4).unwrap();
    let abcd = store.search(ab, cd).unwrap();
    let five = converter.convert(&mut store, &elements[..5]).unwrap();
    assert_eq!(store.get_link(five), Some(Link::new(five, abcd, 5)));
    let repeated = converter.convert(&mut store, &[1, 2, 1, 2]).unwrap();
    assert_eq!(store.get_link(repeated), Some(Link::new(repeated, ab, ab)));
    assert_eq!(walker.walk(&store, repeated).unwrap(), [1, 2, 1, 2]);
}

fn check_strings<T: LinkReference>(mut store: impl Doublets<T>) {
    let symbol = store.create_point().unwrap();
    let sequence = store.create_point().unwrap();
    let encode = encoder(symbol, sequence);
    let decode = decoder(symbol, sequence);
    for text in [
        "",
        "A",
        "\0",
        "abcde",
        "aaaaaaa",
        "Привет 世界",
        "e\u{301}",
        "😀🌍",
        "a\0b",
    ] {
        let root = encode.convert(&mut store, text).unwrap();
        assert_eq!(decode.convert(&store, root).unwrap(), text);
        assert_eq!(
            decode.convert_utf16(&store, root).unwrap(),
            text.encode_utf16().collect::<Vec<_>>()
        );
        let count = store.count();
        assert_eq!(encode.convert(&mut store, text).unwrap(), root);
        assert_eq!(store.count(), count);
    }
}

#[test]
fn unicode_strings_round_trip_on_both_backends_and_address_widths() {
    check_strings(
        unit::Store::<u32, _>::with_constants(Global::new(), LinksConstants::external()).unwrap(),
    );
    check_strings(
        split::Store::<u64, _, _>::with_constants(
            Global::new(),
            Global::new(),
            LinksConstants::external(),
        )
        .unwrap(),
    );
}

#[test]
fn unicode_decoders_preserve_units_and_reject_invalid_data() {
    let mut store =
        unit::Store::<u32, _>::with_constants(Global::new(), LinksConstants::external()).unwrap();
    let symbol = store.create_point().unwrap();
    let sequence = store.create_point().unwrap();
    let encode = encoder(symbol, sequence);
    let decode = decoder(symbol, sequence);
    let units = [0, 0xd800, 0xffff, 0xdc00];
    let root = encode.convert_utf16(&mut store, &units).unwrap();
    assert_eq!(decode.convert_utf16(&store, root).unwrap(), units);
    assert!(matches!(
        decode.convert(&store, root),
        Err(SequenceError::InvalidUtf16(_))
    ));
    let matcher = TargetMatcher::new(symbol);
    assert_eq!(matcher.target(), symbol);
    assert!(!matcher.is_matched(&store, 999));
    assert!(matcher.is_matched(&store, symbol));
    let char_decode =
        UnicodeSymbolToCharConverter::new(RawNumberToAddressConverter::new(), matcher);
    assert!(matches!(
        char_decode.convert(&store, sequence),
        Err(SequenceError::NotUnicodeSymbol(_))
    ));
    assert!(matches!(
        decode.convert(&store, symbol),
        Err(SequenceError::NotUnicodeSequence(_))
    ));
    assert!(matches!(
        decode.convert(&store, 999),
        Err(SequenceError::Store(Error::NotExists(999)))
    ));
    let too_large = store.get_or_create(0xffff_0000, symbol).unwrap();
    assert!(matches!(
        char_decode.convert(&store, too_large),
        Err(SequenceError::CodeUnitOutOfRange(65536))
    ));
    let missing = store.get_or_create(999, sequence).unwrap();
    assert!(matches!(
        decode.convert(&store, missing),
        Err(SequenceError::Store(Error::NotExists(999)))
    ));
    let null = store.get_or_create(0, sequence).unwrap();
    assert!(matches!(
        decode.convert(&store, null),
        Err(SequenceError::Store(Error::NotExists(0)))
    ));
}

#[test]
fn unicode_requires_external_support_before_writing_symbols() {
    let mut store = unit::Store::<u32, _>::new(Global::new()).unwrap();
    let symbol = store.create_point().unwrap();
    let sequence = store.create_point().unwrap();
    assert!(matches!(
        encoder(symbol, sequence).convert(&mut store, "A\0"),
        Err(SequenceError::ExternalReferencesDisabled)
    ));
    assert_eq!(store.count(), 2);
    // Narrow addresses can be checked without creating a narrow-address store.
    assert!(matches!(
        AddressToRawNumberConverter::new().convert(0xffff_u16),
        Err(SequenceError::RawNumberOutOfRange(_))
    ));
}

#[test]
fn walker_errors_terminate_and_do_not_affect_future_walks() {
    let mut store = unit::Store::<u32, _>::new(Global::new()).unwrap();
    let a = store.create_point().unwrap();
    let cycle = store.create_point().unwrap();
    let walker = RightSequenceWalker::new(|address| address == a);
    assert!(
        matches!(walker.walk(&store, cycle), Err(SequenceError::CyclicSequence(address)) if address == cycle)
    );
    let second = store.create_link(a, cycle).unwrap();
    store.update(cycle, second, a).unwrap();
    let mut iter = walker.iter(&store, second);
    assert_eq!(iter.next().unwrap().unwrap(), a);
    assert!(
        matches!(iter.next(), Some(Err(SequenceError::CyclicSequence(address))) if address == second)
    );
    assert!(iter.next().is_none());
    assert!(iter.next().is_none());
    let missing = store.create_link(a, 999).unwrap();
    assert!(matches!(
        walker.walk(&store, missing),
        Err(SequenceError::Store(Error::NotExists(999)))
    ));
    assert_eq!(walker.walk(&store, a).unwrap(), [a]);
    assert_eq!(walker.walk(&store, 0).unwrap(), [] as [u32; 0]);
}

#[test]
fn walker_handles_bounded_deep_sequences_on_a_small_thread_stack() {
    std::thread::Builder::new()
        .stack_size(128 * 1024)
        .spawn(|| {
            let mut store =
                unit::Store::<u32, _>::with_constants(Global::new(), LinksConstants::external())
                    .unwrap();
            let marker = store.create_point().unwrap();
            let a = store.create_link(0xffff_ffbf, marker).unwrap();
            let mut root = a;
            for _ in 0..2048 {
                root = store.get_or_create(root, a).unwrap();
            }
            let walker = RightSequenceWalker::new(|address| address == a);
            let output = walker.walk(&store, root).unwrap();
            assert_eq!(output.len(), 2049);
            assert!(output.iter().all(|&address| address == a));
            // The iterator also lets a consumer stop without collecting the sequence.
            assert_eq!(walker.iter(&store, root).take(3).count(), 3);
        })
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn cache_memoizes_successes_retries_errors_and_can_be_invalidated() {
    let mut cache = CachingConverterDecorator::<String, usize>::new();
    let mut calls = 0;
    for _ in 0..2 {
        assert_eq!(
            cache
                .convert_with("abc".to_owned(), |text| {
                    calls += 1;
                    Ok::<_, ()>(text.len())
                })
                .unwrap(),
            3
        );
    }
    assert_eq!(calls, 1);
    assert_eq!(cache.len(), 1);
    assert_eq!(cache.get(&"abc".to_owned()), Some(3));
    assert_eq!(cache.convert_with("bad".to_owned(), |_| Err(())), Err(()));
    assert_eq!(cache.len(), 1);
    assert_eq!(
        cache.convert_with("bad".to_owned(), |_| Ok::<_, ()>(42)),
        Ok(42)
    );
    cache.clear();
    assert!(cache.is_empty());
}

#[test]
fn default_stack_is_lifo_and_can_be_reused_after_clear() {
    let mut stack = DefaultStack::new();
    assert_eq!(stack.pop(), None);
    stack.push(1);
    stack.push(2);
    assert_eq!(stack.peek(), Some(&2));
    assert_eq!(stack.len(), 2);
    assert_eq!(stack.pop(), Some(2));
    stack.clear();
    assert!(stack.is_empty());
    stack.push(3);
    assert_eq!(stack.pop(), Some(3));
}

// A minimal store that fails every creation lets us test range validation and
// propagation of storage errors without allocating narrow-address backends.
struct FailingStore<T: LinkReference> {
    constants: LinksConstants<T>,
    writes: usize,
}

impl<T: LinkReference> doublets::Links<T> for FailingStore<T> {
    fn constants(&self) -> &LinksConstants<T> {
        &self.constants
    }

    fn count_links(&self, _query: &[T]) -> T {
        T::from_byte(0)
    }

    fn each_links(&self, _query: &[T], _handler: doublets::data::ReadHandler<'_, T>) -> data::Flow {
        data::Flow::Continue
    }

    fn create_links(
        &mut self,
        _query: &[T],
        _handler: doublets::data::WriteHandler<'_, T>,
    ) -> Result<data::Flow, Error<T>> {
        self.writes += 1;
        Err(Error::LimitReached(T::MAX))
    }

    fn update_links(
        &mut self,
        _query: &[T],
        _change: &[T],
        _handler: doublets::data::WriteHandler<'_, T>,
    ) -> Result<data::Flow, Error<T>> {
        unreachable!("creation fails before update")
    }

    fn delete_links(
        &mut self,
        _query: &[T],
        _handler: doublets::data::WriteHandler<'_, T>,
    ) -> Result<data::Flow, Error<T>> {
        unreachable!("sequence conversion does not delete links")
    }
}

impl<T: LinkReference> Doublets<T> for FailingStore<T> {
    fn get_link(&self, _index: T) -> Option<Link<T>> {
        None
    }
}

#[test]
fn narrow_addresses_are_checked_and_storage_errors_propagate() {
    let mut tiny = FailingStore {
        constants: LinksConstants::<u8>::external(),
        writes: 0,
    };
    let symbol = CharToUnicodeSymbolConverter::new(AddressToRawNumberConverter::new(), 1);
    assert!(matches!(
        symbol.convert(&mut tiny, 256),
        Err(SequenceError::CodeUnitDoesNotFit(256))
    ));
    assert!(matches!(
        symbol.convert(&mut tiny, 128),
        Err(SequenceError::RawNumberOutOfRange(128))
    ));
    assert_eq!(tiny.writes, 0);
    assert!(matches!(
        symbol.convert(&mut tiny, 65),
        Err(SequenceError::Store(Error::LimitReached(255)))
    ));
    assert_eq!(tiny.writes, 1);
    let mut narrow = FailingStore {
        constants: LinksConstants::<u16>::external(),
        writes: 0,
    };
    assert!(matches!(
        encoder(1, 2).convert_utf16(&mut narrow, &[65, 0xffff]),
        Err(SequenceError::RawNumberOutOfRange(0xffff))
    ));
    assert_eq!(narrow.writes, 0);
    assert!(matches!(
        BalancedVariantConverter::new().convert(&mut narrow, &[1, 2]),
        Err(Error::LimitReached(0xffff))
    ));
    assert!(matches!(
        encoder(1, 2).convert(&mut narrow, "abc"),
        Err(SequenceError::Store(Error::LimitReached(0xffff)))
    ));
}
