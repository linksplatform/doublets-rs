# Issue 66: updating a missing link through the uniqueness resolvers

## Research and checklist

I read issue 66 and its single comment, and the Data.Doublets PR 527 review that
the comment links. PR 89 had no review comments or reviews. There are no required
sub-issues. The comment changes how to read the suggested fix. A uniqueness
resolver must not silently gain a compulsory existence check. A caller who wants
missing-address errors selects them explicitly, and can still omit the check when
the operations are known to be valid.

- [x] Reproduce the report on both memory layouts
      (`*_adds_no_existence_check` in `doublets/tests/uniqueness_missing_update.rs`).
- [x] Keep the resolvers free of a hidden `try_get_link(old)?`. Pin that omitting
      the policy omits its work: the store is unchanged and the stored pair is
      reported.
- [x] Pin the explicit error route. `.with_inner_reference_existence_validation()`
      composed outside `Resolve`, `CascadeResolve` and the automatic stack returns
      `Error::NotExists(99)` for both repro updates, with no write and no callback.
- [x] Pin that placement matters. Inside the resolver, the validator is never
      reached for an existing pair.
- [x] Pin the per-call route: `try_get_link(index)?` before the update.
- [x] Document the contract in the resolver rustdoc (with a doctest), the
      automatic-stack builder, `docs/mutation.md` (doc-tested guide) and the README.

## Root cause

`UniquenessResolver::update_links` and
`CascadeUniquenessAndUsagesResolver::update_links` look up the new
`(source, target)` pair first. When the pair is stored, they call
`resolve_address_change_conflict(old, new)`. That function deletes `old` only
`if old != new && links.exist(old)` and then reports the survivor. This is the
C# `LinksUniquenessResolver.ResolveAddressChangeConflict` logic. So with a missing
`old` nothing reaches the store's checked route, and the update returns `Ok`.
When the pair is new, the update is forwarded, and the store's `CheckedMutation`
returns `Error::NotExists`. That explains the asymmetry in the report.

## Decision

The behavior follows from the layering. Each decorator verifies only its own
contract. Existence is the `InnerReferenceExistenceValidator` error policy, or an
explicit lookup. A compulsory lookup in the resolver would make every resolved
update pay for a check that callers with valid addresses do not need. The
resolver behavior is unchanged. The fix documents the contract and tests both
explicit routes.

## Verification

```console
$ bash experiments/issue-66/verify.sh
```
