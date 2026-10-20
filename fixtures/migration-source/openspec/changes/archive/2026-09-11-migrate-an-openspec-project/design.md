## Context

An OpenSpec project has a fixed layout. This repository's is representative and is the test
case. The counts below were measured at commit `f57f640` and recorded in
`evidence/the-source-inventory-2026-09-07.md`; an earlier draft of this design carried counts
from a stale snapshot, and every one of them was wrong by the time the change was implemented.

```text
openspec/
├── config.yaml                                  schema, plus optional context, rules, operations
├── specs/<capability-path>/spec.md              9 capabilities, 97 requirements, 401 scenarios
└── changes/
    └── archive/<date>-<name>/
        ├── proposal.md, design.md, tasks.md     prose, no imposed structure
        ├── specs/<capability-path>/spec.md      28 delta files across 16 changes
        ├── evidence/**                          75 transcripts and measurements, some nested
        ├── documents/*.html                     4 already-converted HTMLSpec documents
        ├── .openspec.yaml                       per-change schema marker and created date
        └── VERSION                              the version the change shipped in
```

193 files in total. An HTMLSpec project has a different layout, and the differences are not
cosmetic. Each row below cites the accepted requirement that fixes the destination, by
requirement heading rather than by capability, as `contract-authority` requires. **Rows citing
nothing are marked silent**, and the full sixteen-row table is in
`evidence/the-mapping-table-and-its-audit-2026-09-07.md`.

| OpenSpec | HTMLSpec | Why they differ |
| --- | --- | --- |
| `specs/<capability-path>/spec.md` | `htmlspec/specs/<capability-path>/spec.html` | **Silent.** No requirement states a layout beneath `htmlspec/specs/`, so the source's nested path is preserved |
| N delta files per change | one `specification.html` | `artifact-kinds` — "Archival produces the specification and then updates the accepted contract": a change directory holds **at most one** specification artifact |
| `## ADDED Requirements` | change state on the unit | `document-schema` — "Semantic units declare a change state": the delta is expressed per unit rather than per file |
| `## RENAMED Requirements` | *no state exists* | `changeState` is exactly `added`, `modified`, `removed`, `unchanged`. **An open question, named rather than filled** |
| `VERSION` file | `VERSION` file, carried unchanged | `project-bootstrap` — "Archival requires the project version to advance": the version **SHALL NOT** be derived from an archive directory's name |
| `config.yaml` context, rules, operations | *nothing* | `project-bootstrap` — "A project declares its name, version, and expected toolchain": the declaration **SHALL NOT** carry authoring rules or project context |
| `.openspec.yaml` schema and created date | *nothing* | **Silent.** No requirement reaches a per-change source config; dropped with the reason recorded |
| `documents/*.html` | carried as evidence | **Silent.** Already the destination format, at contract version 0.1.0, with asset references resolving to nothing |
| no asset store | `htmlspec/assets/`, content-addressed | `project-bootstrap` — "A project has a declared root holding one asset store": documents render offline, so the runtime is part of the project |

Several of those rows are lossy, in both directions, and every one of them is a decision
somebody has to make about a specific file. That is what makes this agent-driven rather than a
transform.

## Goals

- An existing OpenSpec project reaches HTMLSpec with its argument, its reasoning, and its
  evidence intact — not only its requirements.
- Every source entity is accounted for, and a contributor can see the accounting before
  anything is written.
- A failed or abandoned migration leaves the repository exactly as it found it.

## Non-Goals

- A converter, a second source format, deleting the source, round-tripping. See the proposal.

## Decisions

### The split: two exact operations, and judgment in between

The project's standing division is judgment where judgment is needed, mechanical checking
where a rule is exact. Applied here, three things are exact and everything else is not.

**Exact — enumerating the source.** OpenSpec's layout is fixed, so listing every capability,
change, delta, evidence file, and version is enumeration with no reading involved.
`htmlspec migrate survey` does it, emits a structured payload, and writes nothing.

**Exact — checking the plan covers the survey.** Comparing a set of accounted-for entities
against a set of found ones is set difference. `htmlspec migrate promote` refuses while the
difference is non-empty.

**Exact — the existing checks.** `htmlspec validate`, `htmlspec format --check`, and
`htmlspec assets` already exist and already say what a conforming project is. The migration
adds no check of its own; it runs those three over the staged tree.

