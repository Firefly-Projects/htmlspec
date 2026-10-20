# The migration's routines and the workarounds they carry, measured before the change

**Date:** 2026-09-30
**Tasks:** 1.2 to 1.6. Section 8 also serves 6.2.
**Binary:** `htmlspec 0.18.0 (6a591bdf9d92)`, built at HEAD. It prints `6a591bdf9d92-dirty`, because
the working tree holds this set's untracked change directories. `git status --porcelain | grep -vc
'^??'` printed `0`, so no tracked file differs from HEAD.

**Note, 2026-10-01.** Nothing below was measured again, and it keeps the names it was measured
under. Since then, two names were renamed: a risk's `data-disposition` is now `data-response`, and a
task's `data-field="outcome"` is now `data-field="deviation"`, so `give-a-task-an-outcome` is now
`record-a-task-done-differently`. `record-when-a-change-was-created` was withdrawn, so no change
records a creation date. Its measurement, which sections 1 and 5 below cite, is copied into this
change as `evidence/a-creation-date-no-command-reads-2026-09-30.md`.

The working folder is `~/projects/html-spec-migration/`, outside the repository and not a git
repository. Scratch files are under `scratchpad/migrate-into-the-extended-vocabulary/`. Nothing in the
working folder or the repository was written.

## Summary

| Question | Answer |
| --- | --- |
| Does the survey read a change's `created:` or `follows:`? | **No.** Each change in the payload carries `name`, `archived`, `version`, and `deltaSections`, and nothing else. 15 archived `.openspec.yaml` files state `created:`, 3 state `follows:`, and 9 archives have no file. |
| Does `migrate.md` map a modified requirement, a removed one, a task, or `.openspec.yaml`? | **No row for any of them.** The words `outcome`, `partial`, `data-refs`, `CREATED`, `FOLLOWS`, `<ins>`, and `<del>` appear nowhere in it. |
| What do the working folder's rules say? | Plain links and never `data-refs`; a task done differently keeps `done` with a note; four verification outcomes; risks as prose unless headed with a mitigation paragraph. The tasks rule and convention 2 contradict each other on a struck-through task. |
| Does `check_draft.py` read a strike after the task number? | **No.** It flags task 4.5 of `sync-a-change-into-the-accepted-contract` as wrongly `dropped`. |
| Do the other changes' counts reproduce? | **Yes, all five spot-checked**: 54 task and 2 decision units linking across documents, 34 modified requirements and 13 modified scenarios, 76 task notes, 92/1/1 verification outcomes, and 1 drafted risk unit. |
| Does a staged document resolve its project? | **Yes, from inside the staged tree.** Promotion's check read 109 of the staged tree's 116 HTML files, which is every file outside a change's evidence. That exclusion comes from resolving the staged tree as a project. |
| What do the three pilots exercise? | Change markup, outcomes, references, and a creation date. **Not** a partial verification, a risk disposition, or a scenario a merge kept. |
| Does every carried-forward scenario carry its accepted counterpart's identifier? | **All but 1 of 189.** "An invalid specification" in `record-a-renamed-requirement` keeps an unqualified identifier the accepted specification qualifies. |

## 1. The survey today (task 1.2)

```
$ ./target/debug/htmlspec migrate survey --json . > survey.json        # exit 0, 83,069 bytes
$ ./target/debug/htmlspec migrate survey .                              # exit 0
365 entities in .
    13  accepted-specification
     1  adoption-marker
    26  change-configuration
    99  change-document
     1  configuration
     4  converted-document
    63  delta-specification
   130  evidence
    24  recorded-version
     4  release-inventory

