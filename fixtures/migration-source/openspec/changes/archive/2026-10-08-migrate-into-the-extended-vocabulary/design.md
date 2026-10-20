## Context

See proposal.md for why. The shape of the current system that the approach depends on, from
`evidence/the-measurement-2026-09-30.md` unless another source is named:

- **A migration has three exact parts, and the rest is an agent's reading.** `migrate survey`
  enumerates, and `migrate promote` checks coverage, then checks the staged tree, then moves it.
  project-migration's "Conversion of a document's meaning is agent-driven" forbids a program that
  emits documents. So the product side of this change is a contract, the survey, and the instructions
  in `crates/htmlspec-assets/instructions/migrate.md`.
- **The survey reads no change's configuration.** `Change` in `crates/htmlspec-cli/src/migrate.rs`
  (lines 197-215) holds `name`, `archived`, `version`, and `delta_sections`. `.openspec.yaml` is
  classified as `change-configuration` and never opened (section 1).
- **`migrate.md`'s mapping table has eight rows, and the rename row has a test.**
  `a_renamed_section_is_surfaced_and_the_destination_can_express_it` in
  `crates/htmlspec-cli/tests/migrate.rs` reads `renamedFrom` from the published schema and asserts the
  row's wording. That test pattern is the precedent for the rows this change adds (section 2).
- **The staged tree resolves as its own project.** Promotion's `conformance` read 109 of the staged
  tree's 116 HTML files, because discovery resolved the staged project and excluded its evidence.
  `Staged::resolve` refuses a staging location that does not resolve to itself (section 6).
  `resolve-a-reference-across-its-change` derives a document's placement from `ProjectRoot`, and names
  promotion's `conformance` among the callers that go through the one validating function.
