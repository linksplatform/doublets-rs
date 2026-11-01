# Contributing to doublets-rs

Thank you for your interest in contributing! This document provides guidelines and instructions for contributing to this project.

## Development Setup

1. **Fork and clone the repository**

   ```bash
   git clone https://github.com/YOUR-USERNAME/doublets-rs.git
   cd doublets-rs
   ```

2. **Install Rust**

   Install Rust using rustup (if not already installed):

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

   The project uses the stable toolchain configured in `rust-toolchain.toml`:

   ```toml
   [toolchain]
   channel = "stable"
   ```

   The `doublets` crate declares its minimum supported Rust version in
   `doublets/Cargo.toml`:

   ```toml
   rust-version = "1.85"
   ```

3. **Install development tools**

   ```bash
   rustup component add rustfmt clippy
   ```

4. **Install pre-commit hooks** (optional but recommended)

   ```bash
   pip install pre-commit
   pre-commit install
   ```

5. **Build the project**

   ```bash
   cargo build
   ```

## Development Workflow

1. **Create a feature branch**

   ```bash
   git checkout -b feature/my-feature
   ```

2. **Make your changes**

   - Write code following the project's style guidelines
   - Add tests for any new functionality
   - Update documentation as needed

3. **Run quality checks**

   ```bash
   # Format code
   cargo fmt

   # Run Clippy lints
   cargo clippy --all --tests --all-features

   # Check file sizes
   rust-script scripts/check-file-size.rs

   # Check README badges and toolchain guidance
   rust-script scripts/check-readme-badges.rs
   rust-script scripts/check-toolchain-docs.rs

   # Keep Rust test implementations in tests/
   bash experiments/issue-81/check-test-layout.sh

   # Run all checks together
   cargo fmt --all -- --check && cargo clippy --all-targets --all-features && rust-script scripts/check-file-size.rs && rust-script scripts/check-readme-badges.rs && rust-script scripts/check-toolchain-docs.rs
   ```

4. **Run tests**

   ```bash
   # Run all tests
   cargo test

   # Run tests with verbose output
   cargo test --verbose

   # Run doc tests
   cargo test --doc

   # Run a specific test
   cargo test test_name
   ```

   Also verify packages that are built independently of the main workspace:

   ```bash
   bash experiments/issue-22/check-packages.sh
   cargo +stable test --manifest-path doublets-ffi/Cargo.toml --locked
   ```

   Use the latest stable toolchain for development and FFI builds. Some test and
   procedural macro dependencies require newer Rust than the core library's MSRV.

5. **Add a changelog fragment**

   For any user-facing changes, create a changelog fragment:

   ```bash
   # Create a new file in changelog.d/
   # Format: YYYYMMDD_HHMMSS_description.md
   touch changelog.d/$(date +%Y%m%d_%H%M%S)_my_change.md
   ```

   Edit the file to document your changes:

   ```markdown
   ### Added
   - Description of new feature

   ### Fixed
   - Description of bug fix
   ```

   **Why fragments?** This prevents merge conflicts in CHANGELOG.md when multiple PRs are open simultaneously.

6. **Commit your changes**

   ```bash
   git add .
   git commit -m "feat: add new feature"
   ```

   Pre-commit hooks will automatically run and check your code.

7. **Push and create a Pull Request**

   ```bash
   git push origin feature/my-feature
   ```

   Then create a Pull Request on GitHub.

## Code Style Guidelines

This project uses:

- **rustfmt** for code formatting
- **Clippy** for linting with pedantic and nursery lints enabled
- **cargo test** for testing

### Code Standards

- Follow Rust idioms and best practices
- Use documentation comments (`///`) for all public APIs
- Write tests for all new functionality
- Keep functions focused and reasonably sized
- Keep files under 1000 lines
- Use meaningful variable and function names

### Documentation Format

Use Rust documentation comments:

