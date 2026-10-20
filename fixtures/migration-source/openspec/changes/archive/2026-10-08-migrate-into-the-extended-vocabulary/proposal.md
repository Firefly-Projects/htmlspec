## Why

**The nine changes before this one give the destination the vocabulary this repository's migration
had to work around, and nothing yet tells a migration to use it.** project-migration's "A source
distinction the destination cannot express is recorded as open, not mapped to the nearest state"
forbids a migration to add a value, so the migration staged in `~/projects/html-spec-migration/`
worked around every gap it met. `evidence/the-measurement-2026-09-30.md` records each workaround and
reproduces the counts behind it:

- **Plain links instead of references.** The drafting brief says "Never give a unit `data-refs`
  naming a unit in another document", so 54 task units and 2 decision units link to a unit in another
  document, and none of those links is checked.
- **A note instead of a deviation.** 76 task units carry a migration note, and one reader's
  classification finds 33 of them done otherwise than the task's words say. Each is `done`, and every
  count reads it as done as written. The brief's rule for tasks and its convention 2 also contradict
  each other on a struck-through task.
- **Deleted or overstated checks.** Review removed two verifications because each record reached only
  part of its scenario, and two more are staged as `passed` while their method paragraphs say what the
  record lacks. The brief offered four outcomes.
- **Summaries that narrate a word.** 33 of the 34 modified requirements changed their statement text,
  and no unit says which words. Five summaries narrate it instead, as in "Its only change … is `sync`
  added".
- **A predecessor read around the survey.** All 15 `.openspec.yaml` files cross byte-identical into
  `evidence/`, because "The destination has no field for a start date". The survey reads none of their
  values, and the stager opens each file again for `follows:`. The files keep crossing, since that is
  where the creation date lives, but the reason is now the project's: it decided on 2026-10-01 not to
  record a creation date, because no command depends on one.
- **Risks kept as prose.** An accepted risk could not be a unit without a mitigation, so five headed
  accepted risks stayed prose, and the one drafted risk unit reads as mitigated by default, against its
  author's "Accepted".

The product side has the same gaps. `crates/htmlspec-assets/instructions/migrate.md` has no row for
`## MODIFIED Requirements`, `## REMOVED Requirements`, a task's checkbox, or `.openspec.yaml`, and
none of the new words appears in it. `migrate survey` reports each change's name, whether it is
archived, its version, and its delta sections, and nothing from its configuration.

**This change depends on the vocabulary each earlier change lands.**
`reject-change-markup-inside-a-unit` rejects `ins` and `del` inside a unit, as `schema/change-markup`.
`state-a-modified-requirement-whole` makes a change's specification state a modified requirement
whole, each scenario carrying its accepted identifier. `see-a-modification-the-merge-did-not-take`
compares a modification's text in `sync` and archival, through `untakenContent` and
`untakenRemovals`. A migrated archive is never synced, so this change asks nothing of that comparison.
`resolve-a-reference-across-its-change` lets `data-refs` reach a change's specification and every
accepted specification, and places an archived change as an active one.
`mark-the-words-a-modification-changed` permits `ins` and `del` in a requirement or scenario marked
`modified` in a change's specification, and projects deleted text into `deletions`.
`show-what-a-modification-changed` adds `htmlspec show <change> --diff`, and refuses an archived
change, because the text an archived change modified is recorded only in git. So it serves none of the
migration's archived changes, and this change uses it only to check its marks on a rebuilt copy.
`record-a-task-done-differently` adds a
`data-field="deviation"` paragraph on a `done` task, and `schema/deviation-not-done`.
`record-a-partial-verification` adds the outcome `partial`, with `data-field="unchecked"` paragraphs.
`record-how-a-risk-was-settled` adds a risk's `data-response`, `accepted` or `mitigated`.
One more, `record-when-a-change-was-created`, was withdrawn on 2026-10-01, because no command needs
a creation date.

**Why now.** This repository's migration out of OpenSpec is staged, and all 37 drafts have been
reviewed. Promotion has not run. Its plan was reviewed against a destination that had none of this
vocabulary. Without this change, the migrated documents carry the workarounds above, or record as
open questions distinctions that the destination can now express.

## What Changes

- **A distinction the destination can express is mapped, not recorded as open.** project-migration's
  rule is generalized from the rename precedent. It names what now maps and the requirement that
  maps each one, and it judges this against the contract a migration is staged against. The migration
  still adds no vocabulary of its own.
- **A modified requirement is stated whole, and the words it changed are marked.** Each scenario
  carries its accepted counterpart's identifier. Where a source `MODIFIED` block left out a scenario,
  the block is completed from the source's own merge, read from its version history. A kept scenario
  is stated `unchanged`, and a withdrawn one `removed`. The words each modification changed are
  marked with `ins` and `del` against the text it replaced. That is
  required of a migration, although it is only permitted to an author. The marks are the agent's to
  write, and a computed comparison is evidence it reads.
- **A source task keeps the state its record supports.** `[x]` is `done` and `[ ]` is `todo`. `[~]`,
  or a task struck through, is `dropped` with the source's reason. A task done another way is `done`,
  with a deviation. An abandoned task with no stated reason goes to the plan's review, and migration
  writes no reason of its own.
- **A unit the author's words name becomes a reference.** A task or decision that names a requirement
  or scenario in its change's specification, or in the accepted specifications, carries `data-refs`
  beside its link. A unit the document cannot reach stays a link. References are checked in the staged
  tree as after promotion.
