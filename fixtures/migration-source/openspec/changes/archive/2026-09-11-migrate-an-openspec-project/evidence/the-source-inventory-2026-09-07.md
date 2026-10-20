# The source inventory, measured

**Date:** 2026-09-07
**Task:** 1.1
**Method:** `find` and `grep` over `openspec/` at commit `f57f640`, counts cross-checked by a
breakdown that sums to the total. Every number below was measured, not estimated.

## The headline: the planned numbers are stale

`tasks.md` was written against a snapshot that no longer exists. Six of its seven figures are
wrong, and the drift is in one direction — the project kept working while this change sat in
planning. Recording this before writing code is the whole point of task 1.1: **every later
claim about coverage is checked against the numbers below, not against the ones in the tasks
document.**

| Figure | `tasks.md` 1.1 says | Measured | Drift |
| --- | --- | --- | --- |
| markdown files | 100 | **161** | +61 |
| capabilities | 8 | **9** | +1 (`contract-authority`) |
| requirements (accepted) | 79 | **97** | +18 |
| scenarios (accepted) | 310 | **401** | +91 |
| archived changes | 11 | **16** | +5 |
| delta specification files | 21 | **28** | +7 |
| evidence files | 39 | **75** | +36 |

The stale numbers are load-bearing in five other tasks — 3.4 asserts them exactly, 6.3 stages
"this repository's 310 scenarios", 8.2 confirms "all 100 files", 8.5 confirms "the 39 evidence
files". Those tasks are checking the wrong quantities. The figures in this file are what they
must be rewritten against.

A count in a tasks document is a measurement with an expiry date. The project has already
recorded this failure mode once — `contract-authority`'s "Every workflow carries this
obligation in one wording" states its rule *without* a count, because "`sync` made an earlier
count of seven wrong one release after it was written". This is the same defect, in a
planning artifact instead of a specification.

## The file census

193 files under `openspec/`, of which:

| Kind | Count |
| --- | --- |
| markdown (`*.md`) | 161 |
| `VERSION` | 16 |
| `.openspec.yaml` (per-change) | 8 |
| legacy HTML under `documents/` | 4 |
| evidence non-markdown (`.txt`, `.py`) | 3 |
| `config.yaml` (project) | 1 |
| **total** | **193** |

The markdown breakdown, which sums to 161 exactly:

| Where | Count |
| --- | --- |
| `specs/*/spec.md` — accepted capabilities | 9 |
| `changes/archive/*/{proposal,design,tasks}.md` | 48 |
| `changes/archive/*/specs/**/spec.md` — deltas | 28 |
| `changes/archive/*/evidence/**/*.md` | 72 |
| `changes/migrate-an-openspec-project/*.md` — this change | 4 |
| **total** | **161** |

## Accepted specifications: 9 capabilities, 97 requirements, 401 scenarios

| Capability | Requirements |
| --- | --- |
| `artifact-kinds` | 11 |
| `contract-authority` | 5 |
| `deterministic-formatting` | 7 |
| `document-schema` | 12 |
| `document-template` | 8 |
| `document-validation` | 19 |
| `project-bootstrap` | 10 |
| `toolchain-packaging` | 22 |
| `workflow-reporting` | 3 |
| **total** | **97** |

All nine capability paths are one level deep. **No capability in this repository exercises a
nested path**, which is why task 8.4 has to construct a source project that does — the nesting
requirement cannot be proven against this repository at all.

