# Issue #63 investigation

`doublets/tests/usages.rs` reproduces overlapping usages on both backends. Before
the fix, the minimal four-link fixture returned `2` usages and `[4, 4]`, and usage
deletion returned `NotExists(4)`. The three-link query fixture returned
`NotExists(1)` after deleting its matches.

Run the automated regression suite with:

```sh
cargo test -p doublets --test usages --all-features
```

## Separate split-store update limitation

The bounded `self-reference-probe.rs` uses at most four links. It also exposed a
pre-existing problem outside the overlap fix: changing `(2: 2 2)` to `(2: 2 1)`
after deleting a usage causes the split store's source index to report zero
matches for `[any, 2, any]`, although link 2 still has source 2. Subsequent usage
operations can panic. The unit store reports the expected source count of one.

The regression suite constructs each full or partial self-reference from a new
uninitialized link so it tests self-exclusion independently of that update bug.
The probe preserves the separate failing scenario for further investigation:

```sh
cargo build -p doublets
rustc --edition=2021 experiments/issue-63/self-reference-probe.rs \
  -L dependency=target/debug/deps \
  --extern doublets=target/debug/libdoublets.rlib \
  -o /tmp/issue-63-self-reference-probe
/tmp/issue-63-self-reference-probe
```
