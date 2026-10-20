# The entity-to-destination mapping, and the audit that found three faults in it

**Date:** 2026-09-07
**Tasks:** 1.2, 1.3, 1.4
**Method:** one row per OpenSpec entity kind found by `evidence/the-source-inventory-2026-09-07.md`.
Each row's destination is cited to **the accepted requirement that fixes it**, by requirement
heading rather than by capability. Rows citing nothing are marked silent. Every lossy row was
then read against the code rather than against prose, and every non-normative citation in the
change's own design was checked.

`contract-authority`'s "An assertion about the contract names the requirement it rests on" is
the rule this table is built to satisfy, and "Silence is recorded and not filled" is the rule
that governs the silent rows.

## The audit's result, first

The design's six-row table has **three faults**. One of them inverts an accepted requirement.

| Fault | Where | Severity |
| --- | --- | --- |
| The version row states the gate reads archive directory names. An accepted requirement states it **SHALL NOT**. | `design.md:25`, tasks 1.4, 5.4, 5.5 | **Blocking** |
| Four entity kinds have no row, so a survey built from the table is vacuous for 15 files | `design.md:20–27` | Blocking |
| The design attributes the flat-file spelling to the README. It is not there. | `design.md:191–193` | Corrigible |

## Fault 1: the version row is backwards

**The design says** (`design.md:25`):

> \| `VERSION` file \| version inside the archive directory name \| `last_archived_version` scans directory names (`crates/htmlspec-cli/src/commands.rs:1352`) \|

**The accepted contract says** — `project-bootstrap`, "Archival requires the project version to
advance", `openspec/specs/project-bootstrap/spec.md:57`:

> The last archived version SHALL be read from the version each archived change records, in the
> form `toolchain-packaging` requires. **It SHALL NOT be derived from an archive directory's
> name.** A directory name is a display string that is shortened and renamed, and a gate derived
> from one depends on a convention nothing enforces — which is how this gate came to compare
> against a value no project ever had.

**The code agrees with the contract, not the design.** `last_archived_version` is at
`crates/htmlspec-cli/src/commands.rs:2799` — not 1352 — and reads a file:

```rust
fn last_archived_version(project: &ProjectRoot) -> crate::version::Version {
    ...
    .filter_map(|entry| recorded_version(&entry.path()))
    .max()
}

fn recorded_version(directory: &Path) -> Option<crate::version::Version> {
    let text = std::fs::read_to_string(directory.join(crate::project::RECORDED_VERSION)).ok()?;
    crate::version::Version::parse(text.trim()).ok()
}
```

`RECORDED_VERSION` is `"VERSION"` (`crates/htmlspec-cli/src/project.rs:46`).

**Line 1352 is where the claim came from, and it was true once.** At `cf11132^` the function
sat at exactly line 1352 and read `archived_version(&entry.file_name().to_string_lossy())` — it
scanned directory names. `cf11132` is titled *"Finish the archival, and fix the gate that never
fired"*, and the requirement quoted above is the contract that change established. So the
design cites a real line of real code — from before the fix whose whole subject was this
defect. The citation is not careless; it is stale in the most convincing possible way.

**What the destination actually writes.** `archived_name` (`commands.rs:2176`) composes
`{today}-{name}` with **no version component**, and archival writes the version to a `VERSION`
file inside the archived directory (`commands.rs:2153`). `scripts/check-release` reads the same
file (`VERSION_FILE = "VERSION"`, line 66).

**The README is where `<date>-<version>-<name>` comes from.** `README.md:88` shows
`2026-08-04-0.2.0-add-dark-mode` under a heading reading "Proposed project layout", four lines
below `**Status:** early design proposal. The name, commands, and file layout are working
ideas.` Nothing writes that name and nothing reads it. `contract-authority`'s "The project's
obligations are what its accepted specifications state" makes a README non-normative, and its
scenario "A layout stated only in a README" is this case exactly.

