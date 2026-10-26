---
bump: patch
---

### Fixed
- Keep split-store source and target references searchable when their IDs are
  created later, including reuse of deleted IDs. Migrate existing external index
  entries directly into the new ID's internal indexes before creation callbacks.
- Count concrete split-store pairs with absent sources by both endpoints instead
  of counting every usage of the source.
