# The measurement, repeated against the built change

**Date:** 2026-10-01
**Task:** 5.1
**Binary:** `htmlspec 0.18.0 (62403651e46d-dirty)`, built from this change's working tree before the
release, and again as `htmlspec 0.19.0 (8032003a7e7d-dirty)` after the review's fixes. The second
run differs from the first only in the `ins` message, whose reason the review rewrote, and in the
version line. The diagnostics below are the second run's.
**Script:** `measure.sh` and `write_docs.py`, unchanged from the appendix of
`the-measurement-repeated-2026-10-01.md`, run into a fresh scratch project.

## Every document with markup in or around a unit now fails

| Section | Before | After |
| --- | --- | --- |
| 1 and 2: `nothing else` and `store` in one statement, over the project | exit 0, 0 errors, 6 warnings | exit 1, 2 errors, 6 warnings |
| 2: `--strict` on the specification | exit 0 | exit 1, 2 errors |
| 2: `--json` on the specification | `"accepted": true` | `"accepted": false`, two `schema/change-markup` diagnostics with `"unitId": "req-the-spellings-are-fixed"` |
| 3: the block-level `<del><p>` | exit 0 | exit 1, 1 error |
| 5: the deleted link in a verification's method | exit 0 | exit 1, 1 error at `verify-the-spellings-are-fixed` |
| 5: the control, the words removed | exit 1, `schema/invalid` | the same |
| 5: the requirement wrapped whole in `del` | exit 0 | exit 1, 1 error at `req-no-other-spelling` |
| 5: the proposal, markup in four places outside every unit | exit 0, no diagnostics | the same |

The warnings are the ones the evidence recorded. No other diagnostic changed. The diagnostics, as
`validate` prints them:

```
error: schema/change-markup at req-the-spellings-are-fixed: <del>nothing else</del> sits in the requirement req-the-spellings-are-fixed. The projection gives change markup no meaning, so the withdrawn words are still part of what the requirement states. Write the text as it now reads, without the del element; a requirement whose words this change edits is marked data-change="modified"
error: schema/change-markup at req-the-spellings-are-fixed: <ins>store</ins> sits in the requirement req-the-spellings-are-fixed. The projection keeps the inserted words as ordinary text and gives the markup no meaning, so the edit the page shows is in no field the model holds. Write the text as it now reads, without the ins element; a requirement whose words this change edits is marked data-change="modified"
error: schema/change-markup at req-the-spellings-are-fixed: <del>No other spelling SHALL name a command.</del> sits in the requirement req-the-spellings-are-fixed. …
error: schema/change-markup at verify-the-spellings-are-fixed: <del>, as the requirement states them</del> sits in the verification verify-the-spellings-are-fixed. …
error: schema/change-markup at req-no-other-spelling: <del>No other spelling req-no-other-spelling No other spelling SHALL …</del> encloses the requirement req-no-other-spelling. The projection gives change markup no meaning, so the requirement still projects as one the document states. Remove the del element, and mark the requirement data-change="removed" to withdraw it
```

The wrapped requirement is quoted by its first eight words. They include its identifier, because
the template shows the identifier inside the heading and the quote is the element's text as it
stands.

## The projection is unchanged

Every array and object `project-model` printed, eight in all, is byte-identical before and after.
That covers sections 1, 3, and 5, including the formatted copy of section 3. The deleted words are
still in the model, and the deleted link is still a reference. The documents that carry them no
longer validate, and that is the whole of the change.

`format --check` gives the same verdicts as before: the block-level `<del><p>` is not canonical,
and section 1's document is.

## The count

The counter found 5 `<ins` and 8 `<del` in five documents, as it did before the change. The script
writes the same documents, so this is a check that the run was the same run.
