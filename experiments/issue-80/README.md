# Issue 80: compulsory mutation checks vs. an explicitly unchecked API

## Research and checklist

I read issue 80 and its comments (there are none), all three PR 88 feedback
endpoints (all empty), the review of Data.Doublets PR 527 that the issue links,
and the related issue 81 and PR 85 release work. There are no required
sub-issues.

- [x] Specify the minimal mutation preconditions; expose an `unsafe` primitive
      (`mem::UncheckedMutation::update_unchecked`, plus the existing
      `Release::release_unchecked`).
- [x] Keep safe APIs memory-safe. Move the checks into an explicit adapter
      (`decorators::CheckedMutation`), and document how to omit them
      ([`docs/mutation.md`](../../docs/mutation.md)).
- [x] Keep the error, ignore-missing and swallow policies distinct
      (`NonExistentDeletionResolver`, a port of C#
      `LinksNonExistentDeletionResolver`, which never swallows errors).
- [x] Test safe rejection without mutation, valid unchecked calls, and that
      omitting validation omits its work. Unchecked APIs are tested only with
      valid live addresses.

## Baseline

On main (`694ca8f`), both backends embed `try_get_link(index)?` in
`update_links`. `delete_links` calls the crate-private `delete_with_reset`, which
looks the link up four times: its own `try_get_link`, then the reset resolver's
`get_link`, then the reset `update`'s `try_get_link`, then `CheckedRelease`'s
`try_get_link`. The public surface had no update primitive, and omitting
`InnerReferenceExistenceValidator` did not omit the backend check.

## Design

- **Mechanism:** `unsafe fn update_unchecked(index, source, target, handler)`
  requires only that `index` is live and that the indexes are consistent, as
  the store API maintains them. It detaches the old outgoing memberships,
  writes the endpoints, attaches the new ones and reports one event. Deletion
  is the caller's composition of an optional `update_unchecked(index, 0, 0)` and
  `release_unchecked`. Reset is a policy, so there is no third primitive.
- **Checked convenience:** `CheckedMutation<'_, T, L>` is a borrowing view, like
  `CheckedRelease`. It does one lookup and returns `Error::NotExists` before
  any write or callback. The stores' safe `update_links` and `delete_links` are
  one-line delegations to it. A checked delete now does one lookup instead of
  four. Short query/change slices read missing parts as null instead of
  panicking.
- **Policies:** the error policy is the checked route and
  `InnerReferenceExistenceValidator`. The ignore policy is
  `NonExistentDeletionResolver`: a missing requested address gives
  `Ok(Flow::Continue)`, and a live one is forwarded so that every inner error
  propagates. `NoExceptionsDecorator` remains the explicit swallow-all choice.
- **Rejected:** an unsafe constructor returning a view that implements the
  *safe* `Links` trait without checks. Safe callers could then reach undefined
  behavior through the view, which is unsound.

## Requirement-to-evidence map

| Requirement | Implementation | Verification |
| --- | --- | --- |
| Minimal preconditions + `unsafe` primitive | `doublets/src/mem/mutation.rs`, both stores' `UncheckedMutation` impls | `valid_unchecked_updates` (unit + split); trait doctest |
| Safe APIs stay safe; checks are an explicit adapter | `doublets/src/decorators/mutation.rs`; stores delegate | `safe_mutation_rejects_missing_addresses`, `checked_mutation_routes` |
| Documented route to omit checks | `docs/mutation.md` (doctested), `examples/unchecked_mutation.rs` | `cargo test --doc Guide`, example run |
| Error vs. ignore policies distinct, no swallowing | `NonExistentDeletionResolver` in `decorators/existence.rs` | `missing_deletion_is_ignored_only_when_selected`, `missing_deletion_resolver_propagates_unrelated_errors` |
| Omitting validation omits its work | test-only `link_lookups` counter in both stores | `unchecked_mutation_omits_validation_work` in `tests/internal/{unit,split}.rs` |

The missing addresses in these tests are null, a released free-list slot, an
address beyond the allocated range, `any` and `u32::MAX`. They go only to safe
APIs. Each rejection leaves the count, the links and the free list unchanged and
makes no callback.

While writing the tests I found that giving a link the same endpoints as another
live link, which is a duplicate when uniqueness is not enforced, can corrupt the
size-balanced trees on later updates. This is not related to issue 80: the
detach and attach logic is unchanged from main. The tests use distinct
endpoints, as the uniqueness decorators guarantee for safe stacks.

Run `bash experiments/issue-80/verify.sh`. It writes its logs to
`target/issue-80-logs/`.
