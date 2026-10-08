# Issue #63 investigation

`doublets/tests/usages.rs` reproduces overlapping usages on both backends. Before
the fix, the minimal four-link fixture returned `2` usages and `[4, 4]`, and usage
deletion returned `NotExists(4)`. The three-link query fixture returned
`NotExists(1)` after deleting its matches.

Run the automated regression suite with:

```sh
cargo test -p doublets --test usages --all-features
```

## Split-store update probe

The bounded `self-reference-probe.rs` uses at most four links. It also exposed a
pre-existing problem on the original branch, outside the overlap fix: changing
`(2: 2 2)` to `(2: 2 1)` after deleting a usage caused the split store's source
index to report zero
matches for `[any, 2, any]`, although link 2 still has source 2. Subsequent usage
operations could panic. The unit store reported the expected source count of one.

[PR #71](https://github.com/linksplatform/doublets-rs/pull/71) independently fixed
this by classifying self-references before detaching
the split-store indexes. Main, including that fix, was merged into this branch
during validation. The probe now verifies the correct counts on both backends.

The regression suite constructs each full or partial self-reference from a new
uninitialized link so it tests self-exclusion independently of that update bug.
The probe preserves that scenario for further verification:

```sh
cargo build -p doublets
rustc --edition=2021 experiments/issue-63/self-reference-probe.rs \
  -L dependency=target/debug/deps \
  --extern doublets=target/debug/libdoublets.rlib \
  -o /tmp/issue-63-self-reference-probe
/tmp/issue-63-self-reference-probe
```
