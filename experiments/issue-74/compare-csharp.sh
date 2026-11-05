#!/usr/bin/env bash
# Compare with the reviewed C# sources; no upstream files are modified.
set -euo pipefail
cd "$(dirname "$0")/../.."
review_dir="$PWD/target/issue-74-review"
source_dir="$review_dir/Data.Doublets"
revision=f8c521e539935e670f4f484cf451857d2bd3e693
mkdir -p "$review_dir"
if [ ! -d "$source_dir/.git" ]; then
    gh repo clone linksplatform/Data.Doublets "$source_dir" -- --depth 1 --no-checkout
fi
if [ "$(git -C "$source_dir" rev-parse HEAD)" != "$revision" ]; then
    git -C "$source_dir" fetch --depth 1 origin "$revision"
fi
git -C "$source_dir" checkout --detach "$revision"
test "$(git -C "$source_dir" rev-parse HEAD)" = "$revision"
export DOTNET_CLI_TELEMETRY_OPTOUT=1 DOTNET_NOLOGO=1
export DOTNET_ROLL_FORWARD=Major DOTNET_GCHeapHardLimit=0x30000000
# Use .NET 8 as in upstream; allow a newer installed runtime to execute it.
dotnet build experiments/issue-74/csharp/ForwardReferences.csproj \
    --disable-build-servers -p:TargetFramework=net8.0 \
    -p:DataDoubletsSource="$source_dir" -p:UseSharedCompilation=false \
    -m:1 -v:normal > "$review_dir/csharp-comparison.log" 2>&1
dotnet experiments/issue-74/csharp/bin/Debug/net8.0/ForwardReferences.dll \
    >> "$review_dir/csharp-comparison.log" 2>&1
tail -n 12 "$review_dir/csharp-comparison.log"
