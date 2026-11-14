### Fixed
- Return the complete allocation after memory growth so unit and split stores retain their headers, earlier links, and full capacity with heap and file-mapped backends.
- Add regressions for initial capacity, repeated growth, preserved link queries, and backends that return only the grown region.
- Count empty endpoint indexes as zero instead of reading free-list header fields, with a regression for deleted-slot reuse exposed by the updated random test sequence.

### Changed
- Update all Rust dependencies to their latest stable releases, including development tools, embedded release scripts, and excluded legacy crate manifests; refresh the workspace lockfile and update affected APIs.
- Update GitHub Actions and pre-commit hooks to their current stable major releases.
