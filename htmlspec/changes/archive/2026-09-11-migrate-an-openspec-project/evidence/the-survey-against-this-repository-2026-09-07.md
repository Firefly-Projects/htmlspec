# The survey, run against this repository

**Date:** 2026-09-07
**Tasks:** 3.1–3.7
**Method:** `htmlspec migrate survey` implemented at `crates/htmlspec-cli/src/migrate.rs`, wired
into the command surface, and run against this repository's own `openspec/`. Counts
cross-checked against an independent `find` and against a second walk implemented in the test.

## What it reports

```
198 entities in .
     9  accepted-specification
     8  change-configuration
    51  change-document
     1  configuration
     4  converted-document
    29  delta-specification
    80  evidence
    16  recorded-version

17 changes, 16 archived
these changes record no version: migrate-an-openspec-project

versions recorded by more than one change...
  0.1.0  7 changes: 2026-08-10-html-spec, 2026-08-13-content-address-runtime-assets, ...

delta sections in use: ADDED Requirements, MODIFIED Requirements, Purpose, RENAMED Requirements

the project's configuration sets: schema
nothing was written.
```

Every kind matches `find` exactly, and the eight sum to 198, which is
`find openspec -type f | wc -l`. **Zero entities classify as `unrecognized`** — every file in the
project has a kind.

## Why 198 and not the 193 recorded in task 1.1

Task 1.1 measured 193 files. The survey finds 198. The difference is five evidence files: the
ones written by this change's own section 1, in this session, between the two measurements.

That is worth stating plainly because it is the third time this change has been caught by a
recorded count going stale — first the seven figures in task 1.1, then the counts inside
`design.md` and `proposal.md`, now the count in the evidence file that corrected them. The
pattern is not carelessness. It is that a count is a measurement with an expiry date, and this
project has already stated the general form of the rule in `contract-authority`'s "Every workflow
carries this obligation in one wording", which is written *without* a count for exactly this
reason.

**So task 3.4's instruction to "assert the counts exactly" was not followed literally.** Doing so
would have written 198 into a test that fails the next time anybody adds an evidence file — the
same defect one layer down. The tests assert properties that survive the project growing:

| Test | What it pins |
| --- | --- |
| `the_survey_enumerates_every_file_the_project_holds` | the survey's entity list equals an independently-implemented walk of the same tree, file for file, in the same order |
| `every_file_in_this_repository_classifies` | zero `unrecognized`, and all eight known kinds present |
| `the_counts_account_for_every_entity` | the per-kind counts sum to the total |
| `the_greatest_recorded_version_is_the_projects_latest` | the greatest version is numerically greatest and appears in the changelog |
| `the_version_several_changes_share_is_reported_with_all_of_them` | the shared version is reported, and cross-checked against `scripts/check-release`'s own `BASELINE` |
| `the_delta_section_the_destination_cannot_express_is_surfaced` | the rename is surfaced, and the schema still has no `renamed` state |
| `surveying_this_repository_writes_nothing` | the tree is identical before and after |

The first is the one that does the work task 3.4 asked for. A survey that misses a file fails it;
a survey counting a project that legitimately changed does not.

Two of these tests are written to fail when the situation they describe is fixed rather than
broken. `the_delta_section_the_destination_cannot_express_is_surfaced` reads the schema and fails
if a `renamed` state appears, because that is the moment the plan should stop calling it an open
question. `the_version_several_changes_share_is_reported_with_all_of_them` reads
`scripts/check-release` and fails if a shared version is not the script's baseline.

## Task by task

**3.1 — the command exists.** `htmlspec migrate survey [path]`, enumerating capabilities, changes,
change documents, delta specifications, evidence files, recorded versions, and configuration —
**plus the four entity kinds the design's table had no row for**. `Kind` has nine variants; four
of them (`ConvertedDocument`, `ChangeConfiguration`, the nested-evidence handling, and the delta
section recording) exist because task 1.2's audit found them, and `Unrecognized` exists so a kind
nobody anticipated still reaches the plan.

