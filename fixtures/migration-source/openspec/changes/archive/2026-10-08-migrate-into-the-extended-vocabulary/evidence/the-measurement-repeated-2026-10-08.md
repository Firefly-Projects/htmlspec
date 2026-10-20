# The measurement repeated, before any change

**Date:** 2026-10-08
**Tasks:** 1.0, 1.2, 1.3, 1.4, 1.5, and 1.6.
**Binary:** `htmlspec 0.24.0 (b768860d5b82)`, built in this change's worktree from `main` at
`b768860`, before any edit.

Six of the nine changes this one builds on are archived at `b768860`:
`reject-change-markup-inside-a-unit`, `state-a-modified-requirement-whole`,
`see-a-modification-the-merge-did-not-take`, `resolve-a-reference-across-its-change`,
`mark-the-words-a-modification-changed`, and `show-what-a-modification-changed`. The other three,
`record-a-task-done-differently`, `record-a-partial-verification`, and
`record-how-a-risk-was-settled`, are active, and are being implemented beside this change, below it
in one stack. Their names are read from their own change directories here.

The working folder was copied, without `staged/` and `tools/*/target/`, to
`scratchpad/migrate-into-the-extended-vocabulary/working-folder-before/` before anything was run,
with a `sha256` of each of its 162 files. The staged tree's 291 files were hashed too, and those
hashes still match after every command below.

## Summary

| Question | Answer |
| --- | --- |
| Does every name this change uses match its source? | **Yes.** The six archived ones match the code and the accepted specifications. The three active ones match their own specifications and designs. |
| Did an archived change modify either requirement this change modifies? | **No.** `openspec/specs/project-migration/spec.md` was last edited at `6a591bd`. A word diff of each MODIFIED block against the accepted text shows insertions only. |
| Did `state-a-modified-requirement-whole` keep the scenario-identifier sentence? | **Yes**, in the accepted artifact-kinds, with the scenario "A scenario keeps its identifier". |
| Does `show --diff` still refuse an archived change? | **Yes**, exit 2, with the reason that the accepted specifications hold every later change's edit. |
| Does the survey read a change's configuration? | **No.** 30 archives now, 21 with a file, 21 stating `created:`, 6 stating `follows:`, 9 with none. The old 24 still give 15, 3, and 9. |
| Does `migrate.md` have the four rows? | **No.** The table is unchanged since `6a591bd`, and every word count is as measured. |
| Do the working folder's rules sit at the lines the tasks cite? | **Yes, every one.** The `~~` false flag reproduces. |
| Do the five counts reproduce? | **Yes, all five, with no count moved.** |
| Does promotion's check read the staged tree as a project? | **Yes.** Exit 0, 109 documents, which is every HTML file outside `evidence/`. It took 39.9 s, against 21.3 s on 2026-09-30. |
| What does the design get wrong? | **One count.** 55 of the 56 linking units resolve as references, not 53. Section 1.0 has the correction. |

## 1.0 The names this change uses

| Name | Where it is, at `b768860` | Source |
| --- | --- | --- |
| `schema/change-markup` | `CHANGE_MARKUP` in `crates/htmlspec-validate/src/rules/change_markup.rs` line 36 | archived `reject-change-markup-inside-a-unit` |
| `untakenContent`, `untakenRemovals` | payload keys in `crates/htmlspec-cli/src/commands.rs` lines 2638-2643 and 4818-4824 | archived `see-a-modification-the-merge-did-not-take` |
| `Placement` and the order a reference reaches its places | `Loose`, `ChangeDocument`, `ChangeSpecification`, `AcceptedSpecification`, and `RunbookDocument` in `crates/htmlspec-validate/src/references.rs` lines 23-34. A `ChangeDocument`, active or archived, reaches its change's specification, then every accepted specification. A change's specification and an accepted specification reach the accepted specifications. The other two reach only themselves | archived `resolve-a-reference-across-its-change`, and document-schema's "Cross-references address units by identifier" |
| `deletions`, and where `ins` and `del` are permitted | `deletions` on a requirement and a scenario in `crates/htmlspec-parse/src/model.rs` lines 171 and 292. The marks are permitted "inside a requirement or a scenario marked `modified`, in a change's specification", and nowhere else | archived `mark-the-words-a-modification-changed`, and document-validation's "Change markup in or around a semantic unit is rejected" |
| `htmlspec show <change> --diff` | the `show` subcommand | archived `show-what-a-modification-changed` |
| `deviation`, `schema/deviation-not-done` | a task's `data-field="deviation"` paragraph, projecting into its `deviation` field | active `record-a-task-done-differently`: its document-schema delta and its decisions 1, 2, and 7 |
| `partial`, `unchecked`, `schema/missing-unchecked` | the fifth outcome, `data-field="unchecked"` paragraphs projecting into the verification's `unchecked` array | active `record-a-partial-verification`: its document-schema delta and its decisions 1 to 4 |
| `data-response`, `accepted`, `mitigated` | a risk's `data-response`, projecting into its `response` field, `mitigated` by default | active `record-how-a-risk-was-settled`: its document-schema delta and its decisions 1 to 5 |

