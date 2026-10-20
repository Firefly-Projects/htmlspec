# Proving the tests can fail

**Date:** 2026-10-14
**Tree:** this change's branch at `a701a98`, release 0.30.0, and then the two edits this record describes.

Each repair is undone in turn, by the smallest edit that removes it, and the whole workspace's tests
are run with `nix develop -c cargo test --locked --workspace --no-fail-fast`. The edit is reverted
before the next one. A repair no test notices is a repair the suite does not protect. A script made
each edit, ran the suite, and wrote the original bytes back whatever happened.

## The first run

| Repair undone | Edit | Tests that failed |
|---|---|---|
| The rewrite: relocation re-expresses nothing | `relocate.rs`: no value is ever re-expressed | 7: `archival_keeps_every_reference_reaching_what_it_reached` and six `relocate::tests` |
| The promotion: archival leaves each rewritten document staged after the move | `commands.rs`, `perform`: the rename after the move is skipped | 1: `archival_keeps_every_reference_reaching_what_it_reached` |
| Recovery's completion: a moved change's staged documents are left staged | `commands.rs`, `recover_publication`: the rename is skipped | 1: `an_interruption_after_the_move_puts_the_staged_documents_in_place` |
| Recovery's rollback: an unmoved change's staged documents are kept | `commands.rs`, `recover_publication`: the removal is skipped | 1: `an_interruption_before_the_move_removes_the_staged_documents` |
| The leftover check: a staged document already in the change is not refused | `commands.rs`, `perform`: the check never fires | **none** |
| The two places spelled as given: archival relates them without resolving links | `commands.rs`, `rewritten_documents`: `std::path::absolute` for `canonical` | 1: `archival_keeps_every_reference_reaching_what_it_reached` |
| The path check: a store seen from a document still resolves by address alone | `store.rs`, `seen_through`: no route is kept | 5: `a_reference_whose_path_misses_the_store_is_unresolved`, two `rules::assets::tests`, two `store::tests` |

The six `relocate::tests` that failed under the first edit are the ones that assert a rewrite; the six
that assert a value is left alone pass under it, as they did before the fix.

**The spelling mutant bears out design decision 1's claim.** With the two places related as spelled,
the archive test fails. The suite's scratch projects sit under the system's temporary directory, in
`/var/folders/`, and on macOS `/var` is a link to `/private/var`, which is the case the decision
describes.

## The gap, and its repair

No test noticed the leftover check, because `a_leftover_staged_document_is_refused` left behind
`.design.html.archiving`, and archival rewrites `design.html`. Without the check, archival went on to
stage the rewrite, met the leftover file, which `write_new` refuses to overwrite, and refused with a
message that also named it. So the test passed for another reason.

The test now runs twice, once with a leftover for a document archival rewrites and once with one for
`specification.html`, which in that fixture carries no relative reference and so is not rewritten. It
also requires the refusal to say how the file got there. Without the check, archival would carry the
second leftover into the archive. Re-run with the check undone:

```
the leftover check: a staged document already in the change is not refused -> 1 failed
- a_leftover_staged_document_is_refused
```

## The path check, re-run after a refactor

After the first run, `Route::of` became total and the scheme test moved to the reference alone
(`Route::of_reference`), so a store's own route can never be dropped. The path-check edit was made
again on the new line, `seen_through: Some(Route::of(path))`, and the same five tests failed.
