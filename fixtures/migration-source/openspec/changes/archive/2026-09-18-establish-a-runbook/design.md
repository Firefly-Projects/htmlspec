## Context

The proposal settles what a runbook *is*: a fourth place beneath the project root holding a
procedure that is run repeatedly rather than archived, made of two documents of existing kinds.
It leaves this design four questions, and building the cookie recipe before writing any of it
answered three of them by measurement rather than by choice.

Three existing facts shape everything below:

- **`ProjectRoot` is the only thing that knows the layout.** `crates/htmlspec-cli/src/project.rs`
  holds `SPECS`, `CHANGES`, and `ARCHIVE` as private constants and exposes one accessor each.
  The module header states the discipline: the inner path is private, so no caller can widen a
  bare path into a project without checking, and `assets()` derives the store's location from the
  *project* rather than from any document — which is the whole fix for a defect measured at 6 MB
  per change.
- **`list` already has a sum type for what it enumerates.** `Enumerated` is `Changes(Vec<Listed>)`
  or `Specifications(Vec<String>)`, and its doc comment says why it is a sum rather than one
  `Vec<String>`: the two arms no longer carry the same thing, and sharing a vector would make the
  arm that has counts and the arm that cannot have them indistinguishable downstream.
- **A runbook's documents need no validation work, and this was verified rather than assumed.**
  The cookie recipe validates, validates under `--strict`, and formats canonically on the binary
  at `HEAD` with no modification. Evidence:
  `evidence/the-cookie-recipe-2026-09-17.md`.

## Goals / Non-Goals

**Goals**

- A project has a fourth place, at `htmlspec/runbooks/<name>/`, derived in one location.
- `htmlspec list --runbooks` enumerates what is there, discovered from the filesystem.
- The release procedure ships as a real runbook, extracted from the five archived `## N. Release`
  sections and from `archive.md`.
- The two hazards that motivate the whole change are recorded where something reads them.

**Non-Goals**

- The `run` operation, run records, and `htmlspec/runbooks/<name>/runs/`. `log-a-runbook-run`.
- A fifth artifact kind. The enum is closed and its guard test stays untouched.
- A per-runbook version or changelog. The proposal argues this at length from the
  accepted-specification analogy; the one exception is `carry-a-runbook-to-another-machine`'s.
- A workflow, a `/htmlspec:runbook` command, or a tenth `WORKFLOWS` row. This change must be
  provable without one, and it is: everything below is reachable from `list`, `validate`,
  `format`, and `new document`.
- Machine requirements and preflight. `carry-a-runbook-to-another-machine`.
- Inputs. The recipe hit the gap and recorded it; see the last section.

## Decisions

### The on-disk layout is `htmlspec/runbooks/<name>/{specification,tasks}.html`

```
htmlspec/
  specs/                       accepted specifications        (existing)
  changes/                     active changes                 (existing)
    archive/                   archived changes               (existing)
  runbooks/                    the fourth place               (new)
    <name>/
      specification.html       kind=specification: what it is for, what its history taught
      tasks.html               kind=tasks: the ordered procedure
  assets/                      the project's one store        (existing)
```

**The file names are the change directory's names, deliberately.** `commands.rs` already fixes
`specification.html` in `CHANGE_SPECIFICATION` and `tasks.html` in `CHANGE_DOCUMENTS`, and a
runbook using different spellings for documents of the same two kinds would mean a reader
learning two vocabularies for one thing. Nothing derives a runbook's file names from those
constants — that would couple a runbook's shape to a change's for no reason — but the spellings
agree.

**A runbook is a directory even though it holds a fixed pair of files.** `<name>/tasks.html`
rather than `runbooks/<name>.html` for two reasons, one now and one later: the two-document split
is forced by `artifact-kinds` (*A specification states the settled contract as built* — a
specification SHALL NOT carry task units, the only placement rule in the vocabulary), so there
were always going to be two files; and `log-a-runbook-run` puts `runs/` beneath the runbook,
which needs a directory to be beneath.

