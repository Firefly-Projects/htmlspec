# Group 8: verifying the whole property

Every measured claim in this change, with the command that produced it. Written at the
end of group 8, against commit `86c33f4`.

## 8.1 — `scripts/check` passes

```
$ ./scripts/check
=== format ===      === lint ===        === test ===
=== doctest ===     === self-conformance ===
=== self-format === === self-assets ===
all 7 checks passed
```

Seven steps, up from six. `self-assets` is the new one, added in group 4. It failed
deliberately from `f9ea88f` until `86c33f4` removed the duplication — task 4.5 exists to
confirm that failure happened, and it did.

## 8.2 — each library appears exactly once

```
$ htmlspec assets .
checking 8 stored asset(s) under .
each stored asset appears once
```

Cross-checked independently of the tool, by digest, over every stylesheet, script, and
schema in the repository. One duplicate digest turned up and it is not a violation:

```
digest d8779069bc923a3c…:
  fixtures/conformance/valid-asset-reference-matches-stored-bytes/assets/sha256-d8779069bc923a3c/htmlspec.css
  fixtures/conformance/invalid-asset-unresolved/assets/sha256-d8779069bc923a3c/htmlspec.css
```

Those are two conformance fixtures, each of which **is its own project** carrying its own
self-contained store. `htmlspec assets` run against either reports "each stored asset
appears once", because the rule's subject is a project and these are two projects that
happen to pin the same fixture bytes. Each store is 4 KB. Sharing one store between them
would couple two fixtures whose independence is the point.

## 8.3 — the skip-link defect

**Static verification: complete.** All four conditions for a working keyboard skip link
hold in the re-rendered document.

| Condition | Result |
|---|---|
| Skip link present, first focusable element in `<body>` | yes |
| Its target `#hs-content` exists in the document | yes |
| Natively focusable — an `<a href>` needing no `tabindex` | yes |
| Referenced stylesheet has the hidden rule *and* the `:focus` reveal | yes |

The stylesheet it now references is
`../../../../../assets/sha256-3e4014c3004abea5/htmlspec.css`, which contains:

```css
.hs-skip-link { position: absolute; left: -9999px; top: 0; … }
.hs-skip-link:focus { left: var(--hs-gutter); top: var(--hs-gutter); }
```

Before this change the document referenced a stylesheet with **zero** `hs-skip-link`
rules, so the link was positioned statically and the first control a keyboard user
reached did nothing visible. That is the defect that prompted the whole change.

**Interactive verification: done by Scott, 2026-08-13.** I could not run it myself. The
task asks for the link to be reached by keyboard, which is an interaction rather than a
file property, and this machine has Firefox and Safari but no Chrome and no WebDriver. So
I opened the document and Scott performed the check: `Tab` reveals "Skip to content" at
the top-left corner, and `Enter` reaches the content region.

That closes the loop the change opened. The defect was a keyboard user pressing `Tab` and
getting nothing, and a keyboard user pressing `Tab` now gets the link.

## 8.4 — every new rule fails when its defect returns

A check that has never failed is a check nobody has tested. Each defect was reintroduced
into a fresh `htmlspec init` project and the exit code recorded.

| Defect reintroduced | Reported | Exit |
|---|---|---|
| Reference names a fabricated address | `asset/unresolved` | 1 |
| Stored bytes replaced in place | `asset/mismatch` | 1 |
| A second copy of one library | `asset/duplicate` | 1 |

The two reference faults are **distinguishable**, which the specs require because their
causes differ. With bytes replaced in place, `asset/unresolved` fired zero times.

The duplication case also confirms why it needed its own check suite step:

```
$ htmlspec assets .       → exit 1   (duplication reported)
$ htmlspec validate doc.html → exit 0   (the document is faultless)
```

The document is not at fault, and per-document validation correctly says so. Only a
project-level pass sees the violation.

### A coverage boundary this exercise found, stated plainly

A fourth scenario was tried: a document that **honestly** names an older stylesheet
revision which lacks a rule the document declares. The store holds those exact bytes, the
hash matches, and `htmlspec validate` accepts it — exit 0.

That is correct behaviour for these rules. `asset/*` checks whether a reference names the
bytes beside it, not whether those bytes implement every class the document uses. A
document may legitimately pin an older runtime.

What matters is whether the *original* defect is still catchable, and it is — by a
different step. The archive's real reference was `assets/htmlspec.css`, unaddressed,
naming no bytes at all. Reconstructed at `18acf0d` and run against the current toolchain:

```
$ htmlspec validate doc.html    → exit 0   (nothing to resolve: unaddressed)
$ htmlspec format --check doc.html → exit 1   (not canonical: references unfilled)
```

`format --check` gates it, and `scripts/check` runs `format --check`. So the defect fails
the suite, through the formatter rather than the validator.

**The residual gap, named rather than glossed:** a document that pins an old revision
*addressed and honestly* is accepted, even when that revision lacks a rule the document
declares. Closing it would mean checking declared classes against stylesheet contents,
which is a different rule than anything in this change and was never in its scope. It is
worth a future proposal. It is not a regression: no earlier version of this project caught
it either.

## 8.5 — the measurements

Tracked bytes, from Git rather than from the working tree, so the figures are what a clone
costs.

| | `18acf0d` (before) | `86c33f4` (after) |
|---|---|---|
| Asset bytes tracked | 18.1 MB | 6.0 MB |
| Whole tracked tree | 23.8 MB | 11.9 MB |

The duplication removed, as one diff:

```
$ git diff --stat 18acf0d..HEAD -- assets \
    openspec/changes/archive/2026-08-10-html-spec/documents/assets
16 files changed, 74760 deletions(-)
```

Three trees went: the 6.0 MB copy under the archived change, `assets/vendor/`, and the
three flat stylesheets at the old top-level paths. The store grew by nothing — the same
8 blobs are still there, each once, now at their addresses.

The repository halved. The design predicted the store would grow monotonically and never
collect garbage, which remains true; what shrank is duplication, which was never storage
the project needed.

## What is not verified

- **Rendering in a browser generally.** Beyond the skip link, no document in this change
  was opened and visually compared against a reference.
  `every_figure_source_survives_a_saved_render` covers the saved-DOM property in the
  assets crate, and that is a different claim.
- **The keyboard check is not automated.** It passed once, by hand. Nothing in
  `scripts/check` would catch a regression, because the suite has no browser. A WebDriver
  test is the way to fix that and was deliberately not added here.