**Not exact — everything in the middle.** Which paragraph of a design states an obligation.
Whether a three-column comparison should stay a table or become a diagram. Which of five
delta files' requirements belong under which heading of one specification. Whether a
proposal's "Why" section is the value argument or the need. `artifact-kinds` already settled
that these have no grammar, and a migration does not get a different answer because it does
them in bulk.

So `htmlspec migrate` has exactly two subcommands, `survey` and `promote`, and a `migrate`
workflow whose instructions carry the reading and the writing. The alternative — one command
that migrates — was rejected because it is the extractor `artifact-kinds` forbids, and it
would produce 161 documents that pass every check and misstate what they came from.

### The order, and why the checks are last

The workflow's stages, in order:

1. **Survey.** Enumerate. Write nothing.
2. **Read.** Open every source file and record what each one is *for*, in one line. Not what
   it says — what it is doing in the project. This is the pass that finds the three
   proposals arguing the same defect class, and the design whose middle section is a state
   machine written as prose.
3. **Gather the evidence.** Pull the 75 evidence files, the VERSION values, the archive dates,
   and the changelog into the record, because these are the facts about *why* the project is
   shaped this way and they are the first thing a bulk conversion loses.
4. **Plan.** Write the `tasks` document: one task per surveyed entity, its destination, its
   disposition, and — for anything being folded or dropped — the reason. Stop here and report.
5. **Stage.** Build the whole migrated project in the staging location.
6. **Check.** `validate`, `format --check`, `assets`, over the complete staged tree.
7. **Promote.** Move it into place, in one step.

Stages 1 through 4 are the ones that carry the migration's value, and they produce nothing a
validator can look at. That is deliberate and worth saying plainly: the artifact of the first
four stages is a plan a person reads.

**The checks are stage 6 rather than continuous, and this is a real choice with a cost.**
`htmlspec new document` validates each document as it writes it, so an agent staging 100
documents gets one verdict per document for free and could stop at the first failure. The
migration does not stop, for two reasons.

The first is that a partial tree is worse than no tree. Stopping at document 40 leaves 39
migrated documents, 61 unmigrated ones, and a contributor holding both. Staging plus a single
promotion means the repository is only ever in one of two states.

The second matters more. A mapping defect is systematic. If the rule chosen for turning
`#### Scenario:` bullets into scenario units is wrong, it is wrong 401 times in the accepted specifications alone and 545 times across the deltas. Halting on the
first occurrence presents a systematic defect as a single document's problem, and the agent
repairs that document by hand and continues into 309 more. Completing the tree first makes
the shape of the failure visible — "every scenario in every specification reports the same
diagnostic" is a different finding from "specification 3 is broken", and it is the finding
that leads to the actual repair.

An agent may of course check a document while drafting it. What is forbidden is a migration
that *ends* at the first failure rather than at the last.

### Staging is outside the destination, and mirrors it exactly

The staged tree is a complete HTMLSpec project: `spec.nix`, `htmlspec/specs/`,
`htmlspec/changes/`, `htmlspec/changes/archive/`, and one `htmlspec/assets/` store. It is not
a bag of files.

**Complete, because the checks must run in it.** `ProjectRoot::resolve` ascends looking for a
child `htmlspec/` directory and stops at the repository boundary. A staging area that is not
a resolvable project cannot be validated, which would push the checks past promotion and make
promotion the risky step.

**Outside the destination, because the destination is a project.** A staging tree under the
target repository would put a second asset store in a repository that is supposed to hold
exactly one, which is the defect `htmlspec assets` exists to report and the reason
`ProjectRoot::assets()` derives the store's location from the project root rather than from
each document.

**Mirroring the destination's depth, because asset references are relative.**
`write_document` computes the reference from the document's own directory to the project's
store (`crates/htmlspec-cli/src/commands.rs:574`), so `htmlspec/changes/x/proposal.html`
references `../../assets/` and `htmlspec/specs/y.html` references `../assets/`. Stage a
document at the wrong depth and promotion has to rewrite every reference it moves, which is
a second transformation applied after the checks that would have caught its mistakes.

So the staging root holds the same relative paths the destination will, and promotion is a
move.

### The plan is a `tasks` document

A plan is a list of work items with states. That is the definition of the `tasks` kind, which
already requires each task to be individually addressable so that settling one is a change to
one unit rather than to a list — which is exactly what a contributor rejecting one entity's
disposition needs.

The alternative was a fifth artifact kind, `plan`. Rejected: `artifact-kinds` states the set
of four is closed and that a document declaring a kind outside it is invalid. Reopening a
closed vocabulary for one operation's working record would mean every consumer of the kind
set — validation, the templates, the archival flow — grows a case for a kind that appears
once per project, ever.

