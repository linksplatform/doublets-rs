---
bump: patch
---

### Fixed
- Keep new reservations sparse for file-mapped unit and split stores using native unsigned link references. File growth no longer writes zeroes across space already zero-filled by the filesystem.