**So the row is:** `VERSION` file → `VERSION` file, carried across unchanged. The mapping is the
identity, and the design's whole "version history is reconstructed from directory names"
decision (`design.md:232–248`) describes work that must not be done. Tasks 5.4 and 5.5 encode
it and are rewritten.

This is the second time this change has cited a README layout as a contract. The first is
recorded in `design.md:183–213` as the reason the layout-preservation requirement exists. The
same fault, in the same document, in a row the author believed was verified against code.

## Fault 2: four entity kinds have no row

Recorded in full in `evidence/the-source-inventory-2026-09-07.md`. In summary: 4
`documents/*.html`, 8 `.openspec.yaml`, `## RENAMED Requirements` in 2 delta files, and a
nested `evidence/transcripts/` directory. 15 files plus one delta section the destination's
schema cannot express. A survey enumerating only the design's six rows reports coverage over a
set that omits them, which makes the gate in task 4.2 pass while they go missing — the exact
outcome the gate exists to prevent.

## Fault 3: the flat spelling is not in the README

**The design says** (`design.md:191–193`):

> The single flat-file spelling appears once, in the README, four lines under a status line
> reading *"early design proposal. The name, commands, and file layout are working ideas."*

`grep -n 'htmlspec/specs' README.md` returns nothing. The README's layout block
(`README.md:78–90`) shows `specs/theme-selection.html`, which is a flat spelling but is neither
`htmlspec/specs/<name>.html` nor "four lines under" the status line — it is 73 lines under it.

`htmlspec/specs/<name>.html` appears in **eight** places: once in each of the eight installed
workflow bodies, as the worked example of a sentence that *owes a requirement*
(`crates/htmlspec-assets/instructions/archive.md:81`):

> - **Owes one.** "Accepted specifications live at `htmlspec/specs/<name>.html`." Nothing else in
>   the sentence carries that claim.

The spelling the design treats as the source of an invented convention is, in the current tree,
the project's own illustration of a claim requiring a citation. The design's conclusion survives
— the contract states no layout beneath `htmlspec/specs/`, so the source's shape is preserved —
but its evidence for how the fault arose is wrong, and the fix is to cite the eight bodies.

## The table

Destinations are cited by requirement heading. `→` is the destination; **silent** means no
accepted requirement fixes it, so `contract-authority`'s preserving route applies.

### Rows the contract fixes

| # | Source entity | Destination | Requirement that fixes it | Lossy |
| --- | --- | --- | --- | --- |
| 1 | project root `openspec/` | `htmlspec/` | `project-bootstrap` — "A project has a declared root holding one asset store" | no |
| 2 | *(none in source)* | `spec.nix` declaration | `project-bootstrap` — "A project declares its name, version, and expected toolchain" | no — added |
| 3 | *(none in source)* | `htmlspec/assets/`, one store | `project-bootstrap` — "A project has a declared root holding one asset store" | no — added |
| 4 | `changes/archive/<date>-<name>/` | `htmlspec/changes/archive/<date>-<name>/` | `project-bootstrap` — "A project has a declared root holding one asset store" (holds "its archive") | no |
| 5 | `{proposal,design,tasks}.md` | `{proposal,design,tasks}.html` | `artifact-kinds` — "A change is a directory of kinded documents" | no |
| 6 | N delta files per change | one `specification.html` | `artifact-kinds` — "Archival produces the specification and then updates the accepted contract" (`spec.md:256`: "A change directory SHALL hold at most one specification artifact") | **yes** |
| 7 | `## ADDED/MODIFIED/REMOVED Requirements` | `data-change` on the unit | `document-schema` — "Semantic units are identified by class"; enum in `crates/htmlspec-schema/schema/document.schema.json` | no |
| 8 | `VERSION` | `VERSION`, unchanged | `project-bootstrap` — "Archival requires the project version to advance" (`spec.md:57`, quoted above) | no |
| 9 | `config.yaml` `context` and `rules` | **nothing** | `project-bootstrap` — "A project declares its name, version, and expected toolchain" (`spec.md:34`: "The declaration SHALL NOT carry authoring rules or project context") | **yes** |
| 10 | `AGENTS.md`, `.claude/`, `.codex/`, `.pi/` | not migrated; `init` writes replacements | `project-bootstrap` — "Bootstrapping installs agent instructions for the tools a project uses" | **yes**, deliberately |
| 11 | `evidence/**` (75 files) | `evidence/**`, byte-identical | `artifact-kinds` — "A design reasons in whatever form the reasoning takes" (evidence units reference it); carried rather than converted | no |