Every requirement heading the delta specification cites was checked with `grep -F` against
`openspec/specs/` and the three active changes' deltas. Each exists, under the capability named,
except this change's own five added headings. The one quoted sentence, project-bootstrap's "A
declaration no command reads states nothing that can be wrong, so nothing reports it when it is",
is in the accepted text once.

**No archived change modified the two requirements.** `grep -l` over
`openspec/changes/archive/*/specs/project-migration/spec.md` names `migrate-an-openspec-project`,
which added them, and `record-a-renamed-requirement`, which this change was written against.
`git log` of the accepted file ends at `6a591bd`. Each MODIFIED block was cut out and compared
with the accepted requirement by `git diff --no-index --word-diff=plain`. Both diffs hold only
insertions: 40 lines for the collapse requirement and 29 for the open-question requirement, and no
`[-…-]`.

**The scenario-identifier sentence was kept.** The accepted artifact-kinds' "A change's
specification states a modified requirement whole" reads: "The requirement SHALL carry the
identifier the accepted requirement carries, and each scenario it carries forward, alters, or
withdraws SHALL carry the identifier its accepted counterpart carries." Its scenario "A scenario
keeps its identifier" holds it. So the clause in the collapse requirement, the paragraph in design
decision 3, and the amendment to the plan's decision 2 in task 5.1 all stay. Design decision 3 is
edited only to say that the sentence was kept.

**`show --diff` refuses an archived change.** In a scratch project made with `init`, a change moved
to `htmlspec/changes/archive/2026-10-08-a-change/`:

```
$ htmlspec show archive/2026-10-08-a-change --diff
htmlspec: …/htmlspec/changes/archive/2026-10-08-a-change is an archived change. The accepted
specifications hold its edit together with every later change's, so a comparison with them shows a
later change's edit, reversed, where there is one, and nothing where there is none. `show --diff`
compares a change before its merge. What an archived change modified is in its specification's
marks, where it has them, and in the repository's history
exit 2
```

So task 6.3's rebuilt copy, an active change beside the accepted text it modified, is still the
only way to use the view on a migrated modification, as design decision 4 says.

**The correction: 55 of the 56 linking units resolve, not 53.** Design decision 6 says "53 of the
56 linking units resolve, 30 in their change's specification and 23 in an accepted one", and that
"two units name another change's requirement, so those three stay links". It quotes
`resolve-a-reference-across-its-change`'s task 5.3 as it was written. That change's own measurement
of 5.3, `evidence/the-rule-measured-2026-10-03.md` in its archive, found the task's expectation
wrong for those two: task 9.1 of `2026-08-17-project-bootstrap-cli` names
`req-archival-requires-the-project-version-to-advance`, and task 8.3 of
`2026-09-11-migrate-an-openspec-project` names
`req-the-html-to-model-projection-is-published-as-a-normative-specification`. The staged accepted
`project-bootstrap` and `document-schema` declare those identifiers, so each reference resolves in
the third place document-schema's rule lists. Only task 5.1 of
`distinguish-command-names-from-skill-names` fails, with `refs/removed-target`.