35 changes, 24 archived
…
the project's configuration sets: schema
```

The 35 changes are the 24 archived and this set's 11 active ones, and each active one has a
`.openspec.yaml`, which is why there are 26 `change-configuration` entities. The human report names
the project's configuration keys and says nothing about any change's.

Read from the payload:

```
top-level keys: ['changes', 'configuration', 'counts', 'entities', 'greatestVersion', 'root', 'sharedVersions']
keys on each change: ['archived', 'deltaSections', 'name', 'version']
any change carrying created/follows: False
change-configuration entities: 26 archived: 15
```

One change, with its file:

```
$ cat openspec/changes/archive/2026-09-26-log-a-runbook-run/.openspec.yaml
schema: spec-driven
created: 2026-09-17
follows: establish-a-runbook
```

```json
{"archived": true, "deltaSections": ["ADDED Requirements", "MODIFIED Requirements"],
 "name": "2026-09-26-log-a-runbook-run", "version": "0.14.0"}
{"change": "2026-09-26-log-a-runbook-run", "kind": "change-configuration",
 "path": "openspec/changes/archive/2026-09-26-log-a-runbook-run/.openspec.yaml"}
```

**The file is enumerated as an entity, and none of its three keys reaches the payload.** In
`crates/htmlspec-cli/src/migrate.rs`, `CHANGE_CONFIG` (line 51) is used once, at line 492, to classify
the file. `Change` (lines 197-215) has `name`, `archived`, `version`, and `delta_sections`. Only the
project's `config.yaml` has its keys read, by `read_configuration`.

The archive's files, read with `grep '^created:'` and `grep '^follows:'`:

| Archive | `created:` | `follows:` |
| --- | --- | --- |
| `2026-08-10-html-spec` | 2026-08-04 | |
| `2026-08-13-content-address-runtime-assets` | 2026-08-10 | |
| `2026-08-14-separate-artifact-kinds` | 2026-08-13 | |
| `2026-08-31-sync-a-change-into-the-accepted-contract` | 2026-08-31 | |
| `2026-09-02-carry-task-progress-into-the-listing` | 2026-08-31 | |
| `2026-09-02-state-a-changes-place-in-a-sequence` | 2026-09-02 | |
| `2026-09-06-distinguish-a-requirement-from-an-example` | 2026-08-24 | |
| `2026-09-11-migrate-an-openspec-project` | 2026-08-24 | |
| `2026-09-15-catch-a-templates-placeholder-prose` | 2026-09-11 | |
| `2026-09-18-establish-a-runbook` | 2026-09-16 | |
| `2026-09-26-carry-a-runbook-to-another-machine` | 2026-09-17 | `log-a-runbook-run` |
| `2026-09-26-log-a-runbook-run` | 2026-09-17 | `establish-a-runbook` |
| `2026-09-26-record-a-renamed-requirement` | 2026-09-26 | |
| `2026-09-28-confine-the-asset-check-to-the-project` | 2026-09-26 | |
| `2026-09-29-keep-evidence-out-of-document-discovery` | 2026-09-29 | `migrate-an-openspec-project` |

```
$ ls -d openspec/changes/archive/*/ | wc -l                             # 24
$ ls openspec/changes/archive/*/.openspec.yaml | wc -l                  # 15
$ grep -l '^follows:' openspec/changes/archive/*/.openspec.yaml | wc -l # 3
```

The nine without a file are `project-bootstrap-cli`, `conform-generated-frontmatter-names`,
`install-the-full-workflow-set`, `check-release-hygiene`, `distinguish-command-names-from-skill-names`,
`accept-a-described-change`, `pass-the-description-through`, `report-in-the-contributors-terms`, and
`finish-the-archival`. Every file sets `schema: spec-driven` and `created:`, and nothing else but the
three `follows:`. **15, 3, and 9 reproduce** `record-when-a-change-was-created`'s section 1.

**The working folder reads these values around the survey.** `make_decisions.py` lines 16-19 read
them from `change-config.tsv`, a hand-kept table of the same 15 rows. `stage_carry.py` lines 52-58
read `follows:` by opening each `.openspec.yaml` again:

```python
    follows = next(line.split(":", 1)[1].strip()
                   for line in (REPO / path).read_text().splitlines()
                   if line.startswith("follows:"))
```

**Whose clock `created:` is.** `record-when-a-change-was-created`'s evidence, section 5, measured
OpenSpec 1.13.1: `formatLocalDate()` writes both `created:` and the archive directory's date prefix, in
the local time of the machine, and nothing records the zone. Its section 4 shows `created:` on or
before the archive date in 15 of 15.

## 2. `migrate.md` today (task 1.3)

`crates/htmlspec-assets/instructions/migrate.md`, lines 80-89, the whole mapping table:

```
| OpenSpec | html-spec | What decides it |
| --- | --- | --- |
| `specs/<capability-path>/spec.md` | `htmlspec/specs/<capability-path>/spec.html` | **Silent.** No requirement states a layout beneath `htmlspec/specs/`, so the source's nested path is preserved rather than flattened |
| N delta files in one change | one `specification.html` | A change directory holds at most one specification artifact |
| `## ADDED Requirements` | `data-change="added"` on the unit | The change state is expressed per unit rather than per file |
| `## RENAMED Requirements` | a `<p data-field="renamed-from">` paragraph stating the `FROM:` heading, on the requirement headed with the `TO:` heading | A rename is a fact about the heading and separate from the change state. … |
| `VERSION` | `VERSION`, carried unchanged | The archival gate reads the version a change records and never derives one from a directory name |
| `config.yaml` context, rules | *nothing* | The declaration carries no authoring rules or project context, so there is nowhere for this to go |
| `evidence/**` | `evidence/**`, byte-identical | Evidence is worth having because nobody edited it. … |
| generated agent instructions | *not migrated* | `htmlspec init` writes its own from the compiled workflow set |
```

**Missing rows:** `## MODIFIED Requirements`, `## REMOVED Requirements`, a task's checkbox, and
`.openspec.yaml`. Three other surveyed kinds also have no row: `converted-document`,
`release-inventory`, and `adoption-marker`. They carry no vocabulary, and this change does not add
them.

`grep -c` of each word in `migrate.md`:

| Word | Lines |
| --- | --- |
| `<ins>`, `<del>`, `change markup` | 0 |
| `outcome`, `partial`, `unchecked` | 0 |
| `data-refs` | 0 |
| `CREATED`, `FOLLOWS`, `.openspec.yaml` | 0 |
| `REMOVED` | 0 |
| `MODIFIED` | 1, inside the RENAMED row |
| `risk`, `verification` | 0 |
| `disposition` | 4, all meaning a plan entity's disposition (lines 183, 188, 250, 251) |

The last row is a naming fact to keep in view. In `migrate.md` and in project-migration's "The plan
accounts for every surveyed entity", a disposition is `converted`, `folded`, `carried`, or `dropped`.
`record-how-a-risk-was-settled` gives a risk a `data-disposition` of `mitigated` or `accepted`.

**The precedent test.** `crates/htmlspec-cli/tests/migrate.rs` lines 257-313,
`a_renamed_section_is_surfaced_and_the_destination_can_express_it`, reads `renamedFrom` from the
published schema and asserts that the `## RENAMED Requirements` row contains `renamed-from` and neither
"open question" nor "no state exists". No test reads any other row.

## 3. The working folder's workaround rules (task 1.4)

Quoted from the files as they are, with their line numbers.

### `brief-draft.md`, the drafting brief

**References.** Lines 241-242:

> Links are relative, and cross-document links are not validated, so get them right: the review
> checks every one against the staged tree.

Lines 264-265:

> - Never give a unit `data-refs` naming a unit in another document: references resolve within one
>   document only, and it is an error.

Line 316:

> - A task naming a requirement may link to it in `specification.html`. Give it no `data-refs`.

**Tasks.** Lines 310-313:

> - `- [x]` is `done` and `- [ ]` is `todo`. A task the source struck through, or marks as dropped,
>   is `dropped` only if the source states why; that reason, verbatim, is its
>   `<p data-field="reason">`. If the source gives no reason, keep the checkbox's state and add a
>   migration note. A dropped task with no reason is invalid, and inventing one is worse.

Convention 2, lines 111-116:

> 2. **Records are carried as written, and so is disagreement with them.** A task marked done that
>    nothing records happening, that was done differently, or that was struck through keeps its
>    source state, and a note names the disagreement and links the evidence. …

**The conflict.** For a task ticked and struck through with a stated reason, convention 2 keeps it
`done` and the tasks rule makes it `dropped`. The staged draft of task 4.5 of
`sync-a-change-into-the-accepted-contract` followed the tasks rule, and `review-log.md` line 29 records
it: "Task 4.5: `[x]` but struck through with a stated reason, so dropped with a note." For a task done
differently, convention 2 is the only rule, and it keeps `done` with a note.

**Verifications.** Lines 376-386:

> … A verification
> unit needs `data-outcome` set to what the record says happened (`passed`, `failed`, `blocked`, or
> `not-run`), … Write none
> where the check only resembles the scenario, where the outcome would be inferred, or where the
> record predates the build.

**Risks.** Lines 294-296:

> … Use a `risk` unit only where the source gives each risk its own heading and
> states its mitigation in a paragraph of its own. Never split a paragraph or a line to make a unit,
> and never write a heading the source did not have.

**Summaries.** Lines 171-175 ask for a change specification's summary where the reader's
`change_subject_needed` says so: "one to three sentences, drawn from what the change's own documents
say". Nothing says where a word-level change inside a restated requirement goes, and five drafted
summaries narrate one (`mark-the-words-a-modification-changed`'s evidence, section 4).

**Creation dates.** Lines 91-93, "Each change's `.openspec.yaml` was carried byte-identical to
`evidence/.openspec.yaml` in its change. Where it named a predecessor, a `FOLLOWS` file was also
written. … You do not write either.", and line 372, "`evidence/.openspec.yaml` is the change's creation
record and gets none."

### `make_decisions.py`, which writes the plan's decisions

- **Decision 1's rationale** (lines 49-54), written into all 15 entities: "The destination has no field
  for a start date."
- **Decision 1 as reviewed** (lines 82-95), question and answer: "Carry each one."
- **The rules-table row** (lines 225-227), which contradicts decision 1:

  ```python
      ("<code>.openspec.yaml</code>", "folded or dropped",
       "a <code>FOLLOWS</code> file, or none",
       "artifact-kinds' \"A change states the change it follows\"."),
  ```

- **Convention 2** (lines 159-164): "A task marked done that nothing records happening, done
  differently, or struck through keeps its source state. A note names the disagreement and links the
  evidence."
- **Convention 4** (lines 169-175): each modified requirement "is compared with its previous statement
  in the archive, the earlier change's delta, replayed in archive order … The source archive is the
  evidence, not git history."
- **Convention 7** (lines 187-192): "A scenario withdrawn without a delta, as in
  `distinguish-command-names-from-skill-names`, is an open question for its change."

### `replay.py` and `replay.json`

The replay holds each requirement's previous text in memory (`contract`, set at lines 163 and 205)
and saves it only for some units. The fields each unit carries in `replay.json`:

| Unit | Count | Fields saved |
| --- | --- | --- |
| added | 137 | `heading`, `identifier`, `renamed_from`, `scenarios`, `state`, `statement` |
| modified | 33 | `heading`, `identifier`, `previous`, `renamed_from`, `scenarios`, `state`, `statement` |
| modified, with `kept_by_merge` | 1 | the same, and `kept_by_merge` |
| removed | 1 | the same, and `previous_statement`, `previous_scenarios`, `removal_text` |

**For a modified unit, `previous` is a change's name and `statement` is the word `modified` or
`unchanged`. The text it replaced is not saved.** A modified scenario is saved as a heading and a
state. The replay's own comparison finds that 33 of the 34 modified requirements changed their
statement text, and 13 scenarios changed their entries.

`KEPT_BY_MERGE` (lines 86-96) names the two scenarios `separate-artifact-kinds`' merge kept. The replay
adds them to the replayed contract (lines 201-204), and **leaves them out of the unit's `scenarios`**.
The unit carries them only as a list of headings:

```
"kept_by_merge": ["Unknown subcommand", "Discoverable help"],
"scenarios": [{"heading": "Validating a project", "state": "unchanged"}, {"heading": "Validating one document", "state": "unchanged"},
              {"heading": "Initializing a proposal", "state": "added"}, {"heading": "Initializing a specification", "state": "added"},
              {"heading": "Initializing a change", "state": "added"}]
```

The replay reports 0 anomalies and 0 disagreements with the accepted specifications.

### `build_packets.py`

For a modified requirement, the packet gives `previous_statement_in`, a change's name (line 153), and
the drafter reads that change's delta for the text. `carried_beside_the_documents` (line 204) lists
the carried files and `FOLLOWS`.

### `check_draft.py`

It checks validation, the masthead, unit identifiers and states against the packet, source sentences,
links, and carried digests. It reads no `outcome`, no `ins` or `del`, and no `data-refs`.

## 4. The `~~` false flag (task 1.4)

`check_draft.py` line 437:

```python
        m = re.match(r"^\s*[-*]\s+\[([ xX~])\]\s+(~~)?\s*([0-9]+(?:\.[0-9A-Za-z]+)*)", line)
```

The strike is looked for before the task number. The archive's one struck-through task has it after:

```
$ python3 …   # the regex applied to line 33 of 2026-08-31-sync-a-change-into-the-accepted-contract/tasks.md
'- [x] 4.5 ~~Add the subcommand to `invocations.rs`, whose ha'
checkbox: x struck: False number: 4.5
$ grep -c '~~' openspec/changes/archive/*/tasks.md | grep -v ':0'
openspec/changes/archive/2026-08-31-sync-a-change-into-the-accepted-contract/tasks.md:1
```

So `struck` is false, and the tolerance at line 501 (`not (struck and u["state"] == "dropped")`) does
not apply. `check-all.txt` line 170 records the result, and it reproduces:

```
$ python3 check_draft.py ~/projects/html-spec-migration 2026-08-31-sync-a-change-into-the-accepted-contract
=== 2026-08-31-sync-a-change-into-the-accepted-contract: 1 finding(s)
  …
  tasks: 31 task unit(s), source has 31
  …
  ! tasks: task-4-5 is dropped, source checkbox says done