The second alternative was a structured file, `migration-plan.json`. Rejected for the reason
this project chose HTML in the first place: the plan is the document the contributor reviews,
and a JSON file is not reviewed, it is diffed.

### The delta collapse is the hardest mapping, and it is the one with no rule

An OpenSpec change carries N delta files, one per capability it touches, each with `## ADDED
Requirements`, `## MODIFIED Requirements`, or `## REMOVED Requirements`. An HTMLSpec change
carries at most one `specification.html`, and change state lives on the unit as
`data-change="added"`.

For fourteen of this repository's sixteen archived changes N is 1 or 2 and the collapse is nearly
mechanical. For `2026-08-10-html-spec` it is 5, and those five span every capability the project
then had: 47 requirements and 138 scenarios into one document.

There is no rule to write for the arrangement and this change does not write one. The workflow
states the constraint — one specification, change state per unit, every delta's content accounted
for including the parts that are not requirements, identifiers slugged from full headings and
recorded per (change, heading) — and leaves the arrangement to the agent, which is the same
latitude `archive.md` already gives for producing a specification from a completed change.

What the constraint has to say beyond arrangement came out of writing this case by hand, and is
recorded in "The collapse's hard part is not arrangement" above: the `## Purpose` paragraphs, the
within-change identifier collision, the per-change keying of the derivation table, and the omitted
verification units. Those are the parts a rule about requirements alone would lose.

### Where the destination's contract is silent, the source's shape is preserved

An earlier draft of the table above mapped `specs/<cap>/spec.md` to `htmlspec/specs/<cap>.html`
and justified it: a directory holding one file is a directory OpenSpec needs and HTMLSpec does
not. Both halves were wrong, and the way they were wrong is the reason this decision exists.

**Nothing in the accepted contract states a layout beneath `htmlspec/specs/`.**
`project-bootstrap` — "A project has a declared root holding one asset store" — says only that
the project directory holds its accepted specifications, its active changes, and its archive,
"each in its own place". `ProjectRoot::specs()` (`crates/htmlspec-cli/src/project.rs:160`)
resolves `htmlspec/specs` and constrains nothing below it. `archive.md:255` directs an agent at
`<accepted-specs-dir>` without naming a file.

The flat spelling `htmlspec/specs/<name>.html` appears in **eight** places: once in each of the
eight installed workflow bodies. In every one it is the worked example of a sentence that *owes
a requirement* (`crates/htmlspec-assets/instructions/archive.md:81`):

> - **Owes one.** "Accepted specifications live at `htmlspec/specs/<name>.html`." Nothing else in
>   the sentence carries that claim.

So the spelling that produced this fault is, in the current tree, the project's own illustration
of a claim requiring a citation — and the fault was to read the illustration as the claim.

An earlier draft of this decision attributed the spelling to the README instead. It is not
there: `grep -n 'htmlspec/specs' README.md` returns nothing. The README's layout block
(`README.md:78–90`) does show a flat `specs/theme-selection.html`, which is where the impression
came from, but the sentence describing its position was invented. A design that misattributes
its own founding evidence is the same defect at one remove, and it is corrected here rather than
quietly.

**And the justification was false about the source.** `<capability-path>` is a path rather
than a wrapper — the workflow bodies give `identity/user-auth` as an example and instruct an
agent to preserve an existing capability's full path. Flattening it to a filename either
discards the hierarchy or encodes it as `identity-user-auth.html`, which is a naming
convention invented during a migration and enforced by nothing.

So the migration takes no position. Where the destination's contract states a layout,
migration conforms to it; where the contract is silent, migration preserves the source's
shape. `openspec/specs/identity/user-auth/spec.md` becomes
`htmlspec/specs/identity/user-auth/spec.html`. Depth costs nothing: each document's asset
reference is computed from its own directory, so a specification three levels down references
`../../../assets/` and resolves like any other.

The rule generalizes past this row, which is why it is stated rather than fixed in place. A
migration is the worst moment to settle an open question about the destination format. It
settles it at bulk, under a deadline, for a hundred files at once, in a change whose subject
is something else — and every later reader finds a convention with a hundred instances behind
it and no requirement anywhere. If the flat spelling is right, a change whose subject is the
layout should state it, and this migration will follow it the day it exists.

### Identifiers are derived once and reused everywhere