The delta specification was already right about the rule. Its scenario "A task that names another
change's requirement" is about a requirement "only another change's delta states", and these two
are also accepted. One phrase in its list of units a document cannot reach, "a unit in another
change's documents", could be read as covering them, and it now reads "a unit that only another
change's documents declare". Design decision 6 now gives the measured result. Whether a task that
links to another change's specification should carry the reference because the accepted
specifications hold the same identifier is put to the contributor at review.

## 1.2 The survey at HEAD

```
$ ./target/debug/htmlspec migrate survey --json . > survey-head.json     # exit 0, 97,582 bytes
$ ./target/debug/htmlspec migrate survey .                               # exit 0
422 entities in …/migrate-into-the-extended-vocabulary
    13  accepted-specification
     1  adoption-marker
    26  change-configuration
   105  change-document
     1  configuration
     4  converted-document
    64  delta-specification
   168  evidence
    30  recorded-version
    10  release-inventory

35 changes, 30 archived
…
the project's configuration sets: schema
```

```
top-level keys: ['changes', 'configuration', 'counts', 'entities', 'greatestVersion', 'root', 'sharedVersions']
keys on each change: ['archived', 'deltaSections', 'name', 'version']
any change carrying created/follows/configuration: False
change-configuration entities: 26 archived: 21
```

**Nothing moved in what the survey reads.** The 35 changes are 30 archived and 5 active, and each
active change has a `.openspec.yaml`, so there are 26 `change-configuration` entities.

```
$ ls -d openspec/changes/archive/*/ | wc -l                             # 30
$ ls openspec/changes/archive/*/.openspec.yaml | wc -l                  # 21
$ grep -l '^created:' openspec/changes/archive/*/.openspec.yaml | wc -l # 21
$ grep -l '^follows:' openspec/changes/archive/*/.openspec.yaml | wc -l # 6
```

The 24 archives the measurement read still give **15, 3, and 9**, the same files and values. The
six archived since each hold a file:

| Archive | `created:` | `follows:` |
| --- | --- | --- |
| `2026-10-02-reject-change-markup-inside-a-unit` | 2026-09-30 | |
| `2026-10-02-see-a-modification-the-merge-did-not-take` | 2026-09-30 | `state-a-modified-requirement-whole` |
| `2026-10-02-state-a-modified-requirement-whole` | 2026-09-30 | |
| `2026-10-04-resolve-a-reference-across-its-change` | 2026-09-30 | |
| `2026-10-07-mark-the-words-a-modification-changed` | 2026-09-30 | `reject-change-markup-inside-a-unit` |
| `2026-10-08-show-what-a-modification-changed` | 2026-09-30 | `mark-the-words-a-modification-changed` |

Every file in the archive sets `schema: spec-driven` and `created:`, and nothing else but the six
`follows:`. The nine archives with no file are the same nine.

## 1.3 `migrate.md`

`git log` of `crates/htmlspec-assets/instructions/migrate.md` ends at `6a591bd`. The table is at
lines 80-89, with the same eight rows the measurement quotes, and no row for
`## MODIFIED Requirements`, `## REMOVED Requirements`, a task's checkbox, or `.openspec.yaml`.

| Word | `grep -c` |
| --- | --- |
| `<ins>`, `<del>`, `change markup` | 0 |
| `outcome`, `deviation`, `partial`, `unchecked` | 0 |
| `data-refs`, `data-response` | 0 |
| `CREATED`, `FOLLOWS`, `.openspec.yaml`, `created:` | 0 |
| `REMOVED`, `risk`, `verification` | 0 |
| `MODIFIED` | 1, inside the RENAMED row |
| `disposition` | 4, at lines 183, 188, 250, and 251, each a plan entity's disposition |

**All four rows are still missing.**

## 1.4 The working folder's rules, and the `~~` false flag

Every line the tasks cite holds the rule quoted in the measurement's section 3:

