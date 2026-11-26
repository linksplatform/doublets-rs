# Issue #94: internal sources linked list of `split::Store`

- `probe.rs`: a stand-alone example program. Links with one internal source are listed in
  insertion order, as with `useLinkedList: true` in the C# `SplitMemoryLinks`.
- `reproduce-before.sh [commit]`: runs the probe on a commit before the fix
  (default `34e3a23`, v0.22.0) in a temporary worktree.
  1. With the default store, the probe assertion fails because the sources tree orders
     links by target: `left: [1, 6, 5, 7]`, `right: [1, 5, 6, 7]`.
  2. With `USE_LIST` set to `true`, the first update panics with
     `not yet implemented` (`todo!()` in `store.rs`).

  The output of that run is saved in `logs/`.
- `verify.sh`: runs the regression tests and the probe on the current tree.
  It runs `tests/split_sources_list.rs`, `tests/split_forward_reference.rs`,
  and the internal stored-size checks.