**The name is the directory name, and nothing else records it.** The precedent is a change, whose
name is its directory name and which is renamed by moving it. `project-bootstrap`'s
*Archival requires the project version to advance* refuses to read a version out of a directory
name because a version is a value a gate compares; a name is not compared to anything, it is how
the thing is addressed.

**Asset references resolve at this depth with no new mechanism.** Measured: `htmlspec new
document tasks htmlspec/runbooks/<name>/tasks.html` writes
`href="../../assets/sha256-…/htmlspec.css"`, which is correct — `runbooks/<name>/` is two levels
below `htmlspec/`, exactly as `changes/<change>/` is. The store is derived from the project by
`ProjectRoot::assets()`, so a runbook shares the one store and `asset/duplicate` has nothing to
report.

### The location is named in `project.rs`, as a private constant with one accessor

```rust
/// Where a project keeps its runbooks.
const RUNBOOKS: &str = "runbooks";

impl ProjectRoot {
    /// Where the project's runbooks live.
    pub fn runbooks(&self) -> PathBuf { self.htmlspec().join(RUNBOOKS) }

    /// The directory a runbook of this name occupies.
    pub fn runbook(&self, name: &str) -> PathBuf { self.runbooks().join(name) }
}
```

Exactly the shape `specs()`, `changes()`, `archive()`, and `change(name)` already have. The
constant is private and the accessor is public, so the layout stays a fact one module states.

**`establish` gains `runbooks()`, and this is the one substantive question in the pair.**
`ProjectRoot::establish` currently creates `htmlspec/`, `specs/`, and `archive/` — note it does
not create `changes/`, because `archive()` is beneath it and `create_dir_all` makes both.
Bootstrapping now creates `runbooks/` too, empty.

The alternative is creating it lazily, on the first runbook. That was rejected because
`project-bootstrap`'s *A project has a declared root holding one asset store* requires that
bootstrapping produce "the project root … with places for" each thing the directory holds — a
place that does not exist until something is in it is not a place a contributor can find. A
contributor who runs `init` and lists the project should see where a runbook goes. The cost is
one empty directory in a fresh project, which is what `specs/` and `archive/` already are.

**`migrate.rs` has its own `SPECS`/`CHANGES`/`ARCHIVE` constants and gains nothing.** Those
describe an *OpenSpec* project's layout, which has no runbooks. Adding one there would make the
survey claim a source kind that does not exist. This is load-bearing rather than tidy — see the
finding below.

### `list --runbooks` prints one row per runbook: its name and which documents it has

```
$ htmlspec list --runbooks
chocolate-chip-cookies  specification, tasks
release                 tasks
```

**Discovery is a directory read, not a document walk.** Each immediate subdirectory of
`htmlspec/runbooks/` is a runbook, named for the directory. This mirrors `active_changes`, which
reads `changes/` and filters `archive` by name — a runbook needs no such filter, because nothing
else lives under `runbooks/`.

The `--specs` arm is the other available model and it is wrong here: it calls
`discover::documents(&project.specs())` and prints every `.html` path beneath. Applied to
runbooks that prints `release/tasks.html` — a file list rather than a list of runbooks, so two
documents of one runbook read as two entries. **What is being enumerated is the runbook**, and
the documents are what it holds.

**Each row states which of the two documents are present, and this is the selection
information.** `list`'s doc comment states the principle it was built on: a list of names "can
be selected from on no basis but order", so the enumeration carries what a selector needs. For a
change that is the completion fraction. For a runbook it is which documents exist — a runbook
with no `tasks.html` has no procedure, and that is the fact worth seeing before picking one.

**It deliberately prints no task fraction.** The canonical procedure's tasks stay `todo` forever,
because nobody works them — a run copies the document and advances the copy. So the column would
read `0/9` on every runbook at every moment, and a column that never moves teaches a reader to
ignore it. `evidence/the-status-gap-2026-09-17.md` measures the fraction being available and
correct; it is not printed because it says nothing.

