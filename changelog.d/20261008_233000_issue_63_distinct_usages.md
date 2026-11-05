---
bump: patch
---

### Fixed
- Count and list each usage once when a link references the same value as both source and target, while excluding the referenced link itself.
- Delete overlapping usage and query matches once, avoiding spurious `NotExists` errors and duplicate deletion callbacks.