### Rows the contract does not fix — silence, preserved

| # | Source entity | Destination | Why silent | Route |
| --- | --- | --- | --- | --- |
| 12 | `specs/<capability-path>/spec.md` | `htmlspec/specs/<capability-path>/spec.html` | No requirement states a layout beneath `htmlspec/specs/`. `project-bootstrap` says only "each in its own place". `ProjectRoot::specs()` (`project.rs:160`) constrains nothing below it. The flat spelling is non-normative in all eight occurrences. | **preserve** — source's nested path kept |
| 13 | `evidence/transcripts/` (nested) | same nesting | No requirement states evidence is flat | **preserve** |
| 14 | `documents/*.html` (4 files) | *(see below)* | No requirement reaches a source project's already-converted documents | **preserve + report** |
| 15 | `.openspec.yaml` (8 files) | *(see below)* | No requirement reaches a per-change source config | **drop + report** |
| 16 | `## RENAMED Requirements` (2 files) | *(see below)* | `changeState` has no `renamed`. No requirement states what a rename becomes. | **open question, named** |

`contract-authority`'s "Silence across many artifacts at once" requires the preserving route
where a decision under silence is applied at scale. Rows 12 and 13 take it. Rows 14–16 cannot
preserve, and are worked below.

## The three rows that cannot simply be preserved

### Row 14 — `documents/*.html`: the destination format, at the wrong version, with dead references

4 files. All declare `<meta content="0.1.0" name="htmlspec-version">` and reference
`../../../../../assets/sha256-…` or `../../../../assets/sha256-…`, which resolve to nothing
inside `openspec/` — the store they were written against was deleted by `86c33f4`
("Migrate the documents and delete the duplicated asset trees").

Three dispositions, none clean:

| Disposition | What it costs |
| --- | --- |
| carry verbatim | imports 4 documents with dangling asset references and a stale contract version into a project whose asset topology check exists to report exactly that |
| convert as source | re-does a hand-migration that already happened, and the source is already the destination format |
| drop | discards the project's own first attempt at what this change automates |

**Route taken: preserve as evidence, and report.** The files are carried byte-identical into the
migrated change's `evidence/`, referenced from the change's design as what they are — a prior
hand-migration of the same content. This keeps `contract-authority`'s preserving route,
satisfies "Evidence crosses unchanged", and avoids importing dangling references into
`htmlspec/`. **This is the agent's choice, not the project's requirement**, and it is the
cheapest to reverse: a later change that wants them as documents re-points their asset
references and moves them.

### Row 15 — `.openspec.yaml`: dropped, with the `created:` disagreement recorded

8 files, each holding `schema: spec-driven` and `created: <date>`. `schema` names the source
format's workflow and has no meaning in the destination. `created:` is a date, and a change's
date lives in its archive directory name.

**In all 8, `created:` disagrees with the directory name.** Every one reads `created:
2026-08-24`, including `2026-09-06-distinguish-a-requirement-from-an-example` and
`2026-08-10-html-spec`. The field is unmaintained — copied forward rather than written per
change — so it is not a fact worth carrying, and carrying it would import 8 wrong dates.

**Route taken: dropped, reason stated.** The plan records that `schema` has no destination and
that `created:` disagreed with the archive directory name in all 8 cases. Distinct from row 9:
`config.yaml`'s `context` and `rules` are **absent** in this repository (both are commented-out
examples), whereas `.openspec.yaml`'s fields are **present and dropped**. That is the
distinction the plan requirement demands, and this repository supplies both sides of it.