`project-bootstrap`'s Purpose is still the archival placeholder: `TBD - created by archiving
change project-bootstrap-cli. Update Purpose after archive.` A migration carries the
placeholder across as content. It is not the migration's business to fix it, but the plan must
not silently improve it either.

## Archived changes: 16, every one recording a version

| Change | Version | Docs | Deltas | Evidence | `.openspec.yaml` | Legacy HTML |
| --- | --- | --- | --- | --- | --- | --- |
| `2026-08-10-html-spec` | 0.1.0 | 3 | **5** | 8 | yes | 1 |
| `2026-08-13-content-address-runtime-assets` | 0.1.0 | 3 | 2 | 3 | yes | 0 |
| `2026-08-14-separate-artifact-kinds` | 0.1.0 | 3 | 4 | 2 | yes | **3** |
| `2026-08-17-project-bootstrap-cli` | 0.1.0 | 3 | 3 | 6 | no | 0 |
| `2026-08-18-conform-generated-frontmatter-names` | 0.1.0 | 3 | 1 | 2 | no | 0 |
| `2026-08-18-install-the-full-workflow-set` | 0.1.0 | 3 | 1 | 2 | no | 0 |
| `2026-08-21-check-release-hygiene` | 0.2.0 | 3 | 1 | 3 | no | 0 |
| `2026-08-21-distinguish-command-names-from-skill-names` | 0.1.0 | 3 | 1 | 3 | no | 0 |
| `2026-08-22-accept-a-described-change` | 0.3.0 | 3 | 1 | 3 | no | 0 |
| `2026-08-24-pass-the-description-through` | 0.4.0 | 3 | 1 | 3 | no | 0 |
| `2026-08-24-report-in-the-contributors-terms` | 0.5.0 | 3 | 1 | 4 | no | 0 |
| `2026-08-30-finish-the-archival` | 0.6.0 | 3 | 2 | 4 | no | 0 |
| `2026-08-31-sync-a-change-into-the-accepted-contract` | 0.7.0 | 3 | 1 | 4 | yes | 0 |
| `2026-09-02-carry-task-progress-into-the-listing` | 0.8.0 | 3 | 1 | 7 | yes | 0 |
| `2026-09-02-state-a-changes-place-in-a-sequence` | 0.9.0 | 3 | 2 | 8 | yes | 0 |
| `2026-09-06-distinguish-a-requirement-from-an-example` | 0.10.0 | 3 | 1 | **13** | yes | 0 |
| **total** | | **48** | **28** | **75** | **7** | **4** |

Every one of the 16 records a version, so **this repository has no case for the
"source change with no version" rule** — task 3.7 and the requirement's third paragraph must
be proven against a constructed project, exactly as the design already says.

The version distribution: `0.1.0` ×7, then `0.2.0` through `0.10.0` one apiece. The greatest is
**`0.10.0`**, which is what a migrated declaration's `version` must be set to.

`2026-08-10-html-spec` carries 5 deltas spanning every capability the project then had. It is
the delta-collapse case task 1.5 works by hand, and it remains the largest: no later change
exceeds 5.

## Entity kinds the design's table does not cover

The design's OpenSpec→HTMLSpec table has six rows. The source holds **four entity kinds that
appear in no row**, and a survey built from that table alone would not enumerate them — which
means the coverage gate would pass while they went missing.

### 1. `documents/*.html` — 4 already-converted HTMLSpec documents

Two archived changes carry a `documents/` directory holding HTMLSpec documents produced by an
earlier hand-migration:

- `2026-08-10-html-spec/documents/deterministic-formatting.html` — kind `specification`
- `2026-08-14-separate-artifact-kinds/documents/{proposal,design,tasks}.html`

All four declare `htmlspec-version` 0.1.0 and reference an asset store by a relative path
(`../../../../../assets/…` and `../../../../assets/…`) that resolves to nothing inside
`openspec/`. These are the destination format already, at the wrong contract version, with
broken asset references, in a directory the destination has no name for.

This is the most interesting unmapped case, because the obvious dispositions are all wrong:
converting them re-does work, carrying them verbatim imports dangling references and a stale
version, and dropping them discards the project's own first attempt at the thing this change
automates.

### 2. `.openspec.yaml` — 8 per-change schema markers

7 archived changes plus this active one carry `.openspec.yaml` holding `schema: spec-driven`
and a `created:` date. The other 9 archived changes carry none. The design's table maps the
*project's* `config.yaml` and says nothing about these.

The `created:` date is a fact the destination format has nowhere to put — a change's date lives
in its archive directory name — and in 8 cases it **disagrees** with that directory name:
`2026-09-06-distinguish-a-requirement-from-an-example/.openspec.yaml` says `created: 2026-08-24`.

### 3. `## RENAMED Requirements` — a delta section the destination's schema cannot express

Two delta files use it:

- `2026-08-17-project-bootstrap-cli/specs/toolchain-packaging/spec.md:1`
- `2026-08-30-finish-the-archival/specs/toolchain-packaging/spec.md:310`

Both carry `- FROM:` / `- TO:` heading pairs. The destination's `changeState` enum in
`crates/htmlspec-schema/schema/document.schema.json` is exactly
`["added", "modified", "removed", "unchanged"]`. **There is no `renamed`.**

The delta section headings actually present across all 28 files:

| Heading | Occurrences |
| --- | --- |
| `## ADDED Requirements` | 17 |
| `## MODIFIED Requirements` | 17 |
| `## RENAMED Requirements` | 2 |
| `## Purpose` | 6 |

A rename carries information no permitted state holds: that one requirement *is* another under
a new heading. Mapping it to `modified` loses the identity link, which is the one thing the
rename was recorded to state — and identity is what task 4.7's derivation table is keyed on.

### 4. Nested evidence directories

`2026-09-06-distinguish-a-requirement-from-an-example/evidence/transcripts/` holds 4 files.
Evidence is not a flat list of files in every change, so a survey globbing `evidence/*` finds 9
where 13 exist. The recursive count is what the 75 figure above uses.

## Delta content, for the collapse

Across the 28 delta files: **120 requirements** and **545 scenarios**. Both exceed the accepted
totals (97 and 401), because a requirement modified across several changes appears once per
change that touched it. The collapse is therefore not a de-duplication — each change's
specification records what *that change* established, and the same requirement legitimately
appears in several archived specifications with different text.

## What this measurement changes

1. Tasks 3.4, 6.3, 8.2 and 8.5 assert stale counts and must be rewritten against this file.
2. The survey must enumerate four entity kinds the design's table omits, or the coverage gate
   is vacuous for 15 files (4 HTML + 8 yaml + 3 non-markdown evidence).
3. The survey must recurse into `evidence/`, not glob it.
4. `renamed` has no destination state, and that is a contract-level gap rather than a mapping
   choice.
5. Nested capability paths and version-less changes cannot be proven here; both need
   constructed sources.