- **This repository's migration is a working folder of scripts and a brief.** `replay.py` replays
  every delta in archive order. `replay_vs_history.py` compares the replay with the accepted
  specification at every archival commit and finds 0 disagreements, with one entry in `KEPT_BY_MERGE`
  (`state-a-modified-requirement-whole`'s evidence, section 3). `build_packets.py` writes one packet
  per drafting assignment. `check_draft.py` checks each draft. The plan's decisions and conventions
  were reviewed on 2026-09-29 against a destination that had none of the new vocabulary.
- **The nine earlier changes' vocabulary** is as their specifications state it. Two of its names
  collided with existing terms, and both were renamed on 2026-10-01. A task's `outcome` sat beside a
  verification's `data-outcome`, and is now `deviation`. A risk's `data-disposition` sat beside a plan
  entity's disposition, which project-migration's "The plan accounts for every surveyed entity" defines
  as `converted`, `folded`, `carried`, or `dropped`, and is now `data-response`.
- **No command records or reads a change's creation date.** The change that would have recorded one
  was withdrawn on 2026-10-01, and its measurement is copied into this change as
  `evidence/a-creation-date-no-command-reads-2026-09-30.md`.

## Goals / Non-Goals

**Goals:**

- One rule that a migration maps what the destination can express, with every mapping it implies
  named and placed.
- A migrated modified requirement that is whole, completed from the author's own merge, and marked
  where its words changed.
- A survey that carries everything a stage needs from a change's configuration, so no stage reads the
  source a second time.
- Instructions an agent can follow for each mapping, held to the schema by a test.
- This repository's migration routines brought to the same rules, with a proof on drafts.

**Non-Goals:**

- New vocabulary. Every value used here is added by the change whose subject it is.
- A converter, or a program that writes change markup. The conversion stays an agent's reading.
- Re-staging this repository's migration. That is the self-migration's next step.
- A check in the program that the marks are complete, or that a staged `FOLLOWS` agrees with the
  survey.
- A record of a change's creation date. The project decided not to keep one (decision 11).

## Decisions

### 1. The rule is generalized, and judged against the contract a migration stages against

"A source distinction the destination cannot express is recorded as open, not mapped to the nearest
state" gains its other half. Once the destination can express a distinction, migration maps it and
does not record it as open. The requirement lists each distinction the destination now expresses and
names the requirement that maps it, the rename included. The paragraph that keeps settling the
question out of scope for a migration stays word for word, so the migration still adds no value of its
own.

Whether the destination can express a distinction is judged against the contract the migrated project
is staged and checked against. That is this repository's case exactly: its plan was reviewed against a
destination with none of this vocabulary, and staging has not finished.

**Considered: a sentence in each new requirement, as "A source rename is carried as the heading the
requirement replaced" has.** Rejected. Five exceptions to a rule that still said "record as open"
would read as a rule contradicted five times. One general sentence, with a list that points at each
requirement, states the rule once. The rename sentence stays, because it is still true and it was the
first instance.

**Considered: judging against the contract the plan was written for.** Rejected. A reviewed plan would
then keep every workaround it was reviewed with, which is the state this change exists to end.

### 2. Five added requirements, one for each mapped fact

The mappings are separate requirements: tasks, references, partial verifications, risks, and the
change's configuration with its predecessor. Marking and completing a modified requirement go into the
collapse requirement instead, because both are about what the collapsed specification states.

**Considered: one requirement, "A source change's records are carried in the destination's
vocabulary".** Rejected, for two reasons:

- It would hold five subjects, with five reasons and nearly thirty scenarios, and a later reader could
  not cite the rule for tasks without citing the rule for risks.
- Each mapping depends on one earlier change's vocabulary. When that change's name moves at review,
  one requirement is re-based and the rest are not touched. Two names moved on 2026-10-01, as the
  context records, and each touched one requirement. When the creation-date record was withdrawn the
  same day, its requirement went, and no other changed.

**Considered: folding tasks, references, and risks into the collapse requirement.** Rejected. Only
verifications live in the specification. Tasks live in the tasks document, a decision's references in
the design, and risks in the design.

**Considered: putting the survey's reading of the configuration into "Migration surveys the source
before anything is read or written".** Rejected. That requirement states what a survey is. The
configuration requirement applies its rule on reading the fields a document carries to one file, and
keeps the reading of that file beside what is done with it.

### 3. A partial `MODIFIED` block is completed from the source's own merge

A migrated modified requirement is stated whole, as `state-a-modified-requirement-whole` requires of
every change's specification. Where the source's block left out a scenario, the accepted specification
as the source's merge left it decides: a scenario still held is `unchanged`, and one gone is `removed`.
The before and after are read from the source's version history, at the commit that applied the
change's contract. `state-a-modified-requirement-whole`'s evidence, appendix A, gives the method for
finding that commit, and its section 3 finds three such blocks in this repository's archive. One kept
both omitted scenarios, and two withdrew theirs.

Where the history does not hold the accepted text, the omission goes to the plan's review. Migration
neither keeps nor withdraws the scenario on its own reading.

**Considered: OpenSpec's reading, that an omitted scenario is withdrawn.** Rejected. It is wrong for
`separate-artifact-kinds`, whose merge kept "Unknown subcommand" and "Discoverable help". The replay
applied this reading until `replay_vs_history.py` found the one disagreement.

**Considered: reading an omission as kept.** Rejected. It is wrong for "Status visible to a reviewer"
and "One identity per workflow", which were withdrawn.

**Considered: putting every omission to review.** Rejected where the history answers it. The merge is
the author's own record of what they meant, and a review's time belongs to questions only a person can
answer.

In this repository, `replay.py` with `KEPT_BY_MERGE` is how the rule is applied, and
`replay_vs_history.py` is its check. Convention 4 of the plan, "The source archive is the evidence, not
git history", is amended to say that the archival commit decides where the two differ.

**A stated scenario carries its accepted counterpart's identifier**, as artifact-kinds' "A change's
specification states a modified requirement whole" requires: `state-a-modified-requirement-whole`
kept that sentence when it was archived, with its scenario "A scenario keeps its identifier". For a
migrated change, the counterpart is the scenario the migrated accepted specification holds under the
same requirement with the same heading. The plan's decision 2 qualifies a scenario's identifier only in
a document where it has a twin, and so gives one scenario two identifiers. Section 9 of the
measurement finds one such scenario among 189: "An invalid specification", `unchanged` in
`record-a-renamed-requirement`, whose accepted counterpart is qualified. The plan's decision 2 is
amended so that a scenario carried forward takes its counterpart's identifier.

`show-what-a-modification-changed` pairs a stated scenario with its accepted counterpart by this
identifier, so a scenario that kept the wrong one would show as withdrawn and added.

**Considered: leaving decision 2 as reviewed.** Rejected while the sentence stands. The one scenario
would then be the same obligation under two identifiers, which is what "Identifiers are derived once
and every reference resolves through that derivation" exists to prevent for requirements.

### 4. The changed words are marked, by the agent, against the text they replaced

Every requirement and scenario a migrated specification marks `modified` carries `ins` and `del` for
the words the change added and deleted, against the accepted text the change modified. How finely to
mark is the agent's decision, and a paragraph rewritten throughout may be deleted and added whole, as
`mark-the-words-a-modification-changed`'s instructions tell an author.

**Considered: permitting the marks, as `mark-the-words-a-modification-changed` does for an author.**
Rejected, for three reasons:

- The migration already holds the changed words. It compares each modified requirement with the text
  it replaced in order to state each scenario's change state, which a whole statement requires.
  The replay does this for 34 requirements, and finds that 33 changed their statement text.
- No later reader can recover them. The accepted specification an archived change modified has since
  moved on, so a comparison with it shows every later change's edits as well.
  `show-what-a-modification-changed` refuses an archived change for this reason, and says that none
  of the staged migration's 24 archived changes is served by its view.
- Without marks, which words moved survives only where a summary narrates it, and five of this
  repository's drafted summaries already do.

**Considered: a program that emits the marks from a computed diff.** Rejected. It is a converter by
another name, which "Conversion of a document's meaning is agent-driven" forbids. Its output would
also be worse. Which words form one edit, and when a paragraph reads better deleted and added whole,
is a reading. The comparison is evidence the agent reads: in this repository, each packet carries the
previous text, and `check_draft.py` checks the marks against it.

**Considered: taking the previous text from `htmlspec show <change> --diff`.** Rejected as the source
of the marks. `show-what-a-modification-changed` refuses an archived change, because compared with
today's accepted text it shows nothing or a later change's edit reversed, and "The text the accepted
specification held then is recorded only in git" (its proposal). Its view is still useful as a second
check. Task 6.3 rebuilds one of `finish-the-archival`'s modified requirements the way that change's
task 7.1 rebuilds `sync-a-change-into-the-accepted-contract`, as an active change beside the text it
modified, and compares the view's word marks with the draft's.

The replay's "previous statement" is the previous delta's text, not the accepted text at the merge.
The two agree for every requirement in this archive, because `replay_vs_history.py` finds 0
disagreements, so the replay may supply the previous text here. The requirement names the accepted
text, so a source whose replay and history disagree is decided by its history.

### 5. A task keeps the state its record supports, and the brief's contradiction goes

The brief's tasks rule and its convention 2 disagree about a task ticked and struck through with a
stated reason. The rule makes it `dropped`, and the convention keeps it `done` with a note. The rule
wins: `[~]`, or a task struck through, is `dropped` with the source's reason, whichever box was ticked.
Convention 2 is amended to say so, and a task done another way is `done` with a deviation in place of
a note. A task whose result nothing records stays `done`, with its note, because a deviation states
what was done.

Where the account of a difference is the task's own text, as "**Amended during implementation.**" is,
that account is the deviation, in the author's words. Where the account is elsewhere, the deviation
says what was done and names where that is shown: a carried record, which it links, or the source at a
named commit. Several of the 33 notes rest on code at an archival commit, so a carried record alone
would not cover them. In this repository, a deviation written in the migration's words opens with
"Migration note:", because the brief marks everything the migration wrote.

An abandoned task with no stated reason goes to the plan's review. If the review supplies none, the
task takes the state OpenSpec reads from its checkbox, with a note, and the plan records it. This
repository has no such task: all four abandoned tasks state their reason.

**Considered: a reason paragraph saying that the source gave none.** Rejected. It is a sentinel in a
field whose job is to say why, and validation would accept it as a reason.

**Considered: keeping convention 2.** Rejected. It is the nearest state that validates, and `list`
would count every one of the 33 tasks as done as written.

**Considered: striking the words through with `del`.** Rejected, as settled for this set of changes. A
task never carries change markup, and `dropped` already says its words were given up. The staged
drafts carry the strike as `s`, which is presentational and stays permitted.

### 6. A unit the author's words name becomes a reference, and its link stays

A task or decision that names a requirement or scenario in its author's words gains `data-refs` for
it, where document-schema's "Cross-references address units by identifier" reaches the unit from that
document. The link around the author's words stays. `resolve-a-reference-across-its-change`'s task
5.3 measured what this does to the staged tree: 55 of the 56 linking units resolve, 30 in their
change's specification and 25 in the accepted specifications. Two of the 25 link to another change's
specification, and resolve because the accepted specifications declare the same identifiers. Task
5.1 of `distinguish-command-names-from-skill-names` names a withdrawn scenario and reports
`refs/removed-target`, so it alone stays a link.

**Corrected on 2026-10-08.** This paragraph said 53 resolve, and that the two units naming another
change's requirement stay links. That was task 5.3's expectation as written, and the measurement
recorded in that change's archive found it wrong (`evidence/the-measurement-repeated-2026-10-08.md`,
section 1.0). Reachability is decided by identifier, as validation decides it, so the rule here
gives those two a reference. Whether it should is put to the contributor at review.