```rust
/// Brief description of the function.
///
/// Longer description if needed.
///
/// # Arguments
///
/// * `arg1` - Description of arg1
/// * `arg2` - Description of arg2
///
/// # Returns
///
/// Description of return value
///
/// # Errors
///
/// Description of when errors are returned
///
/// # Examples
///
/// ```
/// use doublets::example_function;
/// let result = example_function(1, 2);
/// assert_eq!(result, 3);
/// ```
pub fn example_function(arg1: i32, arg2: i32) -> i32 {
    arg1 + arg2
}
```

## Testing Guidelines

- Write tests for all new features
- Maintain or improve test coverage
- Use descriptive test names
- Keep all Rust test implementations in the package's `tests/` directory.
- Prefer integration tests that exercise the public API.
- Private-state tests live in `doublets/tests/internal/` and are included with
  `#[cfg(test)]` and `#[path = "..."]` from the module they exercise. This keeps
  private implementation details private while preserving internal coverage.
- Standalone Rust script tests follow the same rule, using `scripts/tests/`.
- Use a single file instead of a new directory containing only one file.

Example integration test in `doublets/tests/creation.rs`:

```rust
use doublets::{unit, Doublets, Link};
use mem::Global;

#[test]
fn creates_a_point() {
    let mut store = unit::Store::<u32, _>::new(Global::new()).unwrap();
    let index = store.create_point().unwrap();
    assert_eq!(store.get_link(index), Some(Link::point(index)));
}
```

## Pull Request Process

1. Ensure all tests pass locally
2. Update documentation if needed
3. Add a changelog fragment (see step 5 in Development Workflow)
4. Ensure the PR description clearly describes the changes
5. Link any related issues in the PR description
6. Wait for CI checks to pass
7. Address any review feedback

## Changelog Management

This project uses a fragment-based changelog system similar to [Scriv](https://scriv.readthedocs.io/) (Python) and [Changesets](https://github.com/changesets/changesets) (JavaScript).

### Creating a Fragment

```bash
# Create a new fragment with timestamp
touch changelog.d/$(date +%Y%m%d_%H%M%S)_description.md
```

### Fragment Categories

Use these categories in your fragments:

- **Added**: New features
- **Changed**: Changes to existing functionality
- **Deprecated**: Features that will be removed in future
- **Removed**: Features that were removed
- **Fixed**: Bug fixes
- **Security**: Security-related changes

### During Release

Fragments are automatically collected into CHANGELOG.md during the release process. The release workflow:

1. Collects all fragments
2. Updates CHANGELOG.md with the new version entry
3. Removes processed fragment files
4. Bumps the version in Cargo.toml
5. Creates a git tag and GitHub release

## Project Structure

```
.
├── .github/workflows/    # GitHub Actions CI/CD
├── changelog.d/          # Changelog fragments
│   └── README.md         # Fragment instructions
├── ci/                   # CI scripts
├── dev-deps/             # Development dependencies (git submodules)
├── doublets/             # Main doublets library
│   ├── src/              # Source code
│   │   └── decorators/   # Policy wrappers and transactions
│   ├── tests/            # Integration tests
│   └── benches/          # Benchmarks
├── doublets-ffi/         # Stable Rust FFI bindings and procedural macros
├── integration/          # Integration tests
├── scripts/              # Utility scripts
├── .gitignore            # Git ignore patterns
├── .pre-commit-config.yaml  # Pre-commit hooks
├── Cargo.toml            # Workspace configuration
├── CHANGELOG.md          # Project changelog
├── CONTRIBUTING.md       # This file
├── LICENSE               # Unlicense (public domain)
├── README.md             # Project README
└── rust-toolchain.toml   # Rust toolchain configuration
```

## Release Process

This project uses semantic versioning (MAJOR.MINOR.PATCH):

- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

Releases are managed through GitHub releases. To trigger a release:

1. Manually trigger the release workflow with a version bump type
2. Or: Update the version in Cargo.toml and push to main

## Getting Help

- Open an issue for bugs or feature requests
- Use discussions for questions and general help
- Check existing issues and PRs before creating new ones

## Code of Conduct

- Be respectful and inclusive
- Provide constructive feedback
- Focus on what is best for the community
- Show empathy towards other community members

Thank you for contributing!