OpenSpec addresses a requirement by its heading text. HTMLSpec addresses it by a `req-`
identifier that must be unique within its document and stable across the project, because
`data-refs` on verification units and cross-references between changes point at it.

The migration derives each identifier once, from the requirement's full heading, and records the
derivation in the plan. Every later reference — a delta in an archived change naming a
requirement that also appears in an accepted specification, a verification unit naming what it
checked — resolves through that one table. Deriving independently at each site produces
`req-a-project-has-a-declared-root` in one document and `req-declared-root` in another, and
nothing catches it because both are locally valid.

**The table is keyed on (change, source heading) rather than on the heading alone**, and this is
the correction the hand-written collapse forced. A renamed requirement has different headings in
different changes while being one continuous obligation: the requirement that began as *A single
command exposes validate, format, and init* was renamed twice and is now *The command surface
names its operations as OpenSpec names them*. Each archived specification states what its own
change established, per `artifact-kinds` — "A specification states the settled contract as
built" — so that obligation legitimately carries four identifiers across the migrated project.
Keying on the heading alone would either collapse them into one, misstating three changes, or
report three collisions that are not collisions.

**Deriving from the full heading is load-bearing rather than incidental.**
`deterministic-formatting`'s "Formatting can be checked without writing" and
`toolchain-packaging`'s "Formatting can check without writing" are distinct obligations whose
full-heading slugs differ; a rule that truncated or dropped stop-words would map them together
and manufacture a collision the next rule then fires on.

Two requirements whose full headings derive the same identifier is a collision the plan reports
rather than resolving by suffix. A suffix would silently make two obligations look like versions
of one.

### Version history is carried in the file the gate reads, and the mapping is the identity

`VERSION` becomes `VERSION`. Both formats record an archived change's version in a file of
that name beside its documents, so this row is the identity and there is nothing to
reconstruct.

**An earlier draft of this design said the opposite, and it was the worst kind of wrong.** It
had migration compose each archive directory as `<date>-<version>-<name>`, justified by
"`last_archived_version` scans directory names (`crates/htmlspec-cli/src/commands.rs:1352`)".
Two accepted requirements forbid exactly that:

`project-bootstrap` — "Archival requires the project version to advance"
(`openspec/specs/project-bootstrap/spec.md:57`):

> The last archived version SHALL be read from the version each archived change records, in the
> form `toolchain-packaging` requires. **It SHALL NOT be derived from an archive directory's
> name.** A directory name is a display string that is shortened and renamed, and a gate derived
> from one depends on a convention nothing enforces — which is how this gate came to compare
> against a value no project ever had.

`toolchain-packaging` — "An archived change is named for its date and refuses to collide"
(`spec.md:403`) states archival names the directory "for the date of archival followed by the
change's name" — no version component. So the composed name contradicts a stated requirement
about what an archived directory is called, not merely the gate's implementation.

**The citation was true once, which is why it survived review.** At `cf11132^` that function
sat at exactly line 1352 and read
`archived_version(&entry.file_name().to_string_lossy())` — it did scan directory names.
`cf11132` is titled *"Finish the archival, and fix the gate that never fired"*, and the
requirement quoted above is the contract that change established. Today the function is at
`commands.rs:2799` and calls `recorded_version`, which reads the file. The design cited real
code from before the fix whose entire subject was this defect.

The spelling `<date>-<version>-<name>` comes from `README.md:88`, under a heading reading
"Proposed project layout", below `**Status:** early design proposal`. `contract-authority` —
"The project's obligations are what its accepted specifications state" — makes that
non-normative, and its scenario "A layout stated only in a README" is this case exactly. **This
is the second README-as-contract fault in this change**; the first is recorded below as the
reason the layout-preservation requirement exists. One change producing the same fault twice,
the second time inside the decision written to prevent it, is the argument for that requirement
rather than a footnote to it.

The declaration's `version` is set to the greatest version found among the migrated archives —
`0.10.0` in this repository — not to something higher. `htmlspec archive` refuses unless the
declared version is ordered *after* the last archived one, so a migrated project's next
archival is correctly forced to advance, which is the behavior a project that just migrated
should have.

Where a source change records no version, the migration does not invent one. `recorded_version`
returns `None` for an absent file, an unreadable one, and contents that do not parse, and
`last_archived_version` folds that into a `max`, so a change with no version cannot lower the
answer. The plan records that the source had none. This repository has no such case — all 16
record one — so task 3.7 proves it against a constructed project.

