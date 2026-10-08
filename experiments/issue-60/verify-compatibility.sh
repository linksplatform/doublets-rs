#!/usr/bin/env bash
# Run from the repository root; keep SDK build output and logs out of the source tree.
set -euo pipefail
mkdir -p target/issue-60-logs
DOTNET_NOLOGO=1 dotnet run --project experiments/issue-60/csharp/Compatibility.csproj \
    --artifacts-path target/issue-60-csharp > target/issue-60-logs/csharp-compatibility.log 2>&1
rg '^[0-9]+: [0-9a-f]{8} [0-9a-f]{8}$' target/issue-60-logs/csharp-compatibility.log \
    > target/issue-60-logs/csharp-triples.txt
diff -u experiments/issue-60/csharp/expected.txt target/issue-60-logs/csharp-triples.txt
cargo test -p doublets --test sequences csharp_unicode_pipeline_stores_exact_utf16_links