- `brief-draft.md`: lines 241-242, 264-265, and 316 (links); 310-313 and 111-116 (tasks and
  convention 2); 376-386 (verifications); 294-296 (risks); 171-175 (summaries); 91-93 and 372
  (creation dates); 94-97 (scenario identifiers).
- `make_decisions.py`: 16-19 (`change-config.tsv`); 49-54 (`EVIDENCE_REASON`); 58-60 (each restated
  date); 82-95 (decision 1); 96-110 (decision 2); 159-164, 169-175, and 187-192 (conventions 2, 4,
  and 7); 225-227 (the rules-table row).
- `stage_carry.py`: 52-63, the second reading of each `.openspec.yaml`.
- `replay.py`: `contract` set at 163 and 205, `KEPT_BY_MERGE` at 92-96, the kept scenarios added to
  the contract at 201-204 and to no unit.
- `build_packets.py`: `scenario_id` at 97-98, `previous_statement_in` at 153, the carried list at
  204.
- `check_draft.py`: `source_tasks` at 434-444, its pattern at 437, the tolerance at 501.

No cited line moved, so tasks 5.1 to 5.6 need no correction.

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

**The false flag reproduces.**

## 1.5 The five counts

Each script is the one the cited evidence gives, run against `~/projects/html-spec-migration/staged/`
with the working folder's `project-model` where the script uses it.

| Count | Script | Result |
| --- | --- | --- |
| task and decision units linking to a unit in another document | `resolve-a-reference-across-its-change`, `the-measurement-2026-09-30.md`, step 3 | `task: 54 units, 66 links, 15 changes`, `decision: 2 units, 2 links, 2 changes`, 31 / 23 / 2 by target, 1 removed target. **Unchanged** |
| modified requirements and scenarios | `mark-the-words-a-modification-changed`, appendix B | `34 modified requirements in 19 changes`, 13 modified scenarios. **Unchanged** |
| task units with a migration note | `record-a-task-done-differently`, section 2 | 970 tasks (965 done, 4 dropped, 1 todo), 76 with a note: 74 `done`, 1 `todo`, 1 `dropped`. `grep` finds the same 76. **Unchanged** |
| verification outcomes | `record-a-partial-verification`, section 3 | 109 documents, 21 with verifications, 94 units: 92 `passed`, 1 `not-run`, 1 `blocked`. **Unchanged** |
| drafted risk units | `record-how-a-risk-was-settled`, section 3 | one, `risk-an-agent-that-cites-a-requirement-that-does-not-support-the-claim` in `2026-09-06-distinguish-a-requirement-from-an-example/design.html`, with a mitigation. **Unchanged** |

No count moved, so no design decision needs another look on this account. Decisions 4 and 5 rest on
the second and third rows, decision 6 on the first, decision 8 on the fourth, and decision 9 on
the fifth.

## 1.6 Promotion over the staged tree

```
$ time ./target/debug/htmlspec migrate promote $M/plan/plan.html --survey $M/survey.json \
    --staged $M/staged --into <scratch>/dest16 --check
checked 109 document(s) and 8 stored asset(s) across the complete staged project
301 surveyed entities, 301 plan items
every surveyed entity has a disposition
every check passed and nothing was moved. Run this without --check to promote
real 39.90   user 39.65   sys 0.21
exit 0
$ ls -A <scratch>/dest16 | wc -l                                          # 0
$ find . -name '*.html' | wc -l                                           # 116, in staged/
$ find . -name '*.html' -not -path '*/evidence/*' | wc -l                 # 109
```

**The document count is 109, which is every HTML file outside a change's evidence.** So the staged
tree still resolves as its own project, by the mechanism the measurement's section 6 describes.

**The check now takes twice as long.** On 2026-09-30 it took 21.3 s, and
`resolve-a-reference-across-its-change` measured 19.8 to 20.0 s on 2026-10-03. Here it took 39.65 s
of user CPU. The `htmlspec 0.24.0 (facdaca)` binary in the main checkout takes 39.41 s on the same
tree, so the cost arrived at or before `facdaca` and is not this change's. It is recorded for
whoever runs promotion, and this change does not pursue it.
