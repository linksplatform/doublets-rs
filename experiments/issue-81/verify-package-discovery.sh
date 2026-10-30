#!/usr/bin/env bash
# Verify package checks discover every manifest even without ripgrep.
set -euo pipefail
cd "$(dirname "$0")/../.."
fixture=$(mktemp -d)
trap 'rm -rf "$fixture"' EXIT
mkdir -p "$fixture/bin" "$fixture/repo/experiments/issue-22" \
    "$fixture/repo/core" "$fixture/repo/ffi/macros"
for command in bash dirname git grep mkdir sort; do
    ln -s "$(command -v "$command")" "$fixture/bin/$command"
done
cat > "$fixture/bin/cargo" <<'EOF'
#!/usr/bin/env bash
echo "$*" >> "$PACKAGE_DISCOVERY_CALLS"
EOF
chmod +x "$fixture/bin/cargo"
cp experiments/issue-22/check-packages.sh "$fixture/repo/experiments/issue-22/"
git -C "$fixture/repo" init -q
printf '[workspace]\n' > "$fixture/repo/Cargo.toml"
printf '[package]\n' > "$fixture/repo/core/Cargo.toml"
printf '[package]\n' > "$fixture/repo/ffi/macros/Cargo.toml"
export PACKAGE_DISCOVERY_CALLS="$fixture/calls"
touch "$PACKAGE_DISCOVERY_CALLS"
PATH="$fixture/bin" bash "$fixture/repo/experiments/issue-22/check-packages.sh"
for manifest in core/Cargo.toml ffi/macros/Cargo.toml; do
    grep -Fx "check --manifest-path $manifest --all-targets --all-features --locked" \
        "$PACKAGE_DISCOVERY_CALLS"
done
[[ $(wc -l < "$PACKAGE_DISCOVERY_CALLS") -eq 2 ]]
rm "$fixture/repo/core/Cargo.toml" "$fixture/repo/ffi/macros/Cargo.toml"
if PATH="$fixture/bin" bash "$fixture/repo/experiments/issue-22/check-packages.sh" \
    > "$fixture/empty-output" 2>&1; then
    echo 'Package discovery accepted zero packages.' >&2
    exit 1
fi
grep -q 'No package manifests were checked.' "$fixture/empty-output"
echo 'Package discovery checks both packages without ripgrep.'