**Considered: replacing the link with the reference.** Rejected. A reference is not a hyperlink, and a
reader of the page would lose the way to the unit.

**Considered: a reference for every linked unit, reachable or not.** Rejected. Each unreachable one
fails `refs/unresolved` or `refs/removed-target`, and `resolve-a-reference-across-its-change` keeps
references between changes out on purpose.

**Considered: references inferred from a task's subject or its capability.** Rejected. The model
would then hold relationships the author never stated. The rule that the author's words name the unit
is the brief's rule for links, and `check_draft.py` enforces it by flagging any `data-refs` entry that
no link in the same unit points at.

### 7. Placement resolves inside a staged tree, so nothing in staging changes

Section 6 of the measurement shows how. `Staged::resolve` refuses a staging location unless
`ProjectRoot::resolve` finds that location itself, and conformance runs only after it passes. From any
staged document, `ProjectRoot::resolve` reaches the staged root before any `.git`.
`resolve-a-reference-across-its-change` derives placement from `ProjectRoot`, positionally and on
canonical paths, and promotion's conformance reaches it through the one validating function. The
staged tree's paths are the destination's, because promotion is a move. So a document's placement
in staging is its placement after promotion, and a staged reference reaches the staged change's
specification and the staged accepted specifications.

**This is not a gap in the code, and it is a gap in what is tested.** No test puts a cross-document
reference into a staged tree. Task 2.4 adds one to `crates/htmlspec-cli/tests/promote.rs`: one staged
task whose reference resolves in its change's staged specification, and one whose reference resolves
nowhere, run through `migrate promote --check`.