**A third arm on `Enumerated`, not a second boolean pair.** `Enumerated::Runbooks(Vec<Runbook>)`
joins `Changes` and `Specifications`, and `label()` and `is_empty()` gain their arms — so the
empty case prints `no active runbooks` through the path that already exists. The flags stay two
independent `bool`s on the CLI, matching `--specs`, with `--runbooks` winning if both are passed
and that being the whole of the interaction; a three-way enum in the argument surface would be a
different flag shape from the one the proposal specifies.

The structured payload gains `{"runbooks": [{"name": …, "documents": [...]}]}`, keyed by the same
word the flag uses.

### A runbook's documents are validated by the existing path, and this was proven not assumed

**No new rule, no schema change, no projection change, and nothing in
`crates/htmlspec-validate/src/rules/`.** A runbook's `tasks.html` is a `tasks` document and its
`specification.html` is a `specification` document. `artifact-kinds` states what each owes and
validation already checks it.

The proof is the cookie recipe, built before any of this was written, on the unmodified binary:

```
$ htmlspec validate   htmlspec/runbooks/chocolate-chip-cookies/tasks.html   → accepted, 0 diagnostics
$ htmlspec validate --strict  …                                            → accepted, 0 diagnostics
$ htmlspec format --check     …                                            → is canonical
9 task units, 5 citation links, all preserved through formatting
```

**Two properties fall out of the two kinds and neither needed asking for.** A runbook whose
`tasks.html` carries no task unit is reported by `kind/no-tasks`, because a procedure with no
steps is not a procedure. A runbook whose `specification.html` carries a task unit is reported by
`kind/task-in-specification` as an *error* — which is the forced split doing work rather than
merely being obeyed: the procedure cannot drift into the canonical document, because validation
refuses it.

The citation links survive because of a deliberate line at
`crates/htmlspec-validate/src/rules/safety.rs:37`: `href` is unchecked on `a` and checked on
`link`, since a hyperlink is a citation and a stylesheet loads. Somebody drew the line a cited
recipe needs before there was a recipe.

### The release runbook's content is extracted, and the hazards are its reason for existing

`specification.html` states what the runbook is for and what its history taught.
`tasks.html` carries the four steps **in the order the hazard forces**:

1. Advance the workspace version and relock offline.
2. Regenerate the installed instruction files.
3. Write the `CHANGELOG.md` entry.
4. Confirm `scripts/check-release` passes. `VERSION` is written at archival.

**The five archived sections all state regenerate *before* version, and all five are wrong about
it.** `2026-08-24-report-in-the-contributors-terms` §6, `2026-08-24-pass-the-description-through`
§8, `2026-09-06-distinguish-a-requirement-from-an-example` §6,
`2026-09-11-migrate-an-openspec-project` §9, and
`2026-09-15-catch-a-templates-placeholder-prose` §7 each put regeneration first, and the last
three each discovered at run time that the version bump restaled every installed file and
regenerated again afterwards. **So the runbook's order is the correction, not the transcription.**
That is the change working: five records of one procedure, and the artifact that replaces them
states the thing all five had to learn.

The two hazards are `specification.html`'s requirements rather than notes in the tasks, because
a note in a tasks document is what the archive already held and nothing read:

- **Advancing the version restales every installed file.** `generatedBy: htmlspec <version>` is
  written from `CARGO_PKG_VERSION` at run time, so regeneration after a version bump is never a
  no-op — it rewrites one line in every installed file. The `installed` stage of `scripts/check`
  is what says so, and it found this on its first release.
- **`htmlspec init` inside a repository that is already a project bootstraps a stray `htmlspec/`
  and a `spec.nix` at `0.0.0`**, which fails `self-assets` with nine duplicate-asset errors.
  Reproduced for this change rather than quoted:
  `evidence/the-location-is-blocked-2026-09-17.md`.