carried files: all intact
exit 0
```

The draft is right and the check is wrong: the source ticks the box, strikes the words, and says "**No
action: the premise was wrong.**"

## 5. Per-feature counts in the staged tree (task 1.5)

Each count is the named change's, from its own evidence. Five were run again here, against
`~/projects/html-spec-migration/staged/`, with the script each evidence file gives.

| Feature | Count | Source | Re-run here |
| --- | --- | --- | --- |
| task units linking to a unit in another document | 54, in 15 changes (66 links) | `resolve-a-reference-across-its-change`, step 3 | **Reproduced**: `task: 54 units, 66 links, 15 changes` |
| decision units doing the same | 2 (2 links) | the same | **Reproduced**: `decision: 2 units, 2 links, 2 changes` |
| of the 56, linking to their change's specification / an accepted one / another change's | 31 / 23 / 2 | the same | **Reproduced** |
| links to a withdrawn unit | 1, task 5.1 of `distinguish-command-names-from-skill-names` | the same | **Reproduced** |
| modified requirements / modified scenarios | 34 in 19 changes / 13 | `mark-the-words-a-modification-changed`, section 4 | **Reproduced**: `[(('change-spec', 'requirement'), 34), (('change-spec', 'scenario'), 13)]` |
| task units with a migration note | 76, of which 74 `done`, 1 `todo`, 1 `dropped` | `give-a-task-an-outcome`, section 2 | **Reproduced**, by `grep` (76) and by `project-model` (970 tasks: 965 done, 4 dropped, 1 todo) |
| of those, done otherwise than their words say | 33 | the same, one reader's classification | not re-derived: a reading, not a count |
| verification units removed in review for covering part of a scenario | 2 | `record-a-partial-verification`, section 1 | not re-derived: from `review-log.md` lines 124 and 126 |
| verification units kept as `passed` while covering part | 2 | the same, section 2 | the two units are `passed` at lines 365 and 373 of `content-address-runtime-assets/specification.html` |
| verification outcomes | 92 `passed`, 1 `not-run`, 1 `blocked` | the same, section 3 | **Reproduced** with `grep` over files outside `evidence/` |
| declines that read as a check of part of a scenario | 5 of 13 | the same, section 4 | not re-derived: a reading |
| drafted risk units | 1, in `distinguish-a-requirement-from-an-example/design.html` | `record-how-a-risk-was-settled`, section 3 | **Reproduced**: the only file outside `evidence/` holding `class="risk"` |
| headed accepted risks with no mitigation, blocked by the missing value | 5 | the same, section 6 | not re-derived |
| `.openspec.yaml` with `created:` / with `follows:` / archives with none | 15 / 3 / 9 | `record-when-a-change-was-created`, section 1 | **Reproduced** in section 1 above |
| MODIFIED blocks that leave out a scenario the requirement held | 3, one kept by the merge and two withdrawn | `state-a-modified-requirement-whole`, section 3 | `replay.json` names the kept one, as section 3 above shows |

## 6. Promotion over the staged tree, and where a staged document sits (task 1.6)

Promotion into an empty scratch destination, checking only:

```
$ M=~/projects/html-spec-migration
$ time ./target/debug/htmlspec migrate promote $M/plan/plan.html --survey $M/survey.json \
    --staged $M/staged --into <scratch>/dest --check