- **A partial check is recorded as `partial`,** with what it did not check. It is not recorded as
  `passed`, and it is not deleted.
- **A risk's stated settlement becomes its response.** A headed accepted risk needs no mitigation to
  be a unit.
- **The survey reads each change's own configuration.** For each change, the payload carries whether
  it has a configuration file, every key the file sets, and its `follows:` value. It reads no other
  value, because no stage consumes one.
- **`follows:` becomes `FOLLOWS`,** as the source wrote it, and is inferred for no change that states
  none.
- **The creation date stays in carried evidence.** It is neither mapped nor recorded as an open
  question. Every `.openspec.yaml` still crosses byte-identical into `evidence/`, and the date crosses
  inside it, because the project decided not to record a creation date.
- **`migrate.md` maps each of these.** It gains rows for a modified requirement, a removed
  requirement, a task, and `.openspec.yaml`, and guidance on each subject, each citing its
  requirement. A test holds the rows to the schema, as the rename row's test does.
- **This repository's migration follows**, in its working folder: the decisions, the stager, the
  replay, the packets, the brief, and the draft checker. The three pilot packets are re-drafted as
  proof, and so are three packets holding what the pilots do not.
- **Unchanged:**
  - the document contract, so `CONTRACT_VERSION` stays where `record-how-a-risk-was-settled` leaves
    it;
  - promotion's checks and the staging code. Placement already resolves inside a staged tree;
  - the survey's entity kinds, its counts, and the coverage gate;
  - the rename mapping, and evidence crossing unchanged.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `project-migration`:
  - **Modified: "A source distinction the destination cannot express is recorded as open, not mapped
    to the nearest state".** It gains the rule that a distinction the destination expresses is mapped,
    and the list of what now maps, and it says that a creation date is neither mapped nor recorded as
    open, because the project decided not to record one. It is modified rather than joined by a new
    requirement, because a
    second requirement would state the same rule's other half. The rename requirement's own sentence,
    "A rename SHALL NOT be recorded as an open question once the destination can express it", was the
    first instance of the general rule, and it stays as it is.
  - **Modified: "A change's delta specifications collapse into that change's one specification".** A
    modified requirement is stated whole, a partial block is completed from the source's merge, and the
    changed words are marked. This is where the collapse's obligations already live, and each of the
    three is about what the collapsed specification states.
  - **Added: five requirements, one for each fact mapped outside the specification's requirements.**
    They are "A source task is carried in the state its record supports", "A unit the author's words
    name is referenced wherever the migrated project resolves it", "A check that reached part of what
    it names is carried as a partial verification", "A source risk's stated settlement is carried as
    its response", and "A source change's own configuration is surveyed, and its predecessor becomes
    its declaration". Each of the first four maps one earlier change's vocabulary, so each can be
    re-based alone if that change's name moves, as two did on 2026-10-01. The last also states that
    the creation date crosses only inside the carried file. The design's decision 2 gives the reasons
    against one requirement for all five.
  - **Checked and unchanged:** "Migration surveys the source before anything is read or written", whose
    rule on reading the fields a document carries the configuration requirement applies. "The migrated
    project is assembled in a staging location and promoted in one step", whose scenario "Checks run in
    the staging location" already requires what references need. "Evidence crosses unchanged", which
    governs a change's evidence and not its configuration file. "A source rename is carried as the
    heading the requirement replaced".

No other capability is modified. The vocabulary belongs to the changes that add it, and this change
only maps a source onto it.

## Impact

- `crates/htmlspec-cli/src/migrate.rs`: `Change` gains the change's configuration: whether there is a
  file, the keys it sets, and `follows:`. `payload()` carries it.
- `crates/htmlspec-cli/src/commands.rs`: `migrate_survey`'s human report names the changes that state
  a predecessor.
- `crates/htmlspec-cli/src/main.rs`: the `migrate survey` help says it reports each change's
  configuration.
- Tests:
  - unit tests in `migrate.rs` for the configuration;
  - `crates/htmlspec-cli/tests/migrate.rs`: the harness reads the configuration, a test compares it
    with an independent read of every `.openspec.yaml`, and a test holds the new `migrate.md` rows to
    the published schema;
  - `crates/htmlspec-cli/tests/promote.rs`: a staged project whose references cross documents is
    checked as it will be after promotion.
- `crates/htmlspec-assets/instructions/migrate.md`: four rows, guidance on each subject, and stage 3
  gathering the text each modification replaced. The five installed copies are regenerated.
- The working folder, outside the repository, with results recorded in `evidence/`: `make_decisions.py`,
  `stage_carry.py`, `replay.py`, `build_packets.py`, `brief-draft.md`, and `check_draft.py`.
- **Not in this change**, and recorded so it is not assumed to follow:
  - **The full re-stage of this repository's migration.** It is the self-migration's next step. It
    must re-draft all 37 assignments under the revised brief, and it must also take in the ten
    archives this set of changes adds to `openspec/changes/archive/`, which the reviewed plan predates.
  - A program check that a migration's marks are complete, or that a staged `FOLLOWS` agrees with
    the survey. Both remain the review's and the draft checker's.
  - Rows in `migrate.md` for a converted document, a release inventory, and the adoption marker. They
    carry no vocabulary.
  - Re-auditing the staged migration's 90 other `passed` verifications, or the 8 declined checks that
    reached something other than their scenario.
  - The 88 risk entries that have no heading. A risk's response does not give them the shape of a unit.
  - The open questions `queue.md` lists for the contributor, none of which this vocabulary settles.
