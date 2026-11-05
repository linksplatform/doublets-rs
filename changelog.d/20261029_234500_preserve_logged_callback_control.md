---
bump: patch
---

### Fixed
- Preserve callback `Flow::Break` in `create_link_with` even when an inner layer
  returns `Continue`, while completing link initialization.
- Return callback breaks from logged mutations without skipping completed mutation
  log entries or overriding store and logging errors.
