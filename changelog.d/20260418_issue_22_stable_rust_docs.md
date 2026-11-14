### Fixed
- Completed stable Rust support across standalone decorator and FFI packages, replacing nightly-only language features and outdated APIs while preserving decorator names and C exports.
- Fixed FFI handle ownership and callback continuation during the migration; added CRUD, persistence, constants, and error regressions for all four exported integer widths.
- Added stable compilation of every package and dependency graph to the Linux, macOS, and Windows CI matrix; updated FFI and contributor toolchain guidance.
- Keep README dependency snippets and path dependency versions in tracked lockfiles synchronized during automatic releases so stable checks remain reproducible after a version bump.
