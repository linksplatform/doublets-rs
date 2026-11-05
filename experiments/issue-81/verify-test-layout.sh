#!/usr/bin/env bash
# Exercise the layout guard on finite fixtures without ripgrep on PATH.
set -euo pipefail
cd "$(dirname "$0")/../.."
fixture=$(mktemp -d)
trap 'rm -rf "$fixture"' EXIT
mkdir -p "$fixture/bin" "$fixture/repo/experiments/issue-81" \
    "$fixture/repo/src" "$fixture/repo/tests/internal"
for command in bash dirname git; do
    ln -s "$(command -v "$command")" "$fixture/bin/$command"
done
cp experiments/issue-81/check-test-layout.sh "$fixture/repo/experiments/issue-81/"
git -C "$fixture/repo" init -q
cat > "$fixture/repo/tests/internal/allowed.rs" <<'EOF'
#[test]
fn allowed() {}
EOF
check() {
    PATH="$fixture/bin" bash "$fixture/repo/experiments/issue-81/check-test-layout.sh"
}
check
for attribute in '#[test]' '#[tokio::test]' '#[tokio::test(flavor = "current_thread")]'; do
    printf '%s\nfn forbidden() {}\n' "$attribute" > "$fixture/repo/src/lib.rs"
    if check > "$fixture/output" 2>&1; then
        echo "Layout guard accepted $attribute under src/." >&2
        exit 1
    else
        result=$?
        if [[ $result != 1 ]]; then cat "$fixture/output"; exit "$result"; fi
    fi
    grep -q 'src/lib.rs:1:' "$fixture/output"
done
rm "$fixture/repo/src/lib.rs"
check
echo 'Layout guard accepts tests/ and rejects source tests without ripgrep.'
