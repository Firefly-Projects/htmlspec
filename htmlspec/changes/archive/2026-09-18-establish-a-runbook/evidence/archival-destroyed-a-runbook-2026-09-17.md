# `htmlspec archive <runbook>` destroyed the runbook

**A data loss this change introduced and its own contract forbade.** Found in review of PR #29,
not by any check. Reproduced end to end before fixing, so the fix is against a measurement.

## The reproduction, on the code at `88759b6`

A scratch project, the shipped release runbook copied into it, the project version advanced:

```
$ htmlspec list --runbooks
release  specification, tasks

$ htmlspec archive htmlspec/runbooks/release --yes
note: this change has no proposal.html and no design.html …
note: 4 task(s) are not done or dropped: task-advance-the-version, …
note: 5 requirement(s) this change states appear in no accepted specification: …
archived to /private/tmp/destroy/htmlspec/changes/archive/2026-09-17-release at version 0.1.0

$ htmlspec list --runbooks
no active runbooks

$ ls htmlspec/runbooks/
(empty)
```

The runbook is gone. Its two documents are in `changes/archive/2026-09-17-release/`, with a
`VERSION` file archival wrote beside them.

## What was wrong

`resolve_change` returned `Ok(target)` for **any** existing directory. A runbook directory is an
existing directory, so it resolved as a change and archival moved it.

**The contract already forbade this and nothing checked it.** `runbook-authority`'s *A project
keeps its procedures in a fourth place* states that a procedure run repeatedly is not a change and
does not archive, "so a representation that archives it after its first run destroys the thing
being kept." I wrote that sentence and then shipped the representation it describes. This is
exactly the shape `project-bootstrap` names — a rule nothing checks is not a rule — which is also
the shape this whole change exists to close for the release procedure.

## Why my own testing missed it, and the count was wrong twice

**Four unrelated gates refuse before archival moves anything:**

1. the version-ordering gate,
2. the absent-`specification.html` gate,
3. the example-**identifier** gate,
4. the placeholder-**summary** gate.

Gates 3 and 4 share `review/example-content` but fire on different content, which is why the count
looks like three from the rule list.

A half-built runbook bounces off all four, so every cheap probe looks safe. Only a *complete*
runbook — two documents, real prose, real identifiers, version advanced — reaches the move. That is
precisely the runbook a project has once this change ships.

**The count was three until the regression test was run against the pre-fix code.** Its first
failure read *"the refusal does not name what the directory is"* followed by the placeholder-summary
warning: the test was passing the pre-fix code for an unrelated reason. A test that cannot fail for
the reason it was written is worse than no test, so gate 4 is now cleared explicitly.

`the-status-gap-2026-09-17.md` walked a runbook path through this same resolver, saw `status`
report on it, and stopped at "the report's shape is wrong for a runbook". `archive` shares the
resolver and moves files. **The measurement was one operation short of the data loss.**

## `sync` and `status` needed the same guard, for different reasons

Both share the resolver. Neither destroys anything, and both are guarded anyway:

- **`sync` is the one that matters.** Measured before the fix: `sync --change
  htmlspec/runbooks/release` listed all five of the release runbook's requirements as `absent` and
  printed *"Merge each into the specification that should hold it."* `runbook-authority` states
  that a requirement in a runbook's specification is what that runbook requires of a run rather
  than an obligation of the project. **So the report directs an agent to merge into the accepted
  specifications a set of requirements the contract says do not belong there** — the same failure
  `contract-authority` records about the comparison operation and example content: an obligation
  nobody wrote entering the contract by the hand of an agent doing as it was told. It writes
  nothing, and the harm is in what it tells an agent to write.
- **`status` is merely wrong.** It reports a runbook against a change's four-document shape:
  `absent proposal.html`, `absent specification.html` for documents a runbook never has.

**The refusal message differs per operation**, via a `Refusal` enum. My first fix used one shared
sentence — "so it is not archived" — which is the reason for `archive` and **false** for the other
two. A refusal whose stated reason a contributor can check and find wrong makes the refusal itself
look unreliable, which is the finding the previous change recorded at its task 5.1.

## A runtime check rather than a type, and why

**A runtime check in one shared function, `refuse_a_runbook`, called on all three resolution
routes.** Not a distinct `ChangeDirectory` newtype, and the reason is proportionality rather than
taste: `ProjectRoot::change` already composes change paths in 23 places across `commands.rs` and
`sequence.rs`, so a newtype whose construction proved change-ness would have to thread through all
of them, and the invariant it would carry — "this path is not inside `runbooks/`" — is a fact about
the filesystem at a moment rather than a property of a value. `AGENTS.md` §8 is the governing
clause: the objective is the strongest practical model that eliminates the real failure mode, and
here that is one guard at the single boundary all three operations already pass through.

What the guard *is* placed to do is what §2 asks: it is not a rule a caller must remember. There is
no route from an argument to an archival that does not pass through `resolve_change` or
`resolve_change_in`, and both call it.

## The one that bit me twice: `..` is not normalized

The first guard compared `directory.components().collect::<PathBuf>()` against
`project.runbooks()`. **`Path::components` drops `.` and keeps `..`**, so
`htmlspec/changes/../runbooks/release` compared against `htmlspec/runbooks` found no prefix, and
**the traversal route destroyed the runbook with the guard in place.** Measured, not theorized —
route 3 of four printed `archived to …` on the first fixed build.

That route is reachable in practice and is the `--store` arm's *only* route:
`project.change("../runbooks/release")` composes exactly that path. Verified before the fix:

```
$ htmlspec --store d3 status --change ../runbooks/release
  present tasks.html
  present specification.html
  0 of 4 task(s) done or dropped
```

Fixed with `canonicalize`, which resolves `..` and is available because every caller has already
tested `is_dir`. `a_runbook_reached_by_traversal_is_refused` fails against the `components`
version, so the mistake cannot come back.

## Verification after the fix

All four archive routes refused, runbook intact, archive empty:

| Route | Before | After |
| --- | --- | --- |
| `archive htmlspec/runbooks/release` | archived | refused |
| `archive /abs/path/htmlspec/runbooks/release` | archived | refused |
| `archive ../runbooks/release` | archived | refused |
| `archive ./htmlspec/./runbooks/release` | archived | refused |
| `sync --change <runbook>` | directed a merge | refused |
| `status --change <runbook>` | reported wrongly | refused |
| `--store <id> status --change ../runbooks/release` | reported wrongly | refused |
| `status --change a-real-change` | worked | works |

Five tests, four new and all verified to fail against the pre-fix code:

- `archival_refuses_a_runbook_and_leaves_it_on_disk` (`tests/archive.rs`) — drives the **real
  binary** in a child process and asserts both documents are still on disk afterwards, because the
  thing being prevented is a filesystem move and exit status alone does not distinguish a refusal
  from a near miss. Fails pre-fix with `archival accepted a runbook`.
- `a_runbook_is_refused_where_a_change_is_expected` — the resolver, all three `Refusal` subjects,
  asserting the refusal names the runbook and **not** any of the four gates.
- `a_runbook_reached_by_traversal_is_refused` — the `..` case, which fails against the
  `components` guard as well as against no guard.
- `each_operation_refuses_a_runbook_for_its_own_reason` — that `sync` and `status` do not claim
  "not archived".
- `a_change_beside_a_runbook_still_resolves` — the guard refuses runbooks rather than directories.