**A migrated project inherits a version collision, and must carry the baseline that explains
it.** `toolchain-packaging` — "An archived change records the version it shipped in"
(`spec.md:310`) — requires recorded versions to be distinct, with a stated exception: where
several changes were archived under one version before the obligation existed, the changelog
carries a single baseline entry naming them. **Seven of this repository's changes record
`0.1.0`**, and `scripts/check-release` holds `BASELINE = (0, 1, 0)` for them. So migration
carries the changelog and its baseline entry, and a migrated project whose changelog lost that
entry fails its own release check the moment it arrives — with a diagnostic about version
distinctness that names seven changes and explains none of them.

### Four entity kinds the mapping table originally missed

The table above has nine rows because the audit added four entity kinds the first draft had none
for. They matter more than their file count suggests: **a survey enumerating only the original
six rows reports full coverage over a set that omits 15 files**, so the gate in stage 4 would
pass while they went missing. A coverage check is only as good as the enumeration it compares
against, which is why the survey's entity set is derived from a measurement of the source rather
than from this table.

**`documents/*.html` — the destination format already, and unusable.** Two archived changes carry
four HTMLSpec documents from an earlier hand-migration. All four declare contract version 0.1.0
and reference an asset store by a relative path that resolves to nothing, because `86c33f4`
deleted the store they were written against. Every obvious disposition is wrong: carrying them
verbatim imports dangling references into a project whose asset-topology check exists to report
exactly that, converting them re-does work already done, and dropping them discards the
project's own first attempt at what this change automates. So they cross as **evidence**,
byte-identical, referenced from the migrated change's design as what they are. This is the
agent's reversible choice under silence rather than the project's rule, and it is stated as such:
a later change that wants them as documents re-points their references and moves them.

**`.openspec.yaml` — present and dropped, which is the counterpart case.** Eight files holding
`schema: spec-driven` and `created: <date>`. The schema names the source format's workflow and has
no destination. The date is a fact the destination records in the archive directory name — and in
all eight it **disagrees** with that name, every one reading `created: 2026-08-24` including
`2026-09-06-distinguish-a-requirement-from-an-example`. The field was copied forward rather than
written per change, so carrying it would import eight wrong dates. Dropped, with the reason and
the disagreement recorded.

**`## RENAMED Requirements` — an open contract question, not a mapping choice.** Two delta files
record a rename with `- FROM:` / `- TO:` heading pairs, and the destination's `changeState` enum
is exactly `added`, `modified`, `removed`, `unchanged`. A rename states that one requirement *is*
another under a new heading, and mapping it to `modified` loses the identity link — which is
precisely what the identifier derivation is keyed on, since the source addresses a requirement by
heading text and the destination by a `req-` identifier. A rename is the one case where the
heading changed and the requirement did not.

`contract-authority` — "Silence is recorded and not filled" — offers three routes, and this takes
the third: record the question as open and name the change that closes it. A `renamed` state is a
contract-vocabulary addition affecting validation, the formatter's attribute ordering, and the
presentation rules, and settling it inside a migration is settling a contract question in a change
whose subject is something else — the fault the layout requirement exists to prevent. For this
migration the requirement carries `data-change="modified"` and the rename is stated in prose
naming both headings. **The change that closes it is one whose subject is the `changeState`
vocabulary.**

**Nested evidence.** `evidence/transcripts/` holds four files in one change. A survey globbing
`evidence/*` finds nine where thirteen exist, so the enumeration recurses.

### The collapse's hard part is not arrangement

Working `2026-08-10-html-spec`'s five deltas by hand before writing anything
(`evidence/the-collapse-written-by-hand-2026-09-07.md`) changed what this design says about the
collapse. Arrangement — the thing the decision below calls the open question — turned out to be
the easy part: 47 requirements in a flat sequence is unreadable, and the capability boundary is
the only structure the source expresses, so grouping by capability is forced rather than chosen.

The genuinely hard part is that **each delta carries a `## Purpose` paragraph, and those are not
requirements.** Five of them, and they have nowhere to go: one document has one masthead and five
statements cannot sit in it. They are also the current definitions of five of the project's nine
capabilities, so dropping them loses content the accepted specifications still carry. They become
the paragraph introducing each capability's group — which follows from the grouping rather than
being a separate decision. The reason this matters for the requirement text is that **a rule
written about requirement units alone would drop all five silently**, so the requirement states
that a delta's non-requirement content is accounted for too.

Three further findings the hand-write produced, each of which the requirement text has to carry:

