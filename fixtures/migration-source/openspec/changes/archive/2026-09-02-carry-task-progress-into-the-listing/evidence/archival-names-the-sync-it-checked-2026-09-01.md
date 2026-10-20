# Archival names the sync it already checked

Recorded 2026-09-01 on branch `carry-task-progress-into-the-listing`, for work requested
mid-implementation and outside this change's tasks document.

## What was asked and what was already there

The request: archival should check whether `sync` has been done, warn if not, and ask whether to
sync first.

The check already existed. `findings_for` computes `merge_state(project, specification).absent` —
the requirement identifiers the change states that no accepted specification holds — which *is*
the question `sync` answers. Archival has reported it since 0.6.0 and refuses to perform the
merge itself, which the accepted requirement `Archival assesses the merge and never performs it`
requires.

So nothing needed recomputing, and recomputing would have been the fault this whole change is
about: a second answer to a question the program already answers, able to disagree with the
first. `Findings::needs_sync()` is a predicate over the finding already collected.

## What was missing

Two things, both about what the contributor is told rather than what is detected.

**The note named an operation without naming the command.** It said "Apply the change's contract
before archiving" — the exact wording that, before 0.7.0, sent a reader searching a ten-subcommand
surface for an operation that did not exist. `sync` now exists, and the note did not say so.

**The question invited the wrong answer.** "Archive anyway, over 2 finding(s)?" asks whether to
proceed without saying what proceeding skips. A contributor who says yes archives a change whose
contract never reached the project's specifications.

## What it says now

Unsynced, in check mode:

```
note: 1 task(s) are not done or dropped: task-example
note: 1 requirement(s) this change states appear in no accepted specification: req-example.
This change has not been synced. Run `htmlspec sync --change unsynced-change` to see what
remains, apply the change's contract, or confirm each one was withdrawn deliberately
ready with questions: this change has not been synced, and archival would report the findings
above and ask
```

The question a contributor is asked before archival proceeds:

```
This change has not been synced. Archive anyway, over 2 finding(s)? Answering no leaves
everything as it is, so you can run `htmlspec sync --change unsynced-change` first
```

Declining is named as costing nothing, because it does: archival moves nothing until it has an
answer, and the existing `Cancelled` path already left the project untouched.

## The structured payload states it as a field

An agent driving archival cannot answer a prompt, so the condition the prompt asks about is a
field rather than something to infer from an empty array:

```json
{ "needsSync": true, "unmergedRequirements": ["req-example"], "moved": false }
```

## It goes quiet when it does not apply

The same change after its specification was copied into `specs/`:

```
note: 1 task(s) are not done or dropped: task-example
ready with questions: archival would report the findings above and ask
```

`needsSync` is `false`, `unmergedRequirements` is empty, and no mention of syncing appears. This
is the part worth checking rather than assuming: a warning that shows up unconditionally is one a
reader learns to skip, and the one time it mattered would be skipped with it.

`htmlspec sync` on the same change agrees, which is the property that matters — both read
`merge_state`:

```
1 of 1 requirement(s) this change states have reached the accepted specifications
  reached  req-example
  the change's contract has been applied in full
```

## What was deliberately not built

**Archival does not run the sync.** `artifact-kinds` names updating the accepted specifications as
the only true merge among the four agent-driven operations, and the accepted contract states that
`sync` itself must not modify an accepted specification or propose the text of a merge. An
archival that offered to perform the merge would be the program editing the project's contract
unattended. It names the command; the contributor or agent runs it.

**The prompt stays a yes-or-no.** A three-way choice — archive, sync first, cancel — would need a
new prompt primitive, and "sync first" and "cancel" have identical mechanics: stop, move nothing,
leave the project as it was. The difference is what the contributor does next, which is why the
declining path names the command instead of merely reporting a stop.

## Tests

Five, in `crates/htmlspec-cli/src/commands.rs`:

| Test | What it pins |
| --- | --- |
| `an_unmerged_requirement_is_an_unsynced_change` | an open task is not an unsynced contract |
| `the_question_names_sync_for_an_unsynced_change` | the question names the command and says declining is safe |
| `the_question_does_not_mention_sync_when_the_contract_reached_the_specifications` | no warning where it does not apply |
| `the_question_counts_every_finding` | both kinds of finding are counted, whichever wording |
| `the_reported_finding_names_the_sync_command` | the note names `htmlspec sync --change <name>` |
| `the_payload_states_whether_the_change_needs_syncing` | `needsSync` is true and false in the right cases |

`question_for` is a pure function of the findings so the wording is testable with no terminal —
the same separation `decision` already has from the prompt that feeds it.
