# The measurement repeated against the built change

**Date:** 2026-10-08
**Task:** 7.1
**Binary:** `htmlspec 0.24.0 (b768860d5b82-dirty)`, built from this change's worktree with groups 1
to 6 in place, copied aside as `htmlspec-change`.
**Projection:** `project-model`, the scratchpad copy of the migration's tool, rebuilt against this
worktree's `crates/htmlspec-parse`. Before use it was checked to project `named-fields`'
`verify-partly` with both `unchecked` entries.

The recipe is task 1.1's, unchanged: a scratch project set up with `init --tools none` by this
binary, so the specification template declares `0.6.0` and names `sha256-03627137bcbb81c9`; `new
change measure-partial`; `new document specification`; the ten variants from the same script; and
the same measuring script, which lists every diagnostic but the evidence unit's
`review/stale-reference`, present once on every row as before.

## The ten variants

| Variant | `data-outcome` | Other edit | HEAD | The built change |
| --- | --- | --- | --- | --- |
| a | `not-run` | none | 0: `review/stale-reference` | 0: `review/stale-reference` |
| **b** | **`partial`** | none | 1: `schema/invalid`, `review/stale-verification` | **1: `schema/missing-unchecked`, `review/stale-verification`** |
| **c** | **`partial`** | an `unchecked` paragraph | 1: `schema/projection`, `schema/invalid`, `review/stale-verification` | **0: `review/stale-verification`** |
| d | `passed` | an `unchecked` paragraph | 1: `schema/projection`, `review/stale-verification` | 0: `review/stale-verification` |
| e | `failed` | none | 0: `review/stale-verification` | 0: `review/stale-verification` |
| f | none | the verification removed | 0: `review/unverified-requirement` | 0: `review/unverified-requirement` |
| g | `failed` | requirement `modified` | 0: `review/stale-verification` | 0: `review/stale-verification` |
| h | `not-run` | requirement `modified` | 0: `review/stale-reference` | 0: `review/stale-reference` |
| i | `notrun` (a typo) | none | 1: `schema/invalid`, `review/stale-verification` | 1: `schema/invalid`, `review/stale-verification` |
| j | `not-run` | a second `method` paragraph | 0: `review/stale-reference` | 0: `review/stale-reference` |

**Every expectation of task 7.1 holds:**

- **`partial` with an `unchecked` paragraph is accepted** (c), exit 0, with
  `review/stale-verification` as its only diagnostic on the verification.
- **`partial` without one is rejected as `schema/missing-unchecked`** (b), exit 1, at the
  verification, and nothing else is an error:

  ```
  error   schema/missing-unchecked at verify-the-check-reached-part: a verification with outcome partial states nothing it did not check. Add a paragraph marked data-field="unchecked" for each part of what it names that the check did not reach, or the record claims the part it reached and leaves the rest unnamed
  ```

- **Neither draws `review/unverified-requirement`**, and removing the verification still does (f).

The projected verification of c holds the paragraph in its own field:

```json
[{"change": "added", "id": "verify-the-check-reached-part",
  "method": "State how the requirement was checked, precisely enough that someone else could repeat it.",
  "outcome": "partial",
  "references": [{"origin": "attribute", "target": "req-a-check-covers-its-scenario"}],
  "title": "The example verification verify-the-check-reached-part",
  "unchecked": ["The Then names both locations, and the check recorded only the rule and the exit status."],
  "unit": "verification"}]
```

## What else changed, and what did not

- **d, `passed` with an `unchecked` paragraph, is now accepted**, with the paragraph in
  `unchecked`. That is design decision 4: the field is accepted on every outcome, and a rule
  rejecting it on `passed` is not added.
- **i, the typo `notrun`, is still `schema/invalid`**, and its message now counts the enumeration's
  fifth value: `"notrun" is not one of "passed", "failed" or 3 other candidates`. It still draws
  `review/stale-verification`, because `review.rs`'s `has_run` counts any value but the not-run one
  as having run. That is the pre-existing fault the proposal leaves out of this change.
- **j still drops the second `method` paragraph with no diagnostic**, the other fault left out.
- **a, e, f, g, and h are what they were**, so the four earlier outcomes behave as before.

## The staged migration, as a regression check

The scratchpad copy of `~/projects/html-spec-migration/staged/` was projected again with the
rebuilt `project-model`: the JSON is byte-identical to task 1.2's, 94 units, 92 `passed`, 1
`not-run`, 1 `blocked`. `htmlspec --json validate` over the whole staged project, run with the
HEAD binary and with this one, reports 109 documents, 0 errors, and 781 warnings each, and the two
reports list the same diagnostics with the same messages, document by document. So nothing the
migration would promote is read differently.