**A collapse can collide identifiers within a single change.** `deterministic-formatting` states
"Formatting can be checked without writing" and `toolchain-packaging` states "Formatting can check
without writing"; both survive in the accepted specifications as distinct obligations. The
collapse is where collisions are most likely, because it is the only operation putting several
capabilities' requirements into one identifier namespace. The resolution is to slug the **full
heading** — a rule that truncates or normalizes aggressively manufactures the collision that the
report-rather-than-suffix rule then fires on.

**The derivation table is keyed on (change, source heading), not heading alone.**
`req-a-single-command-exposes-validate-format-and-init` is renamed twice by later changes, so one
continuous obligation legitimately holds four identifiers across the migrated project — three in
archived specifications and one in the accepted specifications. That is correct rather than a
defect: `artifact-kinds` — "A specification states the settled contract as built" — requires each
archived specification to state what *its* change established. The decision below reads as though
one obligation gets one project-wide identifier; for a renamed requirement it cannot, and should
not.

**Verification units are omitted, with the reason recorded.** A specification carries requirement,
scenario, evidence, and verification units, and the source format has no verification unit at all.
Manufacturing 47 of them from a tasks document invents a record. Evidence units are carried,
because the eight evidence files are real sources. A reader comparing a migrated specification
against a natively-archived one will notice the absence, so the plan states the reason rather than
leaving it to be inferred.

### `config.yaml` has no destination, and that is the interesting case

`openspec/config.yaml` carries `schema`, and optionally `context` (project background shown to an
agent), `rules` (per-artifact authoring constraints), and `operations` (advisory guidance for
apply and archive). **Three optional fields rather than two** — the survey reads the field set
from the file, because a list of two written into this design would have missed the third.

`project-bootstrap` — "A project declares its name, version, and expected toolchain"
(`spec.md:34`) — states that the declaration **SHALL NOT** carry authoring rules or project
context, and gives the reason: a field no command reads states nothing that can be wrong, so
nothing reports it when it is.

So there is nowhere for this content to go, and the migration must not pretend otherwise. The
plan names it as dropped and states the reason. Where the field is empty or absent — as in this
repository, where all three are commented-out examples and the file's one live key is `schema:
spec-driven` — the plan says that too, because "there was nothing there" and "there was something
and it is gone" are different facts and a contributor needs to know which one they have.

`schema` itself is dropped rather than absent: it names the source format's workflow schema, and
the destination has one artifact vocabulary rather than a choice among several.

**This repository supplies both sides of the distinction, but not from one file.** `config.yaml`
gives the absent side; its one dropped field is a scalar rather than the block of content the
rule is written about. The present-and-dropped side with real content is `.openspec.yaml`'s
`created:` in eight changes — content that existed, was read, and is gone. Task 3.7's
version-absence case is a third instance, and it needs a constructed project because all 16
archived changes here record a version.

This is the general rule the coverage gate enforces: an entity with no destination is reported,
never omitted.

### Risk: an agent that converts rather than migrates

**Accepted, and mitigated by the order rather than by a check.** Nothing mechanically
distinguishes a thoughtfully migrated document from a mechanically converted one. Both
validate. The mitigation is that stages 2 and 3 come before stage 5 and produce a plan the
contributor reads: an agent that skipped the reading pass writes a plan whose per-entity
lines say nothing, and that is visible to a person at a glance.

Stating the limit rather than implying a test could cover it follows what the last three
changes each concluded: guidance whose only verification is reading the output.

### Risk: 161 documents is a lot of judgment in one pass

**Accepted, with the plan as the pressure valve.** A migration of this size will get
individual documents wrong. The plan is reviewed before staging, promotion is one step, and
the source survives untouched, so a wrong call is corrected by fixing the plan and running
again rather than by unpicking a half-migrated tree.

### Risk: the staged tree diverges from the source while the plan is under review

**Accepted, unmitigated.** A contributor who edits `openspec/` between the survey and the
promotion gets a migration of what the survey saw. Re-running the survey is cheap and
detecting the divergence would mean hashing the source tree and storing it, which is
machinery for a case that resolves itself the moment anyone runs the survey again.

## Risks

- **An agent that converts rather than migrates.** Mitigated above by the stage order and by
  a reviewed plan. Not checkable.
- **161 documents of judgment.** Accepted above; the plan and the surviving source are the
  recovery path.
- **Source edited mid-migration.** Accepted above, unmitigated.
- **A collapsed delta that misstates what a change established.** The same risk archival
  already carries, and the same mitigation: the agent reads the change's implemented result,
  and the implementation is what happened.
