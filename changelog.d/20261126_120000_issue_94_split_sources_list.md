---
bump: minor
---

### Added
- Implement the internal sources linked list of `split::Store`: updates,
  creation of referenced IDs, and releases now attach and detach links through
  it instead of panicking at `todo!()`. A trailing const generic `USE_LIST`
  selects it, and `split::TreeStore` names the store that keeps the
  size-balanced sources tree.

### Changed
- `split::Store` now uses the linked list by default, like `useLinkedList: true`
  of the C# `SplitMemoryLinks`. `[any, source, any]` queries return links in
  insertion order instead of target order. The two modes keep different index
  metadata: open split index files written by 0.22 or earlier with
  `split::TreeStore`.

### Fixed
- In list mode, search a pair with an internal source and an absent target in
  the external targets tree instead of the external sources tree, and keep a
  live `(source, null)` link from looking like a free slot.
