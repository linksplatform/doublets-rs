# Case Study: CI/CD Publishing Failed (Issue #49)

## Timeline of Events

1. **2026-04-14T15:13:22Z** - Push to `main` branch triggers CI/CD pipeline (run [#24406907038](https://github.com/linksplatform/doublets-rs/actions/runs/24406907038))
2. **2026-04-14T15:13:22Z - 15:14:10Z** - Jobs `Detect Changes`, `Lint and Format Check`, `Code Coverage`, `Test` (all platforms), `Build Package` pass successfully
3. **2026-04-14T15:14:10Z** - `Auto Release` job starts, passes setup steps (checkout, rust toolchain, rust-script, git config, bump type detection)
4. **2026-04-14T15:14:34Z** - **FAILURE** at step "Check if version already released or no fragments"
   - Error: `Could not find name in ./Cargo.toml`
   - Exit code: 1

## Root Cause Analysis

### Primary Root Cause: Workspace Cargo.toml incompatibility

The repository uses a **Cargo workspace** structure:

```
./Cargo.toml          # [workspace] only - NO [package] section
./doublets/Cargo.toml # [package] with name="doublets", version="0.1.0-pre+beta.15"
./integration/Cargo.toml # [package] with publish=false
```

All CI/CD release scripts (`check-release-needed.rs`, `version-and-commit.rs`, `publish-crate.rs`, `get-version.rs`, `bump-version.rs`, `collect-changelog.rs`, `create-github-release.rs`, `check-version-modification.rs`) were designed for **single-crate repositories** where the root `Cargo.toml` contains both `[package]` and `[workspace]` sections, or is purely a `[package]`.

The scripts use regex `^name\s*=\s*"([^"]+)"` to extract the crate name from root `Cargo.toml`, which fails when the root is a workspace-only manifest.

### Secondary Root Cause: Pre-release version regex in bump-version.rs

The `bump-version.rs` script uses regex `^version\s*=\s*"(\d+)\.(\d+)\.(\d+)"` which requires the version string to end immediately after the patch number. The actual version `0.1.0-pre+beta.15` has a pre-release suffix, causing the regex to not match.

## Affected Components

| Script | Reads Name | Reads Version | Writes Version | Impact |
|--------|-----------|---------------|----------------|--------|
| check-release-needed.rs | Yes | Yes | No | **Blocks all releases** |
| version-and-commit.rs | Yes | Yes | Yes | Blocks version bumping |
| publish-crate.rs | Yes | Yes | No | Blocks crates.io publishing |
| get-version.rs | No | Yes | No | Blocks version reporting |
| bump-version.rs | No | Yes | Yes | Blocks version bumping |
| collect-changelog.rs | No | Yes | No | Blocks changelog generation |
| create-github-release.rs | Yes (optional) | No | No | Missing badges in releases |
| check-version-modification.rs | No | No | No | Checks wrong file for diffs |
| rust-paths.rs | No | No | No | Missing workspace info |

## Solution

Added workspace-aware `Cargo.toml` resolution to all affected scripts:

1. **Detection**: Check if root `Cargo.toml` contains `[workspace]` section
2. **Member parsing**: Extract `members = [...]` list from workspace manifest
3. **Publishable member selection**: Find the first member whose `Cargo.toml` does NOT contain `publish = false`
4. **Path resolution**: Return the publishable member's `Cargo.toml` path instead of root

Additionally:
- Fixed `bump-version.rs` regex to handle pre-release version suffixes
- Added `-p <package>` flag to `cargo publish` command in `publish-crate.rs` for workspace repos

## Verification

All scripts tested locally against the actual workspace structure:

```
$ rust-script scripts/check-release-needed.rs
Detected workspace Cargo.toml, searching for publishable member...
Using publishable workspace member: doublets (doublets/Cargo.toml)
Crate: doublets, Version: 0.1.0-pre+beta.15, Published on crates.io: true
```

## Template Repository Impact

- **Rust template** (`link-foundation/rust-ai-driven-development-pipeline-template`): Same bug confirmed. All 5 release scripts lack workspace awareness. Issue filed: https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/36
- **JS template** (`link-foundation/js-ai-driven-development-pipeline-template`): Not affected. npm/pnpm workspace root `package.json` files still contain `name` and `version`, and the template uses `@changesets/cli` with native workspace support.

## CI Logs

- Full run log: `ci-logs/run-24406907038-full.log`
- Failed step log: `ci-logs/run-24406907038-failed.log`