**Considered: a requirement of its own for placement in staging.** Rejected. The scenario "Checks run in
the staging location" in "The migrated project is assembled in a staging location and promoted in one
step" already requires the staged tree to be checked "as they would after promotion". The references
requirement names the case, and the test holds it.

### 8. A partial check is recorded as `partial`, and nothing else widens

Where a carried record shows a check that reached part of what the verification names, the
verification records `partial`, with an `unchecked` paragraph for each part the record does not show,
in the scenario's own words. Where a check reached something else, no unit is written, as before. In
this repository that restores the two units review removed and corrects the two staged as `passed`.
`record-a-partial-verification`'s section 4 reads five more declined checks as partial. The re-stage
decides each against its record.

**Considered: re-auditing every `passed` unit here.** Rejected. It is a review of 90 judgments, not a
mapping, and nothing measured says more of them are partial.

### 9. A risk's stated settlement becomes its response, and the shape rule stays

A risk becomes a unit only where the source gave it a heading and put its statement and any measure
in paragraphs of their own. The response changes one thing: an accepted risk needs no mitigation,
so a headed accepted risk with no measure now has that shape. `record-how-a-risk-was-settled`'s
section 6 counts five such risks, and the one drafted unit, which is accepted with a partial measure.

Migration declares a response only where the source states one. "Not mitigated" is `accepted`:
`record-how-a-risk-was-settled` defines an accepted risk as one the change bears as it stands, and
names its own release runbook's "Not mitigated" risks as the case. A risk that states a mitigation and
no settlement takes the default.

