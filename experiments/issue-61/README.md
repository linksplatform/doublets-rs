# Issue 61 experiments

## Split-store self-reference reproduction

The transaction address-reuse test exposed a split-store indexing bug. Updating
`a` from `(a, a)` to `(b, a)` classified `a` as external after detaching its old
index nodes. The internal target count became zero instead of one. Clearing that
link later detached an absent internal node and panicked with an overflow.

`split_reuse.rs` traces a finite five-link graph, address reuse, index nodes, and
each content reset. Run it with:

```bash
cargo build -p doublets --lib
rustc --edition=2021 experiments/issue-61/split_reuse.rs \
  --extern doublets=target/debug/libdoublets.rlib \
  -L dependency=target/debug/deps -o /tmp/issue-61-split-reuse
RUST_BACKTRACE=full /tmp/issue-61-split-reuse > /tmp/issue-61-split-reuse.log 2>&1
```

The minimal automated reproduction is
`doublets/tests/split_self_reference.rs`. It failed on the incorrect count before
the fix. Classifying destinations before detaching indexes fixes both the count
and subsequent reset.

## Transaction recovery constraints

Store callbacks run after mutation, so callback-only logging leaves a crash gap.
The generic store interface also lacks a flush hook and exact-address allocation.
The decorator therefore persists a before snapshot at begin and an after snapshot
at commit, alongside reversible transitions. Recovery can undo a mutation with no
callback record and redo a commit whose backend writes did not persist. Restoring
addresses uses raw sequential allocation, preserving holes and surviving pair swaps
by clearing indexed contents before reattaching them.

`doublets/tests/transaction_recovery.rs` tests these gaps and launches subprocesses
that exit without running destructors. Recovery assumes exclusive ownership of a
matching, structurally readable store and journal. Retention, version control, and
triggers remain separate layers.

## Local verification

Run `bash experiments/issue-61/verify.sh /tmp/doublets-issue-61-checks` to run the
workspace, documentation, release, packaging, example, and contribution checks.
Each check writes its own log to that directory. The test suite also covers both
store backends, policies around transaction handles, unwind rollback, injected
journal/restoration failures, torn records, corruption, and address narrowing.
