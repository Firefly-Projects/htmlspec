# What this change builds on, read as built

**Date:** 2026-10-08
**Tasks:** 1.0, which reads the stylesheet revision task 5.1 supersedes, and 1.3, which reads the
contract version this change moves from
**Revision:** `main` at `b768860`, after `show-what-a-modification-changed` archived as #48.

## Summary

The design was written against `main` before `mark-the-words-a-modification-changed` landed, and
three of its facts moved under it. Each is corrected in `design.md` before any code. None changes a
decision.

| What the design assumed | As built | Corrected in |
| --- | --- | --- |
| The screen stylesheet is `sha256-aac8967609f80d9f`, and it joins `retained` | `own` records `sha256-ebf35a36fe4dc1a7`, and `aac8967609f80d9f` is already in `retained` | Context, decision 7 |
| The contract moves `0.2.0` to `0.3.0`, or one minor above an earlier move | `main` declares `0.4.0`. The change below this one in the stack moves it to `0.5.0`, so this change moves it from `0.5.0` to `0.6.0` | Migration Plan |
| The outcome enumeration is at schema lines 595-603, the record composition at `runs.rs` 1596-1614 | 613-621, because `deletions` was added above it, and 1597-1615 | Context, decision 6 |

## Task 1.0: the stylesheet revision this change supersedes

`assets/vendor.lock.json` at `b768860`:

```json
"own": [
  { "path": "htmlspec.css",
    "sha256": "ebf35a36fe4dc1a784462654c5075f2e15a2e93ac7a995853876ed8e44dbbce2" },
  …
"retained": [
  { "path": "htmlspec.css",
    "sha256": "aac8967609f80d9f90b9a92adff22e495a5d4863618b5b12a3cf89c9b08453d1",
    "note": "Superseded by the revision `own` records, which draws a change's inserted and deleted words by shape and announces them. …" },
  { "path": "htmlspec.css", "sha256": "e4f5de7cecc4f691…", … },
  { "path": "htmlspec-print.css", "sha256": "0350261bb566616721cb…", … }
```

**The revision task 5.1 supersedes is `ebf35a36fe4dc1a7…`**, the one
`mark-the-words-a-modification-changed` added in #46, not `aac8967609f80d9f…`, which the design
named. Task 5.1's new revision starts from `ebf35a36fe4dc1a7`'s bytes, so it keeps that change's
`ins` and `del` rules, and `ebf35a36fe4dc1a7` moves to `retained`.

`record-a-task-done-differently`, the change below this one in the stack, touches no stylesheet: its
tasks name `assets/template-tasks.html` and the templates' pins, and no `htmlspec.css`. So this
change is the only one in the stack that moves `own`'s stylesheet entry.

The outcome labels in `sha256-ebf35a36fe4dc1a7/htmlspec.css` are at lines 698-723, one `content`
rule per value for `h2`, `h3`, and `h4`, and the comment at line 649 still says "`attr()` would be
shorter and is not used: it would render an invalid value …". The print stylesheet,
`sha256-04ac9a24fc436a87/htmlspec-print.css`, recolors `[data-outcome] > h2::after` and `h3` at
lines 263-264 with no per-value rule, so it needs nothing, as decision 7 says.

## Task 1.3: the version this change moves from

- `crates/htmlspec-schema/src/lib.rs` line 70: `pub const CONTRACT_VERSION: &str = "0.4.0";`
- `crates/htmlspec-schema/CHANGELOG.md`, newest entry: `## 0.4.0 — 2026-10-07`, from
  `mark-the-words-a-modification-changed`.
- The schema's `schemaVersion` enumeration holds `0.1.0` to `0.4.0`, and its `$id` is
  `https://html-spec.dev/schema/0.4.0/document.schema.json`.

**This change moves the contract from `0.5.0` to `0.6.0`.** `0.5.0` arrives with
`record-a-task-done-differently`, the change below this one in the stack, which adds
`data-field="deviation"` on a task. This branch starts from `main`, where that change's work does
not exist, so on this branch the enumeration goes `0.1.0`, `0.2.0`, `0.3.0`, `0.4.0`, `0.6.0`, and
the changelog's `0.5.0` entry is inserted when the stack is assembled. Nothing in this change
depends on that change's code.

The places that state the version, found with `git grep -n "0\.4\.0"` outside the archive and the
conformance fixtures: `CONTRACT_VERSION` and its doc comment, the schema's `$id` and enumeration,
the header of `doc/projection.md`, the four `assets/template-*.html` (meta element and masthead),
`SCHEMA_OWNED_LITERALS`, and the contract sentence in `README.md`. Task 3.3 lists all of them but
the README, which `mark-the-words-a-modification-changed` moved in the same step, so task 3.3 moves
it too. The root `CHANGELOG.md` and `crates/htmlspec-cli/tests/archive.rs` name `0.4.0` as a release
and as an adoption-format sample, not as the contract, and stay.

## What else the design rests on, as built

- **Every reader of the outcome takes its values from the schema**, as the design says.
  `review.rs` `has_run` (line 315) filters the schema's values by the stem `not` and counts every
  other string as having run. `runs.rs` `has_run` (line 1185) first requires the value to be one the
  schema enumerates. `requirements_are_verified` (line 132) never reads the outcome: line 148 asks
  only whether some verification's references name the requirement.
- **The record composition** (`runs.rs` 1597-1615) writes `passed` for a machine check preflight
  passed and `not-run` for every other requirement. It never writes `partial`.
- **`status_named`** in `relations.rs` returns the one enumerated value containing a stem, or `None`
  for none or several. `parti` matches only `partial` in `passed`, `failed`, `blocked`, `not-run`,
  and `partial`, so the rule can key on it.
- **`Paragraphs::single`** (`project.rs` line 410) still says "the caller reports the repetition",
  and `project_verification` still calls it for `method` and reports nothing, as evidence section 6
  found.
- **No accepted requirement and no instruction lists the outcome values.** `grep -rn "not-run"` over
  `openspec/specs/` and `crates/htmlspec-assets/instructions/` finds nothing. The values appear only
  in the schema and the stylesheet, as evidence section 7 says.
- **The delta's MODIFIED requirement equals its accepted text but for this change's edits.**
  `git diff --no-index --word-diff` of "A specification requirement without verification is
  reported as incomplete" against `openspec/specs/document-validation/spec.md` shows only the
  extended third paragraph and the two scenarios "A requirement whose check failed" and "A
  requirement verified only in part". The three headings the delta and the design cite, "The element
  contract is published as a machine-readable schema", "Violations are classified by severity", and
  "A run's conformance to its runbook's contract is reported and never gated", exist as quoted.
  `openspec validate record-a-partial-verification --strict` passes.