301 surveyed entities, 301 plan items
every surveyed entity has a disposition
checked 109 document(s) and 8 stored asset(s) across the complete staged project
every check passed and nothing was moved. Run this without --check to promote
exit 0      21.258 total
$ ls -A <scratch>/dest        # empty
```

```
$ cd $M/staged
$ find . -name '*.html' | wc -l                                  # 116
$ find . -name '*.html' -not -path '*/evidence/*' | wc -l        # 109
```

**The check read exactly the 109 files outside a change's evidence.** `conformance`
(`crates/htmlspec-cli/src/commands.rs`, lines 3021-3090) walks with `discover::documents(staged)`, and
that function excludes evidence only after `ProjectRoot::resolve` finds the project the walk's root
belongs to. So the staged tree resolved as its own project. Two facts make that hold for every staged
document, not only for the walk:

- `ProjectRoot::resolve` (`project.rs` lines 151-171) ascends from a path to the first directory
  holding `htmlspec/`, stopping at `.git`. From any staged document it reaches `staged/` first. Neither
  the working folder nor any directory above it holds `.git`.
- `Staged::resolve` (`staging.rs` lines 229-303) refuses a staging location unless
  `ProjectRoot::resolve` finds the location itself (lines 244-254). Conformance runs only after that.

`conformance` passes each document to `validate_source(&source, &store, …)` with
`store_for(path)`. `resolve-a-reference-across-its-change`'s design names promotion's `conformance`
among the callers of `validate_source`, which that change makes derive a document's placement and
reach from its path through `ProjectRoot` (its decisions 3 and 4, and its task 3.6). **So placement
resolves inside the staged tree, by the mechanism measured here, and a staged reference reaches the
staged change's specification and the staged accepted specifications.** Nothing in the staging code
needs to change for it. What no test states today is that a reference across documents resolves in
a staged tree as it does after promotion.

## 7. What the three pilots exercise

`queue.md` names the pilots: `spec-runbook-portability`, `2026-09-26-carry-a-runbook-to-another-machine`,
and `2026-08-30-finish-the-archival`. `brief-draft.pilot.md` is the brief before the revision they
prompted. Counted from their packets and staged documents (`pilots.py`, appendix A):

| Pilot | What it holds | Which new mapping it exercises |
| --- | --- | --- |
| `carry-a-runbook-to-another-machine` | 9 added and 1 removed requirement; 38 added and 2 removed scenarios; 1 task note (7.3, no record of its result); `.openspec.yaml` with `created: 2026-09-17` and `follows: log-a-runbook-run`; in its specification, 2 links to requirements in other changes' specifications and 1 to an accepted one | the creation date and the predecessor |
| `finish-the-archival` | 3 modified requirements, all with changed statement text, one renamed; 25 unchanged and 34 added scenarios; 9 task notes, 5 of them done otherwise than the words say (3.3, 4.8, 4.12, 7.3, 7.4); 14 task links to its own `specification.html#…`; in the design, 1 link to its own specification, 2 to accepted ones, and 1 to a section of itself; 3 headed risks whose measure shares the statement's paragraph; no verification unit; no `.openspec.yaml` | change markup, outcomes, and references |
| `spec-runbook-portability` | an accepted specification: 4 requirements, 17 scenarios, 1 link to an archived change, no `data-refs` | that an accepted specification carries no change markup and no rename paragraph |