### Row 16 — `## RENAMED Requirements`: no destination state exists

The destination's `changeState` enum is exactly `["added", "modified", "removed",
"unchanged"]`. Two delta files record a rename with `- FROM:` / `- TO:` heading pairs:

- `2026-08-17-project-bootstrap-cli/specs/toolchain-packaging/spec.md:1`
- `2026-08-30-finish-the-archival/specs/toolchain-packaging/spec.md:310`

Both rename the same requirement, successively: *A single command exposes validate, format, and
init* → *…validate, format, bootstrap, and create* → *The command surface names its operations
as OpenSpec names them*.

A rename states that one requirement **is** another under a new heading. Mapping it to
`modified` loses the identity link — and identity is precisely what task 4.7's derivation table
is keyed on, since OpenSpec addresses a requirement by heading text while the destination
addresses it by a `req-` identifier. A rename is the one case where the heading changed and the
requirement did not, so it is the case the derivation table exists to handle and the case its
input cannot express.

**Route taken: recorded as an open question, with the change that closes it named.**
`contract-authority`'s third route: "Where a reversible choice would mislead because the
question is large, record it as open and name the change that would settle it." A `renamed`
state is a schema change — a new `changeState` value is a contract-vocabulary addition
affecting validation, the formatter's attribute ordering, and the presentation rules. Deciding
it inside a migration is settling a contract question in a change whose subject is something
else, which is the fault the layout requirement was written against.

For this migration: the requirement carries `data-change="modified"`, and the rename is stated
in prose in the migrated specification, naming both headings. The plan records that the source
stated a rename and the destination has no state for it. **The change that closes it: one whose
subject is the `changeState` vocabulary.**

## Confirming the remaining lossy rows against the code

Task 1.4 names three code sites. One was wrong (fault 1). The other two hold.

**`ProjectRoot::assets()` derives one store per project from the project root** — asserted at
`crates/htmlspec-cli/src/project.rs:157`, actually at **line 179**:

```rust
pub fn assets(&self) -> PathBuf {
    self.htmlspec().join(htmlspec_assets::store::DIRECTORY)
}
```

Derived from `self` — the project root — with no document in the expression. The module comment
(`project.rs:14–20`) records the measured defect this closed: 6 MB per change, because the
previous code derived the store from each document's own directory. Two tests pin it:
`the_store_location_does_not_depend_on_where_resolution_started` and
`every_change_in_a_project_shares_one_store`. **Confirmed** — the design's staging-outside-the-
destination decision rests on this and rests correctly.

**`write_document` computes the asset reference from each document's own directory** — asserted
at `crates/htmlspec-cli/src/commands.rs:574`, actually at **lines 577–583**:

```rust
let document_directory = path.parent().unwrap_or_else(|| Path::new("."));
let assets_directory = match assets_root { ... };
let reference = match relative_reference(document_directory, &assets_directory) {
```

**Confirmed** — so a staged tree at the wrong depth does require every reference rewritten, and
the mirror-the-destination decision holds. Line 574 is the comment above it, which is close
enough to be a citation rather than an error.

Both line numbers are off by 5 and 22. Only fault 1's was off by 1447 — the distance between a
function's position before and after the change that fixed it.

## What this audit changes

1. **The version row inverts an accepted requirement.** Tasks 1.4, 5.4, 5.5 and the design's
   "Version history is reconstructed from directory names" decision direct work the contract
   forbids. `VERSION` → `VERSION`, carried. Section 2's requirement text must not restate the
   directory-name rule.
2. The survey enumerates 16 entity kinds, not 6.
3. `renamed` is an open contract question, named and not filled.
4. `documents/*.html` cross as evidence — the agent's reversible choice, stated as such.
5. `.openspec.yaml` is dropped with its `created:` disagreement recorded, and is the
   present-and-dropped counterpart to `config.yaml`'s absent-in-the-source.
6. The design's README attribution is wrong and should cite the eight workflow bodies.