### The runbook lives at the repository root, and that is the agent's choice under silence

**This repository cannot hold its own runbook where the contract says a runbook goes**, and the
reason has nothing to do with runbooks: creating `htmlspec/` at this repository's root makes
`ProjectRoot::resolve` succeed, which turns the repository into a project, which turns
`self-assets` red by comparing across the four stores its conformance fixtures deliberately
carry. One `mkdir` and no files is enough. Measured in
`evidence/the-location-is-blocked-2026-09-17.md`.

`openspec/runbooks/` fails a different check —
`every_file_in_this_repository_classifies` reports the document as classifying as no known
OpenSpec kind — and the honest fix there is not available, because teaching `migrate.rs` about
runbooks would make an OpenSpec survey claim a kind OpenSpec does not have.

So this repository's release runbook is at **`runbooks/release/`, at the repository root**, which
passes both. The contract is silent on where a repository that is not an html-spec project keeps
a runbook, and this is `contract-authority`'s second route: choose reversibly and own the choice.
**The choice is the agent's rather than the project's** — two documents move if a contributor
disagrees, and nothing derives the location from anything. The exact precedent already exists:
this repository records a predecessor in `.openspec.yaml` rather than in the `FOLLOWS` file
`project.rs` defines, for the same reason.

## Risks / Trade-offs

- **The location this change specifies is not the location this repository dogfoods.** The
  contract, the CLI, and the scratch-project verification all say `htmlspec/runbooks/`; the real
  runbook is at `runbooks/`. Accepted and stated in the design above rather than discovered by a
  reader. What closes it is migrating this repository to `htmlspec/`, which is
  `migrate-an-openspec-project`'s unfinished half.
- **A runbook has nowhere to put its inputs** → not solved, deliberately. See the finding below.
- **`list --runbooks` says nothing about whether a runbook's documents are valid.** So does
  `list --specs`, and for the same reason: `validate` is the operation that answers that, and an
  enumeration that validated every document it named would make listing cost a parse per file.
- **An empty `runbooks/` in every freshly bootstrapped project.** Accepted, matching `specs/` and
  `archive/`. A place a contributor cannot find is not a place.
- **Nothing yet reports a runbook directory holding neither document.** `list --runbooks` shows
  it with an empty document list, which is visible but not reported. A rule whose subject is a
  runbook's completeness is `log-a-runbook-run`'s territory — it is the same shape as a run whose
  records do not satisfy its log contract, and building half of it here would fork that design.

## Migration Plan

Additive throughout. No existing document changes meaning, no fixture changes state, no
diagnostic is added or reworded, and no existing command's output changes: `list` with no flag
and `list --specs` print exactly what they printed. A project that never creates a runbook gets
one empty directory at its next `init`, which `init` is already required to be safe to re-run.

`document.schema.json`, `TEMPLATES`, `assets/vendor.lock.json`, and
`crates/htmlspec-validate/src/rules/` are untouched, which the proposal states as a checkable
claim and which held.

## The finding this change was asked to test, and the answer

**A change that produces no deliverable had not been tested against this workflow's rhythm.**
The proposal names it open and says the result is worth recording either way. This change ran
the experiment and the result is in `evidence/the-rhythm-2026-09-17.md`. The short form: the
rhythm fit, and the one place it strained was not where the proposal expected.

**A runbook has nowhere to put its inputs, and building the recipe is what found it — again.**
The ingredients became a `<table>` in a document section titled "What the procedure consumes",
with a paragraph in the document saying that is a finding rather than a design. Three options
were available and the reasoning for rejecting the other two is recorded in
`evidence/the-cookie-recipe-2026-09-17.md`. **No inputs mechanism was invented**, per the
proposal: the shape of the answer needs its own argument, and the recipe's equipment list is why
— a stand mixer is closer to a machine requirement, which is
`carry-a-runbook-to-another-machine`'s subject, than to an ingredient.