**No pilot holds a partial check, a risk unit, or a scenario a merge kept.** The partial checks are in
`content-address-runtime-assets` (the two kept as `passed`), `confine-the-asset-check-to-the-project`,
and `record-a-renamed-requirement` (the two removed). The one drafted risk unit, and five headed
accepted risks, are in `distinguish-a-requirement-from-an-example` and three other designs. The kept
scenarios are in `separate-artifact-kinds`.

## 8. The source the plan was written from

The working folder's `survey.json` is the payload the reviewed plan was checked against:

```
$ stat -f '%Sm' ~/projects/html-spec-migration/survey.json
Sep 29 18:53:32 2026
$ git log -1 --format='%h %ci' 6a591bd
6a591bd 2026-09-29 18:51:23 -0500
$ python3 -c "import json; p=json.load(open('…/survey.json')); print(len(p['entities']), len(p['changes']))"
301 24
```

It was written two minutes after `6a591bd` was committed, and it holds 301 entities in 24 changes,
the same 301 that promotion reports as surveyed and planned in section 6. So `6a591bd` is the source
the plan describes. Today's survey of the working tree holds 365 entities in 35 changes, because this
set's eleven changes are in `openspec/changes/`, and after archival they join the archive the plan
does not cover.

## 9. A carried-forward scenario whose identifier is not its accepted counterpart's

