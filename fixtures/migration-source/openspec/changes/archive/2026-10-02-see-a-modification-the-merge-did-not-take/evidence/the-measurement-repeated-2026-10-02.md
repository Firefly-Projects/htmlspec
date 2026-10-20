# The measurement, repeated before anything changed

**Date:** 2026-10-02
**Tasks:** 1.1 to 1.3
**Binary:** `htmlspec 0.20.0 (715480415d8c-dirty)`, built from `7154804`, the merge of #41. The
`-dirty` mark is this change's own proposal, design, and tasks, corrected before the measurement ran
(see "Before the measurement"), and this file. No compiled source differs from HEAD.

## Before the measurement

The proposal, the design's Migration Plan, and task 6.2 each said the contract version stays `0.2.0`.
They were written before `reject-change-markup-inside-a-unit` archived and moved the contract to
`0.3.0`. All three now say `0.3.0`.

Design decision 2 ended by leaving `state-a-modified-requirement-whole`'s decision 4 to that change's
review, marked "Confirm at review". That review settled it on 2026-10-02 and kept the sentence that a
stated scenario carries its accepted counterpart's identifier, because the `sync` instructions it wrote
match each stated scenario to its accepted counterpart by identifier. The paragraph now says so. This
change's comparison still compares scenarios by text, and nothing in it moves.

`openspec validate see-a-modification-the-merge-did-not-take --strict` reports the change valid.

## 1.1 Every result reproduces

`run-all.sh` (appendix D of `the-measurement-2026-09-30.md`) was run again from an empty directory with
the rebuilt binary. Its log, `run-all-2026-10-02.log`, has 523 lines, as the first run's did, and
`diff` against the first run's log differs in one line only, the version the binary prints:

```
1c1
< htmlspec 0.18.0 (6a591bdf9d92-dirty)
---
> htmlspec 0.20.0 (715480415d8c-dirty)
```

So every number in the first measurement holds at HEAD, each against the task's list:

| Expected | At HEAD |
| --- | --- |
| `req-x` reached at every step | reached at steps 1, 2, 3a, 3b, 4, 5, and 6 |
| `req-y` reached in step 3a | reached |
| `req-y` absent in steps 3b, 4, and 5 | absent in all three |
| step 6: "applied in full" and "ready" | `the change's contract has been applied in full`, `ready: archival would archive this change`, `"needsSync": false` |
| step 5: "This change has not been synced", `needsSync: true` | `ready with questions: this change has not been synced`, `"needsSync": true`, `"unmergedRequirements": ["req-y"]` |
| step 4: `stated` holds `req-x`, `req-y`, `req-z` | the same |

Every `sync` and `archive --check` exited 0. The step 3b archival under a pseudo-terminal answered with
end-of-input exited 3 and left the project unchanged, and archival without a terminal or `--yes` exited
2 and left it unchanged. Both documents validate at every step, with only
`review/unverified-requirement` warnings.

**Both defects stand at HEAD.** `state-a-modified-requirement-whole` changed instructions and the
contract's text, and no code, so this was the expected result.

## 1.2 The predecessor, and what the instructions say now

`state-a-modified-requirement-whole` is archived as
`openspec/changes/archive/2026-10-02-state-a-modified-requirement-whole`, and
`openspec/specs/artifact-kinds/spec.md` holds "A change's specification states a modified requirement
whole" once.

### Every sentence that names a set `sync` reports, or says what happens to a withdrawal

Read in `crates/htmlspec-assets/instructions/` at `7154804`.

**`sync.md`:**

