#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

# Resolve the test executable before starting the memory-limited scope.
test_binary=$(
    cargo test --locked -p doublets --all-features --test file_mapped_growth \
        --no-run --message-format=json |
        jq -er 'select(.target.name == "file_mapped_growth" and .executable != null) | .executable'
)

sudo -n systemd-run --wait --pipe --collect --quiet --uid="$(id -u)" \
    --property=MemoryMax=24M --property=MemorySwapMax=0 --property=RuntimeMaxSec=30s \
    /bin/sh -eu -c '
        read -r cgroup < /proc/self/cgroup
        case "$cgroup" in
            0::/*) cgroup="/sys/fs/cgroup${cgroup#0::}" ;;
            *) echo "Expected a cgroup v2 test scope" >&2; exit 1 ;;
        esac
        max=$(cat "$cgroup/memory.max")
        swap=$(cat "$cgroup/memory.swap.max")
        printf "cgroup memory limit %s bytes, swap limit %s bytes\n" "$max" "$swap"
        test "$max" = 25165824
        test "$swap" = 0
        exec "$1" --test-threads=1
    ' sh "$test_binary"