**3.2 — `--json` emits the payload the gate consumes.** Keys: `root`, `counts`,
`greatestVersion`, `sharedVersions`, `configuration`, `changes`, `entities`. Entity paths are
**relative** to the repository root, so two surveys of one project run from different directories
produce identical payloads — a gate comparing a plan against a payload must not depend on where
the migration ran.

**3.3 — a non-project is reported as such.** Names the path, names what was expected
(`openspec/specs` or `openspec/changes`), and exits **2**, which `exit.rs` reserves for an
operation that could not run rather than a document that failed a check.

```
$ htmlspec migrate survey /tmp/not-a-project
htmlspec: /tmp/not-a-project is not an OpenSpec project: no openspec/specs or
openspec/changes directory at /tmp/not-a-project/openspec. A migration reads an
OpenSpec project, so point it at the repository holding one
exit: 2
```

A project with capabilities and no changes is still a project — requiring both directories would
refuse a young project for being young. Pinned by
`a_project_with_no_changes_is_still_a_project`.

**3.4 — counts asserted.** Above.

**3.5 — the survey writes nothing.** Asserted twice: against a constructed source
(`the_survey_writes_nothing`) and against this repository
(`surveying_this_repository_writes_nothing`). Both compare the file list before and after, which
is what catches a survey that writes a cache or a lock file — something every other test here
would pass.

**3.6 — repeatable, in the same order.** `the_survey_is_repeatable_in_the_same_order` compares two
surveys and their payloads, then asserts the entity order is the sorted one. Order matters because
a coverage failure is diagnosed by a person diffing two payloads, and filesystem order differs
between machines.

**3.7 — an absent version reports as absent.** `a_change_recording_no_version_reports_none`
asserts `version: None`, and asserts the payload carries the field **as null rather than omitting
it** — so a reader cannot mistake "recorded none" for "did not look". This repository supplies a
real case without construction: `migrate-an-openspec-project` is an active change with no
`VERSION`, and the survey names it rather than defaulting it.

## A defect the implementation found: version ordering

`greatest_version` compares numeric components, not text. Written the obvious way it would
compare strings, and `"0.9.0" > "0.10.0"` is true in string ordering.

The consequence is not cosmetic. A migrated project's declaration is set to the greatest archived
version, and `htmlspec archive` refuses unless the declared version is ordered after the last
archived one. A survey reporting `0.9.0` here would set the declaration to `0.9.0` while the
archive contained `0.10.0` — so the next archival would compare against a version it had already
passed, and **pass without advancing**. That is precisely the failure
`project-bootstrap`'s "Archival requires the project version to advance" exists to prevent, and it
would have arrived through the migration rather than through the gate.

Pinned by `the_greatest_version_is_ordered_numerically_rather_than_as_text`, using `0.9.0`,
`0.10.0`, `0.2.0`.

## The tests were checked against a mutation

A passing test proves nothing about a bug it cannot see. The nested-evidence descent was broken
deliberately — restricted to depth 1, which is the flat-glob bug that under-counted this
repository's evidence by four — and both layers caught it:

- `evidence_nested_deeper_than_the_format_describes_is_enumerated` FAILED
- `every_file_in_this_repository_classifies` FAILED (the four transcripts became `unrecognized`)

Restored, and the suite returned to green.

## State of the suite

- `cargo test`: **all green**, 30 result lines, zero failures. 15 unit tests in `migrate.rs`, 7
  integration tests in `tests/migrate.rs`.
- `cargo clippy --all-targets`: **no warnings**.
- `cargo fmt --all`: applied.
- `./scripts/check`: **all 10 checks passed**, including `installed` (40 instruction files match
  what `init` writes) and `release`.

The `installed` check still passes because section 7 has not run yet — no workflow body has been
added, so the compiled table and the installed files still agree. That check is what task 9.1
exists to satisfy once `migrate.md` lands.