| Lines | What it says | What this change does to it |
| --- | --- | --- |
| 74-86 | "Four sets come back", the table of `stated`, `reached`, `absent`, `untakenRenames`, and "`absent` and `untakenRenames` are what you work through. Every stated identifier is in exactly one of `reached`, `absent`, and `untakenRenames`" | task 4.1: the table and the partition |
| 94 | "For each identifier in `absent`:" | unchanged. A requirement marked `removed` no longer reaches `absent` |
| 103-111 | step 3: an `added` requirement that restates an accepted one is not folded in, because "`absent` would never empty, and the only report left for it would call a merged obligation withdrawn" | its word "withdrawn" becomes "abandoned", with the absent note |
| 130-135 | "`htmlspec sync` lists it under `reached` whether or not you have merged its text. The comparison reads identifiers, and headings for a rename, and never a modified requirement's words. **So `reached` does not tell you a modification landed.**" | **false after this change.** Task 4.1 rewrites the section to work through `untakenContent` |
| 154-167 | steps 4 and 5: hold the whole requirement for a scenario marked `unchanged` whose text differs, and for an accepted scenario left out | kept, as design decision 7 says |
| 175 | "For each entry in `untakenRenames`:" | unchanged |
| 195-201 | a renamed requirement in `reached`, and one in `absent` | unchanged |
| 204-212 | "A requirement you should not merge": "An identifier in `absent` may be there because the requirement was withdrawn", "Leave it absent", "say why you concluded it was withdrawn" | task 4.1: a requirement marked `removed` never appears in `absent`, and what remains there was abandoned |
| 221-224 | "`absent` should be empty, or hold only the identifiers you reported, as withdrawn or as a modification the change stated as an addition, and `untakenRenames` should be empty. A modified requirement is in `reached` either way" | task 4.1. **The last sentence is false after this change** |
| 233-246 | "What to report": the added requirement left unmerged, renames, modified requirements merged or held, scenarios left out, "Which identifiers you left absent, and why each was withdrawn deliberately" | task 4.1: all four worklists, and "abandoned" |
| closing paragraph | "archival makes this same comparison again — it should find nothing. **Do not archive a change while one of its modified requirements is held.** Archival's comparison reads identifiers, so it cannot see a held requirement" | the stop stays. **The reason is false after this change**, and is rewritten |

**`archive.md`:**

| Lines | What it says | What this change does to it |
| --- | --- | --- |
| 274-278 | step 4: requirements no accepted specification holds. "After step 5, each remaining one should be a requirement the change deliberately withdrew" | task 4.2: corrected |
| 279-283 | step 4: renames not taken, "`untakenRenames` in the payload names each one with both headings, and `needsSync` is true while any remain" | task 4.2: the two new findings go beside it |
| 314-319 | step 5's bullet on a modified requirement, and the two cases that hold it | kept |
| 326-328 | step 5: "A requirement the change withdrew is removed. A withdrawal is the one case where deleting is correct" | task 4.2: archival then counts it as reached |
| 340-346 | step 5's stop: "`htmlspec archive` compares identifiers, so it counts that requirement as reached and archives without asking" | the stop stays. **The reason is false after this change**: archival will report the requirement's content as not taken and ask. It archives over that only with `--yes`, or a yes at the prompt |
| 352-356 | step 6: "Archival cannot read a modified requirement's text, though: it counts one as reached whenever the accepted specifications hold its identifier" | **false after this change**, and rewritten |
| 381 | step 6: "If it still reports open tasks, unmerged requirements, untaken renames, or a predecessor that is not archived, it asks" | task 4.2: gains the two sets |

### Corrections to what the tasks and the first measurement cite

- Section 8 of `the-measurement-2026-09-30.md` gave line numbers from before
  `state-a-modified-requirement-whole`. They are now: `archive.md` 326 (was 280) and 276-278 (was
  233-235). `sync.md` 74-86 is unchanged, and 153-154 is now 206-207, and 168 is now 221.
- Tasks 4.1 and 4.2 cite sections by name, not lines, and every section they name exists. Two passages
  are not named in either task, and both carry a reason this change makes false: `sync.md`'s closing
  stop and `archive.md`'s step 5 stop and step 6 paragraph. Both were added by
  `state-a-modified-requirement-whole`'s review. They are corrected under tasks 4.1 and 4.2, beside the
  sections those tasks name, and the test that pins them,
  `no_workflow_archives_a_change_while_a_modified_requirement_is_held`, pins the stops and not their
  reasons, so it holds through the edit.

## 1.3 No other active change modifies these requirements

`grep -l -F "### Requirement: <heading>"` over every active change's
`specs/toolchain-packaging/spec.md`, for each of the four headings this change modifies:

| Requirement | Active changes that state it |
| --- | --- |
| Archival refuses on the contract and asks about the work | this one only |
| Archival assesses the merge and never performs it | this one only |
| The command surface reports whether a change reached the accepted contract | this one only |
| The workflow instructions carry a stage for applying a change's contract | this one only |

The other three active changes with a toolchain-packaging delta are `record-a-task-done-differently`,
`resolve-a-reference-across-its-change`, and `show-what-a-modification-changed`, and none states any of
the four. `git log 6240365..HEAD -- openspec/specs/toolchain-packaging/spec.md` is empty, so the
accepted text this change's MODIFIED blocks restate has not moved since they were written. **No
re-base task is needed.**
