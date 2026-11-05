---
bump: patch
---

### Fixed
- Report nested resets to `(0, 0)` before cascaded deletions in the automatic uniqueness and usages resolution stack, matching C# callback order while preserving cycle termination.
- Correct decorator ordering documentation and synchronize the README dependency example with the released crate version.