**Considered: declaring `mitigated` on every carried risk.** Rejected. It adds a word the author did not
write, and the default already reads it.

**Considered: recording "not mitigated" as open.** Rejected. It is the meaning `accepted` has, not the
nearest value to it.

`migrate.md` writes "a risk's response" or "a risk's `data-response`", and never calls it a
disposition. In `migrate.md` and project-migration, a disposition is what the plan decides about an
entity, and that collision is why the attribute, first named `data-disposition`, was renamed on
2026-10-01.

### 10. The survey reads each change's configuration, and the value of `follows:` only

`Change` gains `configuration: Option<ChangeConfiguration>`. `None` means the change has no
`.openspec.yaml`, and it is serialized as `null`, not omitted, for the reason `version` gives: "this
change recorded none" is a fact the plan carries.

```rust
pub struct ChangeConfiguration {
    /// Every key the file sets, in the order a reader meets them.
    pub keys: Vec<String>,
    /// `follows:`, as the author wrote it, where the file states one.
    pub follows: Option<String>,
}
```

The payload is `"configuration": {"keys": [...], "follows": "..."}`. Keys are read by the rule
`read_configuration` already applies to `config.yaml`. A value is the text after the first colon, with
a trailing ` #` comment removed, trimmed, and one pair of matching quotes removed.

**The survey reads the value of `follows:` and of no other key.** `follows:` feeds `FOLLOWS`, which
the stager writes and the program reads. `created:` would feed nothing, because the project records no
creation date (decision 11), and a survey field nothing consumes is the case project-bootstrap's
sentence, quoted there, excludes. The `created` key is still reported among `keys`, so the plan
accounts for it as it accounts for `schema:`.

**Considered: the stager reading the file again, as `stage_carry.py` does.** Rejected. "Migration
surveys the source before anything is read or written" makes the survey the list a stage is checked
against, and a second reading escapes that.

**Considered: reading `created:` as well, as a date or as unreadable text.** Rejected. Its one
consumer would have been a creation-date record, and the project decided not to keep one. A value the
survey parses and reports with nothing to receive it is the field project-bootstrap's sentence
excludes, and a malformed date it reported would be a fault nobody could act on, in a file that
crosses byte-identical anyway.

`follows:` is carried as text. Whether it names a change is resolution, which `FOLLOWS`'s reader
already reports as archived, active, or unmatched.

### 11. The creation date stays in carried evidence

No migrated change gets a creation-date record, and `created:` maps to nothing. The date crosses where
it already does. Every `.openspec.yaml` is carried byte-identical into its change's `evidence/`, as
the plan's reviewed decision 1 carries all 15 files, and the staged tree holds them. So the creation
date is neither a mapped distinction nor an open question.

