# Issue 81 and PR 85 review plan

Objective: fully implement and verify issue 81 and all PR feedback in PR 85, without merging it.

- [x] Read complete issue and PR description; verify the prepared branch and clean starting tree.
- [x] Read all issue/sub-issue, PR conversation, inline review, and review-submission endpoints; inventory requirements and existing evidence.
- [x] Inspect contributing guidance, all Rust source-tree tests, module visibility, existing test organization, and recent related changes.
- [x] Preserve baseline test inventory and reproduce the review concern before changing files.
- [x] Move every Rust source-tree test to the corresponding tests directory, preserving coverage without widening production APIs solely for tests; collapse single-file module directories where applicable.
- [x] Update contributor guidance and any affected verification/docs references.
- [ ] Merge latest main into the authorized branch without rewriting history.
- [ ] Run focused regression tests, all local tests, formatting, strict Clippy, package/file-size/documentation checks with one Cargo job and reduced debug information; save large logs under ci-logs/.
- [ ] Review complete PR diff for original release/reset requirements, callback/index/reuse safety, test preservation, and unintended removals.
- [ ] Commit useful atomic work, push only issue-81-c1387393dd57, and update PR title/description with reproduction, verification, and Closes #81.
- [ ] List latest CI runs with timestamps and exact SHA; preserve and inspect logs for non-passing runs, fix failures, and wait for all checks.
- [ ] Confirm latest main included, no remaining source-tree tests, clean worktree, all criteria verified, PR ready and unmerged.

## Acceptance inventory

Original issue: minimal release with explicit live/detached/incoming-index preconditions; optional endpoint reset layer; safe checked access; valid callbacks/results; necessary structural index bookkeeping; no redundant reset; both layouts, self references, incoming usages, reuse, and callbacks covered. Existing implementation and evidence are documented in README.md and docs/deletion.md; reverify after test relocation.

New feedback: https://github.com/linksplatform/doublets-rs/pull/85#issuecomment-6523873049 requires no Rust tests under src, all tests under tests, and collapsing single-file folders when relocation leaves them.
