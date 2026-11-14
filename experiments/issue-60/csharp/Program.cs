// Generate the golden triples used by doublets/tests/sequences.rs with the
// actual C# package cited in issue #60, rather than a second implementation.
using Platform.Collections.Stacks;
using Platform.Data;
using Platform.Data.Doublets;
using Platform.Data.Doublets.CriterionMatchers;
using Platform.Data.Doublets.Memory.United.Generic;
using Platform.Data.Doublets.Sequences.Converters;
using Platform.Data.Doublets.Sequences.Unicode;
using Platform.Data.Doublets.Sequences.Walkers;
using Platform.Data.Numbers.Raw;
using Platform.Memory;

var path = Path.GetTempFileName();
try
{
    var constants = new LinksConstants<uint>(enableExternalReferencesSupport: true);
    using var memory = new FileMappedResizableDirectMemory(path, UnitedMemoryLinks<uint>.DefaultLinksSizeStep);
    using var links = new UnitedMemoryLinks<uint>(memory, UnitedMemoryLinks<uint>.DefaultLinksSizeStep,
        constants, Platform.Data.Doublets.Memory.IndexTreeType.Default);
    var symbolMarker = links.CreatePoint();
    var sequenceMarker = links.CreatePoint();
    var symbols = new TargetMatcher<uint>(links, symbolMarker);
    var encoder = new StringToUnicodeSequenceConverter<uint>(links,
        new CharToUnicodeSymbolConverter<uint>(links, new AddressToRawNumberConverter<uint>(), symbolMarker),
        new BalancedVariantConverter<uint>(links), sequenceMarker);
    var decoder = new UnicodeSequenceToStringConverter<uint>(links,
        new TargetMatcher<uint>(links, sequenceMarker),
        new RightSequenceWalker<uint>(links, new DefaultStack<uint>(), symbols.IsMatched),
        new UnicodeSymbolToCharConverter<uint>(links, new RawNumberToAddressConverter<uint>(), symbols),
        sequenceMarker);
    var root = encoder.Convert("A\0😀");
    if (root != 10 || decoder.Convert(root) != "A\0😀")
        throw new Exception("Unexpected sequence encoding");
    for (uint i = 3; i <= root; i++)
        Console.WriteLine($"{i}: {links.GetSource(i):x8} {links.GetTarget(i):x8}");
}
finally
{
    File.Delete(path);
}
