---
bump: minor
---

### Added
- Generic `TransactionsDecorator` with commit/rollback handles, rollback on drop, reversible transitions, memory and durable checksummed file journals, and recovery of interrupted transactions.
- Transaction builder, runnable example, optional diagnostics, and tests for both store backends, I/O failures, journal corruption and subprocess crash recovery.

### Fixed
- Classify split-store self references before detaching index nodes, preventing an update from moving a live reference to the external index and later panicking during rollback.
