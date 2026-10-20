# The measurement repeated against HEAD, before any change

**Date:** 2026-10-08
**Tasks:** 1.1 and 1.2
**Binary:** `htmlspec 0.24.0 (b768860d5b82)`, built from `main` at `b768860` in this change's
worktree before any edit, and copied aside as `htmlspec-head` so that task 7.1 can run the same
recipe against the built change. Every `htmlspec` below is that copy.
**Projection:** the migration's `project-model` tool, copied into the session scratchpad and built
against this worktree's `crates/htmlspec-parse` at `b768860`. It is written `project-model` below.
The tool under `~/projects/html-spec-migration/` was not modified or run.

## Task 1.1: the ten variants of evidence section 5

The scratch project was built as section 5 states: `htmlspec init --tools none`, `new change
measure-partial`, and `new document specification … --subject "A partial verification, measured"`,
each exit 0. The variants were written by appendix A's script with the two rows its table carries
and its code did not: **i**, the typo `notrun`, and **j**, a second `method` paragraph reading "A
second method paragraph, which the projection keeps or drops." Each variant was run through
`htmlspec --json validate` by a short script that lists every diagnostic but the evidence unit's
`review/stale-reference` on `evidence-the-head-binary`, which every row carries once, as before.

| Variant | `data-outcome` | Other edit | Exit | Diagnostics on the verification or requirement |
| --- | --- | --- | --- | --- |
| a | `not-run` | none | 0 | `review/stale-reference` |
| **b** | **`partial`** | none | **1** | **`schema/invalid`**, `review/stale-verification` |
| c | `partial` | an `unchecked` paragraph | 1 | `schema/projection`, `schema/invalid`, `review/stale-verification` |
| d | `passed` | an `unchecked` paragraph | 1 | `schema/projection`, `review/stale-verification` |
| **e** | **`failed`** | none | **0** | `review/stale-verification`, and **no `review/unverified-requirement`** |
| **f** | none | the verification removed | **0** | **`review/unverified-requirement`** |
| g | `failed` | requirement marked `modified` | 0 | `review/stale-verification` |
| h | `not-run` | requirement marked `modified` | 0 | `review/stale-reference` |
| i | `notrun` (a typo) | none | 1 | `schema/invalid`, `review/stale-verification` |
| j | `not-run` | a second `method` paragraph | 0 | `review/stale-reference` |

**Every row is what the 2026-09-30 measurement recorded, and every expectation of task 1.1 holds:**

- `partial` is `schema/invalid`, with the same message:
  `"partial" is not one of "passed", "failed" or 2 other candidates`.
- `data-field="unchecked"` is `schema/projection` on `partial` (c) and on `passed` (d):
  `data-field="unchecked" names no field of this unit; it accepts "method", "note"`.
- `failed` draws no `review/unverified-requirement` (e), and removing the verification draws it (f).
- `partial` (b) and the typo `notrun` (i) both draw `review/stale-verification`, with the messages
  "this verification records the outcome partial against …" and "… the outcome notrun against …".
- The second `method` paragraph (j) is dropped with no diagnostic. The projected verification's
  `method` is the first paragraph alone:

```json
[{"change": "added", "id": "verify-the-check-reached-part",
  "method": "State how the requirement was checked, precisely enough that someone else could repeat it.",
  "outcome": "not-run",
  "references": [{"origin": "attribute", "target": "req-a-check-covers-its-scenario"}],
  "title": "The example verification verify-the-check-reached-part", "unit": "verification"}]
```

The projected models of b, c, and d keep the outcome as written and hold no trace of the
`unchecked` paragraph, as section 5 recorded for c.

## Task 1.2: the staged migration, recounted

`~/projects/html-spec-migration/staged/` still exists, with every file dated 2026-09-29. It was
copied into the scratchpad, and the copy was projected the way section 3 projected the original:

```
$ find htmlspec -name '*.html' -not -path '*/evidence/*' -print0 | sort -z | xargs -0 project-model > pm-staged-head.json
exit 0
```

Counted by a short Python reading of `verifications[].outcome`:

```
not-run htmlspec/changes/archive/2026-08-10-html-spec/specification.html verify-print
blocked htmlspec/changes/archive/2026-08-22-accept-a-described-change/specification.html verify-described-change-run-2026-08-22-a-description-with-no-name
109 documents projected; 21 hold verifications; 94 units; {'passed': 92, 'not-run': 1, 'blocked': 1}
```

**The counts are section 3's: 94 units in 21 specifications, 92 `passed`, 1 `not-run`, 1
`blocked`, 0 `failed`.** No staged document carries `data-outcome="partial"` or
`data-field="unchecked"`: `grep -rl` over the copy finds none.

The four units of evidence sections 1 and 2:

| Unit | Section 3 said | Now |
| --- | --- | --- |
| `verify-a-second-store-planted-inside-the-project` | removed on review | absent |
| `verify-mutation-19` (in `record-a-renamed-requirement`) | removed on review | absent from that change's specification, which holds four units: `verify-six-rules-each-seen-failing-before-it-existed`, `verify-each-invalid-fixture-reports-exactly-its-one-error`, `verify-mutations-1-to-10`, `verify-mutations-11-to-18` |
| `verify-reference-names-a-fabricated-address` | `passed` | `passed`, in `2026-08-13-content-address-runtime-assets` |
| `verify-a-second-copy-of-one-library` | `passed` | `passed`, in the same specification |

**One identifier needs a note.** A unit named `verify-mutation-19` does exist in the staged tree, in
`2026-09-26-log-a-runbook-run/specification.html`, as `passed`. It is another change's mutation 19,
"Mutation 19: only task units copied", which names
`req-a-record-carries-the-procedure-it-ran-and-the-obligations-it-attests-to`. It is not the unit
review removed from `record-a-renamed-requirement`, which was about the collision check.