**The reason is the project's refusal, not a gap in the destination.** On 2026-10-01 the project
decided not to record a creation date, because no command depends on one. The contract judges a
recorded value by what depends on it, and project-bootstrap's "A project declares its name, version,
and expected toolchain" states the test: "At least one command SHALL depend on the declaration. A
declaration no command reads states nothing that can be wrong, so nothing reports it when it is". The
withdrawn change's design applied that test, and three others, to every reader a date could have, and
concluded: "So, by the contract's own measure, the date is accuracy without authority." Its
measurement, copied as `evidence/a-creation-date-no-command-reads-2026-09-30.md`, gives the facts the
verdict rests on. OpenSpec itself reads `created:` nowhere except to check its form (section 5), and
this repository's migration already carries all 15 dates in evidence (section 3).

That is why the date is not recorded as open under decision 1's rule. An open question names the
change that would settle it, and that change was proposed, measured, and withdrawn. A question the
project has answered is not open, and asking it again in every migrated change would be the question
nobody needs to answer that decision 1 retires.

**The file crosses into `evidence/` in this repository, as reviewed.** It is the only record of the
date, and of its `schema:` key, which has no destination. The product requirement leaves the choice to
the plan, says the contract is silent on it, and asks a plan that drops the file to say that the date
goes with it. In `make_decisions.py`, decision 1's rationale changes from "The destination has no
field for a start date" to the project's refusal: on 2026-10-01 the project decided not to record a
creation date, because no command depends on one. Each entity's reason stops restating its date, which
the carried file states, and the rules-table row stops saying "a `FOLLOWS` file, or none".

**Considered: mapping `created:` to a record of its own.** Rejected. The project withdrew that record,
so there is nothing to map to, and a migration that wrote one would extend the vocabulary, which
decision 1 forbids.

**Considered: folding the file into `FOLLOWS`, and dropping it.** Rejected. It reverses a reviewed
decision to save a file of 40 to 77 bytes a change, and it loses the only record of the date.

### 12. `migrate.md` maps each fact, and a test holds the rows to the schema

The table gains four rows:

- `## MODIFIED Requirements`: stated whole, with a partial block completed from the merge, and the
  changed words marked.
- `## REMOVED Requirements`: `data-change="removed"`, holding what the requirement held before, with
  the delta's Reason and Migration as notes. The brief already does this, and document-schema's
  "Semantic units declare a change state" keeps a withdrawn unit present.
- a task's checkbox.
- `.openspec.yaml`: `follows:` as `FOLLOWS`, the file as a plan entity, and `created:` as a key with no
  record of its own, which crosses only inside the file.

The guidance gains a short section for each subject: modified requirements, tasks, references,
verifications, risks, and the configuration. Each cites its requirement by heading. Stage 3 also
gathers the text each modification replaced. "What to report" asks which distinctions were mapped, so
that a mapped distinction is not presented as open, or the reverse.

A new test, beside the rename test, reads `deviation`, `partial`, `unchecked`, and `response` from the
published schema, and asserts that each new row names its vocabulary and none says "open question".
**Corrected on 2026-10-08: it is two tests.** This change was built on a branch whose schema held
none of the three changes' fields, so the schema half,
`the_destination_expresses_each_distinction_migrate_md_maps`, was marked `#[ignore]` until the stack
below it was assembled, and runs since, and the rows half, `each_distinction_the_destination_expresses_has_a_row`,
runs on its own.
Another asserts that the survey's payload carries every archived `.openspec.yaml`'s keys and
`follows:` value, compared with an independent reading of the files. It states a property, as the
file's own header asks, so it keeps passing as the archive grows.

### 13. The working folder is changed file by file, and reviewed decisions are amended