`state-a-modified-requirement-whole` asks each scenario a change carries forward, alters, or
withdraws to carry the identifier its accepted counterpart carries. Its design marks that sentence
for review (decision 4). The plan's decision 2 derives a scenario's identifier from its heading, and
qualifies it with its requirement's heading only in a document where two scenarios share a heading:
"The same scenario keeps the unqualified identifier in any document where it has no twin."

`scenario_ids.py` (appendix B) compares every scenario that a staged change specification states,
other than `added`, inside a requirement marked `modified`, with the scenario of the same heading under
the same requirement in the staged accepted specifications:

```
$ cd ~/projects/html-spec-migration/staged && python3 scenario_ids.py
carried-forward, altered, or withdrawn scenarios with an accepted counterpart: 189
with no accepted counterpart today: 2
whose identifier differs from the accepted counterpart's: 1
   ('2026-09-26-record-a-renamed-requirement', 'req-the-command-surface-reports-whether-a-change-reached-the-accepted-contract', 'scenario-an-invalid-specification', ['scenario-the-command-surface-reports-whether-a-change-reached-the-accepted-contract-an-invalid-specification'])
```

The two with no counterpart are the two scenarios withdrawn by a restatement, which no accepted
specification holds today. **One scenario differs.** Accepted toolchain-packaging holds "An invalid
specification" under two requirements, so the plan qualifies both there. `record-a-renamed-requirement`
restates one of those requirements, with that scenario `unchanged`, in a document where the scenario
has no twin, so it keeps the unqualified identifier. The same scenario appears under
`scenario-an-invalid-specification` in `sync-a-change-into-the-accepted-contract` and
`finish-the-archival` too, but there it is `added`, which the sentence does not cover.

