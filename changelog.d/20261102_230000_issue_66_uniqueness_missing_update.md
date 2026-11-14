---
bump: patch
---

### Changed

- Document that `UniquenessResolver`, `CascadeUniquenessAndUsagesResolver` and `with_automatic_uniqueness_and_usages_resolution()` do not check that an updated address exists (issue #66). Updating a missing link to an already stored pair reports that link without changes. Get `Error::NotExists` by explicitly composing `.with_inner_reference_existence_validation()` outside the resolver or by calling `try_get_link(index)?` per call. The resolvers deliberately do not gain a compulsory existence check.
- Add regression tests for both explicit routes on both memory layouts, and a test showing that the resolvers add no hidden lookup.