The tasks name each file and what changes in it. `make_decisions.py` records each decision as it was
put in review, beside its answer, because the plan is carried as evidence. So decision 1's question
and answer stay as they are, and a dated amendment goes beside them. The same holds for conventions
2, 4, and 7. The revised conventions go to the contributor at the plan's next review, which is part of
the re-stage.

`make_decisions.py` reads `follows:` from the survey's payload instead of `change-config.tsv`, and
stops restating each `created:` value, which the carried file already states. `stage_carry.py` writes
`FOLLOWS` from the payload. That closes the second reading decision 10 names. `stage_carry.py` also
stops refusing a target that already holds the same bytes, so it can run over the tree it staged
before.

### 14. The proof re-drafts the pilots, and three packets the pilots do not reach

The three pilots exercise change markup, deviations, references, and a predecessor. They hold no
partial check, no risk unit, and no scenario a merge kept (measurement, section 7). A proof that never
exercised those three would prove nothing about them. So the proof also re-drafts:

- `2026-08-13-content-address-runtime-assets`, for its two partial checks;
- `2026-08-14-separate-artifact-kinds`, for its two kept scenarios;
- `2026-09-06-distinguish-a-requirement-from-an-example`, for its drafted risk unit and a headed
  accepted risk with no measure.

**This goes past "the three pilots", and the contributor confirmed it on 2026-10-01.** The routines
run on a copy of the source extracted from the plan's commit, `6a591bd`, with `git archive`, and the
drafts go into a scratch copy of the staged tree. The reviewed plan predates the ten new archives, and
the staged tree is left for the re-stage to redo whole.

**Considered: the full re-stage here.** Rejected. It is 37 assignments plus the ten new archives, and
it is the self-migration's next step, not this change's subject.

## Risks / Trade-offs

- **[Marks are only as right as the previous text.]** → The replay agrees with the accepted
  specification at every archival commit, and `check_draft.py` checks each unit's marks against it
  both ways. With `del` unwrapped and `ins` dropped, the text must give the previous text. With `ins`
  unwrapped and `del` dropped, it must give the current one.
- **[An agent marks a rewritten paragraph phrase by phrase until it cannot be read.]** → `migrate.md`
  and the brief say to delete and add such a paragraph whole.
- **[A deviation written in the migration's words reads as the author's.]** → It opens "Migration
  note:" and names where the fact is shown.
- **[A name moves at review: `unchecked`, `partial`, `deletions`, or another.]** → Two already have,
  as the context records, and this change uses the new names. Task 1.0 re-reads every name from the
  archived changes. The new tests read field names from the schema, so a rename fails them rather
  than passing them.
- **[Reviewed conventions change after review.]** → They are amended beside the reviewed text, dated,
  and put to the contributor at the next review.
- **[The proof covers 6 of 37 assignments.]** → Task 6.5 runs the revised `check_draft.py` over every
  other existing draft and records what it finds, by kind, as the re-stage's worklist. Those drafts
  predate the revised brief, so the findings are expected.
- **[A reference is added where the author named nothing.]** → `check_draft.py` flags each `data-refs`
  entry that no link in its unit points at.

## Migration Plan

- **Contract version.** No unit, field, attribute, or projection rule is added, so the document
  contract does not move. It stays where `record-how-a-risk-was-settled` leaves it. Under the bump
  table in `crates/htmlspec-schema/CHANGELOG.md`, nothing this change edits is versioned there.
- **The survey's payload** gains one key on each change, `configuration`. Nothing is renamed or
  removed. `crate::plan::surveyed_paths` and the coverage gate read `entities` only, so a survey written
  before this change still checks a plan.
- **Existing projects.** Nothing to do. A migration planned before this change is re-planned under
  decision 1 before its affected documents are staged.
- **Rollback.** Revert. The payload loses the key, and `migrate.md` loses the rows.
- **Release.** Expected to be `0.28.0`, the next minor increment after `record-how-a-risk-was-settled`'s
  `0.27.0`. The survey reports more, and the migration contract changes. The contributor confirms the
  increment at the release task.
