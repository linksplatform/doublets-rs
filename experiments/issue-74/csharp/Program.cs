using Platform.Data;
using Platform.Data.Doublets;
using Platform.Data.Doublets.Memory;
using Platform.Data.Doublets.Memory.Split.Generic;
using Platform.Data.Doublets.Memory.United.Generic;
using Platform.Memory;

static void Check(ILinks<ulong> links, ulong source, ulong target, string stage)
{
    var any = links.Constants.Any;
    foreach (var query in new[] { new[] { any, source, target }, new[] { any, source, any }, new[] { any, any, target }, new[] { any, source, 99UL } })
    {
        var found = new List<ulong>();
        links.Each(link => { found.Add(link[0]); return links.Constants.Continue; }, query);
        var expected = new List<ulong>();
        links.Each(link => {
            if ((query[1] == any || link[1] == query[1]) && (query[2] == any || link[2] == query[2]))
                expected.Add(link[0]);
            return links.Constants.Continue;
        }, Array.Empty<ulong>());
        found.Sort();
        expected.Sort();
        if (!found.SequenceEqual(expected) || links.Count(query) != (ulong)expected.Count)
            throw new Exception($"{stage}: [{string.Join(",", query)}] -> [{string.Join(",", found)}], expected [{string.Join(",", expected)}]");
    }
    if (links.GetSource(2UL) != source || links.GetTarget(2UL) != target)
        throw new Exception($"{stage}: link 2 changed");
}

// Small IDs fit the reserved memory even before their links exist.
foreach (var external in new[] { false, true })
foreach (var sourceReference in new[] { false, true })
foreach (var backend in new[] { "united", "split-list", "split-tree" })
{
    var constants = new LinksConstants<ulong>(enableExternalReferencesSupport: external);
    using var data = new HeapResizableDirectMemory();
    using var index = new HeapResizableDirectMemory();
    ILinks<ulong> links = backend == "united"
        ? new UnitedMemoryLinks<ulong>(data, 4096, constants, IndexTreeType.Default)
        : new SplitMemoryLinks<ulong>(data, index, 4096, constants, IndexTreeType.Default, backend == "split-list");
    using var store = (IDisposable)links;
    links.CreatePoint();
    links.CreatePoint();
    ulong source = sourceReference ? 3UL : 2UL;
    ulong target = sourceReference ? 2UL : 3UL;
    links.Update(2UL, source, target);
    if (links.Count(new[] { 3UL }) != 0)
        throw new Exception("ID 3 must not exist yet");
    Check(links, source, target, "before creation");
    links.Update(1UL, sourceReference ? 3UL : 1UL, sourceReference ? 1UL : 3UL);
    Check(links, source, target, "multiple usages before creation");
    if (links.CreatePoint() != 3)
        throw new Exception("Expected ID 3");
    if (links.Count(new[] { 3UL }) != 1)
        throw new Exception("ID 3 must now exist");
    Check(links, source, target, "after creation");
    Console.WriteLine($"{backend}, external range={external}, (2: {source} {target}): exact query -> [2] before and after creating ID 3; wildcard/count checks passed");
}