The two scripts that sections 7 and 9 name are reproduced below, so the measurement does not depend on the scratch directory surviving.

## Appendix A: `pilots.py`

```python
"""What the three pilot packets would exercise of the extended vocabulary. Read only.
Run from ~/projects/html-spec-migration."""
import json, re, os
from collections import Counter

def count(pattern, text):
    return len(re.findall(pattern, text))

TASK = r'class="task"'
RISK = r'class="risk"'
VERIFY = r'class="verification"'
TO_SPEC = r'href="[^"]*specification\.html#'
TO_ACCEPTED = r'href="[^"]*specs/[^"]*#'
TO_OTHER_SPEC = r'href="\.\./[^"]*#'

for name in ["2026-09-26-carry-a-runbook-to-another-machine", "2026-08-30-finish-the-archival",
             "spec-runbook-portability"]:
    p = json.load(open(f"packets/{name}.json"))
    print("==", name, p["assignment"])
    if p["assignment"] == "change":
        reqs = [r for c in p["specification"]["capabilities"] for r in c["requirements"]]
        print("  requirement states:", dict(Counter(r["state"] for r in reqs)))
        print("  modified, with their statement's state:",
              [(r["identifier"], r.get("statement"), r.get("renamed_from")) for r in reqs if r["state"] == "modified"])
        print("  scenario states:", dict(Counter(s["state"] for r in reqs for s in r["scenarios"])))
        print("  follows:", p["follows"], "| carried:", p["carried_beside_the_documents"])
        root = "staged/" + p["staged_directory"]
        for doc in ["tasks.html", "design.html", "specification.html"]:
            t = open(os.path.join(root, doc)).read()
            print(f"  {doc}: tasks {count(TASK, t)}, migration notes {t.count('Migration note')}, "
                  f"risks {count(RISK, t)}, verifications {count(VERIFY, t)}, data-refs {t.count('data-refs=')}, "
                  f"links to a specification.html# {count(TO_SPEC, t)}, links to specs/…# {count(TO_ACCEPTED, t)}")
    else:
        t = open("staged/" + p["destination"]).read()
        REQ = count(r"class=.requirement.", t); print(f"  requirements {REQ}, "
              f"links to another accepted spec# {count(TO_OTHER_SPEC, t)}, data-refs {t.count('data-refs=')}")
```

