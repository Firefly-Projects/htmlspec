# The measurement against the built change

**Date:** 2026-10-08
**Tasks:** 7.1 and 7.2
**Binary:** `htmlspec 0.24.0 (b768860d5b82-dirty)`, built from the working tree committed as
`502b98a`, with the workspace version not yet moved. Models come from the scratch copy of
`project-model` rebuilt against this worktree's changed `crates/htmlspec-parse`. The HEAD side of
every comparison is the run recorded in `the-measurement-repeated-2026-10-08.md`, with the binaries
saved from it. The scripts are that file's appendices, unchanged.

## 1. A risk with no mitigation, in a scratch project (task 7.1)

The same five variants, in a scratch project built by the changed `init`, whose design declares
`0.7.0`:

| Variant | Response written | Mitigation paragraph | HEAD | This change | `response` in the model |
| --- | --- | --- | --- | --- | --- |
| A | none | none | exit 1, `schema/invalid` | **exit 1, `schema/missing-mitigation`** | `mitigated` |
| B | `accepted` | none | exit 1, `schema/invalid` | **exit 0** | `accepted` |
| C | `accepted` | "Not mitigated, …" | exit 0 | exit 0 | `accepted` |
| D | none | "Not mitigated, …" | exit 0 | exit 0 | `mitigated` |
| E | none on the risk, `accepted` on the decision | "Not mitigated, …" | exit 0 | exit 0 | `mitigated` |

**Every expected result holds.**

- **A reports `schema/missing-mitigation` alone**, at the risk, and no `schema/invalid`:

  ```
  error: schema/missing-mitigation at risk-the-inputs-are-prose: a risk whose response is mitigated
  states no mitigation. A risk that declares no data-response is mitigated, so a risk written before
  the attribute existed is held to one. State what holds it down in a data-field="mitigation"
  paragraph, or, if this change bears the risk as it stands, declare data-response="accepted" and
  say in its statement why it is borne
  ```

  The `--json` diagnostic carries `unitId` and no `schemaPath`, as every relational rule's does.
- **B validates.** Its risk projects `"response": "accepted"` and no `mitigation`. This is the risk
  the author meant to write on 2026-09-30, and the attribute now reaches both the model and the
  verdict.
- **C validates**, with `"response": "accepted"` and the workaround sentence as its `mitigation`.
  The sentence still reads oddly, and nothing stops an author writing it. That is the release
  runbook's form, which this change leaves for a later runbook release to rewrite.
- **D validates**, with `"response": "mitigated"`.
- **E is unchanged.** The decision's model is byte-identical to HEAD's: `"status": "accepted"`, and
  nothing from `data-state`, `data-outcome`, or `data-response`.

**Each risk's model is HEAD's plus one field.** For every variant, every key HEAD projected has the
same value, and `response` is the only key added.

## 2. Every risk unit, and what validation reports (task 7.2)

**Every risk that existed at HEAD projects `mitigated`.** Joined by path and identifier, the 87
tracked risks and the 11 staged ones project `"response": "mitigated"`, each with the mitigation it
had. None was lost.

The tracked count is now 95, because this change adds eight risks in five fixtures, each written to
pin a response:

| Fixture | Risk | Response | Mitigation |
| --- | --- | --- | --- |
| `invalid-schema-missing-mitigation` | `risk-example` | `mitigated` | none |
| `invalid-schema-unknown-response` | `risk-example` | `partial` | yes |
| `valid-risk-accepted-with-mitigation` | `risk-example` | `accepted` | yes |
| `valid-risk-accepted-without-mitigation` | `risk-example` | `accepted` | none |
| `risk-response` (projection) | `risk-no-response` | `mitigated` | yes |
| `risk-response` (projection) | `risk-accepted-alone` | `accepted` | none |
| `risk-response` (projection) | `risk-accepted-with-a-measure` | `accepted` | yes |
| `risk-response` (projection) | `risk-mitigated-alone` | `mitigated` | none |

The template's example risk now declares `data-response="mitigated"` and projects as it did.

**`validate --json` reports byte-identical payloads on HEAD and on the change**, for each of the
three trees:

| Tree | Documents | Errors | Warnings | Payloads |
| --- | ---: | ---: | ---: | --- |
| `runbooks/` | 2 | 0 | 0 | identical |
| `fixtures/dogfood/` | 6 | 0 | 18 | identical |
| the staged migration's `htmlspec/` | 109 | 0 | 781 | identical |

The staged tree's 109 documents and 781 warnings are the counts its promotion check recorded on
2026-09-29. Validation reads one of its 11 risks, the drafted one: the other ten sit in carried
evidence, which document discovery leaves out. That one risk carries a mitigation, so it validates
as before, as `mitigated`. Its author calls it accepted, and declaring so is
`migrate-into-the-extended-vocabulary`'s work.

The staged migration was read from a copy in the scratch directory. The original's 291 files, and
the copy's, hash the same as before the first run.
