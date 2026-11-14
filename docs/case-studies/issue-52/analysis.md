# Issue 52: Benchmark Badge Shows Failing

Date: 2026-04-18 UTC

## Requirements

- Download logs and related data into `docs/case-studies/issue-52`.
- Reconstruct the sequence of events for the failing Benchmark badge.
- Identify root causes and solution options.
- Search online for relevant facts about GitHub Actions workflow badges.
- Fix the repository so documentation no longer shows the stale failing Benchmark badge.
- Add a regression check where practical.

## Evidence Collected

- `raw/issue-52.json` and `raw/issue-52-comments.json`: issue text and comments.
- `raw/actions-workflows.json`: active GitHub Actions workflows.
- `raw/actions-runs-first-page.json`: recent Actions run metadata.
- `raw/benchmark-workflow-before-removal.yml`: deleted Benchmark workflow from `8def08d^`.
- `raw/release-workflow-current.yml`: current `CI/CD Pipeline` workflow.
- `badges/benchmark-badge.svg`: current rendered legacy Benchmark badge.
- `badges/benchmark-main-badge.svg`: rendered `Benchmark` badge with `?branch=main`.
- `badges/current-pipeline-badge.svg`: rendered current pipeline badge with `?branch=main`.
- `ci-logs/benchmark-24388559834-failure.log` and `ci-logs/benchmark-24388669998-failure.log`: latest failing Benchmark workflow logs.
- `ci-logs/benchmark-24337694411-success.log`: recent successful Benchmark workflow log.
- `ci-logs/cicd-24411195348-main-success.log`: current main CI/CD success log.
- `raw/pre-fix-badge-check.log`: reproducer showing the saved pre-fix README fails the badge check.
- `raw/readme-badge-check.log`: post-fix badge check output.
- `raw/cargo-*.log` and `raw/check-file-size.log`: local validation logs.

The GitHub Actions artifact API returned empty artifact lists for the inspected runs, saved as `raw/artifacts-*.json`.

## Online Reference

GitHub Actions documentation recommends workflow status badge URLs based on the workflow file path:

`https://github.com/OWNER/REPOSITORY/actions/workflows/WORKFLOW-FILE/badge.svg`

It also documents the `branch` query parameter for showing a specific branch status. Source: <https://docs.github.com/en/actions/monitoring-and-troubleshooting-workflows/monitoring-workflows/adding-a-workflow-status-badge>

## Timeline

- 2025-12-28: `Benchmark` workflow passed on `main` in run `20549074956`.
- 2026-04-13 10:06 UTC: `Benchmark` workflow passed on PR branch `issue-47-33c2f118f59c` in run `24337694411`.
- 2026-04-14 08:18 UTC and 08:20 UTC: `Benchmark` workflow failed in runs `24388559834` and `24388669998`.
- 2026-04-14 08:28 UTC: commit `8def08d` removed `.github/workflows/benchmark.yml` and `.github/workflows/ci.yml`, replacing them with `.github/workflows/release.yml`.
- 2026-04-14 16:38 UTC: current `CI/CD Pipeline` passed on `main` in run `24411195348`.
- 2026-04-18 UTC: README still used `https://github.com/linksplatform/doublets-rs/workflows/Benchmark/badge.svg`, which rendered as `Benchmark - failing`.

## Findings

The active workflow list contains `CI/CD Pipeline` at `.github/workflows/release.yml`; it does not contain an active `Benchmark` workflow. The README still pointed at both legacy workflow-name badge URLs:

- `https://github.com/linksplatform/doublets-rs/workflows/CI/badge.svg`
- `https://github.com/linksplatform/doublets-rs/workflows/Benchmark/badge.svg`

The downloaded badge SVGs show the externally visible state:

- `badges/benchmark-badge.svg`: `<title>Benchmark - failing</title>`
- `badges/benchmark-main-badge.svg`: `Not Found`
- `badges/current-pipeline-badge.svg`: `<title>CI/CD Pipeline - passing</title>`

The latest failing Benchmark logs show that the old workflow attempted to install `nightly-2022-08-22`, but the checked-out `rust-toolchain.toml` selected stable Rust. The build then used `rustc 1.94.1` and failed on old dependencies that still used nightly-only features:

- `benchmark-24388669998-failure.log:557`: `rustc 1.94.1`
- `benchmark-24388669998-failure.log:659`: `error[E0554]: #![feature] may not be used on the stable release channel`
- `benchmark-24388669998-failure.log:672`: `error: could not compile platform-num`

That failure explains why the last visible legacy Benchmark badge state became red. The current repository no longer has that Benchmark workflow, so keeping its badge in README is now misleading.

## Root Causes

1. The workflow migration in commit `8def08d` removed the dedicated `Benchmark` workflow but did not update README badges.
2. The legacy badge used the workflow name URL instead of the currently recommended workflow file URL.
3. The legacy `Benchmark` badge was not scoped to `main`, so deleted or inactive workflow history could surface stale PR-run status instead of the current release pipeline status.
4. The old failing Benchmark runs were caused by a mismatch between an old nightly-oriented workflow and the repository's stable Rust migration. This is no longer the active CI path.

No external issue was filed because the actionable defect is in this repository's documentation and workflow migration, not in an external project.

## Solution Options

1. Replace README workflow badges with the current `CI/CD Pipeline` badge using `.github/workflows/release.yml` and `?branch=main`.
2. Recreate a dedicated stable-compatible Benchmark workflow, then add a branch-scoped Benchmark badge after that workflow passes on `main`.
3. Remove all GitHub Actions badges and rely on the Actions tab.

Option 1 is the lowest-risk fix for this issue because it removes the misleading deleted-workflow badge immediately and points readers to the workflow that currently protects `main`.

Option 2 is useful future work if maintainers want a separate benchmark signal again. It should be done as a separate change because adding a benchmark workflow changes CI runtime and semantics.

## Implemented Fix

- Replaced the stale `CI` and `Benchmark` badges in `README.md` with one active `CI/CD Pipeline` badge:
  `https://github.com/linksplatform/doublets-rs/actions/workflows/release.yml/badge.svg?branch=main`
- Added `scripts/check-readme-badges.rs`.
- Wired the badge check into the CI/CD lint job.

The regression check fails if README documentation reintroduces:

- `https://github.com/linksplatform/doublets-rs/workflows/.../badge.svg`
- `actions?query=workflow%3A...` legacy workflow links
- `actions/workflows/.../badge.svg` URLs that are missing `?branch=main`
- `actions/workflows/.../badge.svg` URLs that reference missing workflow files

By default it scans markdown files outside `docs/case-studies`, because case studies intentionally preserve historical broken URLs as evidence.

## Verification

- `rustc scripts/check-readme-badges.rs -o /tmp/check-readme-badges`
- `/tmp/check-readme-badges docs/case-studies/issue-52/raw/readme-before-fix.md` failed as expected; see `raw/pre-fix-badge-check.log`.
- `/tmp/check-readme-badges` passed for the fixed README.
- `cargo fmt --all -- --check` passed; see `raw/cargo-fmt.log`.
- `rust-script scripts/check-readme-badges.rs` passed; see `raw/readme-badge-check.log`.
- `rust-script scripts/check-file-size.rs` passed; see `raw/check-file-size.log`.
- `cargo clippy --all-targets --all-features` passed; see `raw/cargo-clippy.log`.
- `cargo test --all-features --verbose` passed; see `raw/cargo-test.log`.
- `cargo test --doc --verbose` passed; see `raw/cargo-doc-test.log`.