## Appendix B: `scenario_ids.py`

```python
"""For each scenario a staged change specification carries forward, alters, or withdraws inside a
modified requirement, compares its identifier with the identifier the staged accepted specification
gives the scenario of the same heading under the same requirement. Read only; run from staged/."""
import glob, json, subprocess, pathlib, collections
tool = str(pathlib.Path.home() / "projects/html-spec-migration/tools/project-model/target/release/project-model")
def models(paths):
    return json.loads(subprocess.run([tool, *paths], capture_output=True, text=True, check=True).stdout)
def heading(unit):
    t = " ".join(unit.get("title", "").split())
    return t[: -len(unit["id"])].strip() if t.endswith(unit["id"]) else t
accepted = {}
for path, m in models(sorted(glob.glob("htmlspec/specs/**/spec.html", recursive=True))).items():
    for r in m.get("requirements", []):
        for s in r.get("scenarios", []):
            accepted.setdefault((r["id"], heading(s)), []).append(s["id"])
checked, differ, absent = 0, [], 0
for path, m in models(sorted(glob.glob("htmlspec/changes/archive/*/specification.html"))).items():
    for r in m.get("requirements", []):
        if r.get("change") != "modified":
            continue
        for s in r.get("scenarios", []):
            if s.get("change", "unchanged") == "added":
                continue
            ids = accepted.get((r["id"], heading(s)))
            if ids is None:
                absent += 1
                continue
            checked += 1
            if s["id"] not in ids:
                differ.append((path.split("/")[3], r["id"], s["id"], ids))
print(f"carried-forward, altered, or withdrawn scenarios with an accepted counterpart: {checked}")
print(f"with no accepted counterpart today: {absent}")
print(f"whose identifier differs from the accepted counterpart's: {len(differ)}")
for d in differ:
    print("  ", d)
```
