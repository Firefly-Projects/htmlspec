# What archival makes of the moved runbook, rehearsed in a scratch copy

**Date:** 2026-10-14
**Tree:** this change's working tree after task 3.1 (the runbook moved and its two stylesheet
revisions copied), with `htmlspec 0.30.0 (4fa4e96992f8)` built from `4fa4e96`. The program does not
change in this change, so the archival code is the code that will archive it.

Design decision 7 says the two empty inventories, at 0.29.1 and 0.30.0, do not end the release
runbook's history, and that archiving this change publishes `release` at 0.1.0 against 0.29.0's
snapshot. This rehearses that archival in a copy, so the claim is checked before the real one. Nothing
here touched the repository: the copy is in a scratch directory and was archived there.

## Method

1. Copy `htmlspec/` and `spec.nix` to a scratch directory, and set `version = "0.30.1"` in the copy's
   `spec.nix`.
2. Give the copied change a placeholder `specification.html` stating no requirement, because archival
   refuses a template's example content and the real specification is written at archival.
3. `htmlspec archive govern-this-repository-from-htmlspec --check`, then the same with `--yes`.
4. Copy `Cargo.toml` and `CHANGELOG.md` into the scratch directory, set the copy's workspace version to
   0.30.1, add a `## 0.30.1` heading, and run this change's `scripts/check-release --root .` there.

## Result

The readiness check, before the specification existed, and with it:

```
version 0.30.1 follows 0.30.0
staging 1 runbook release(s)
…
note: 9 task(s) are not done or dropped: task-3-1, task-3-2, …
ready with questions: archival would report the findings above and ask
exit 0
```

The archival:

```
archived to …/htmlspec/changes/archive/2026-10-14-govern-this-repository-from-htmlspec at version 0.30.1
  re-expressed 35 reference(s) in 4 document(s), so each reaches from the archive what it reached from the change
  published 1 runbook release(s) in runbooks.json
```

The inventory it wrote:

```
htmlspec-runbook-inventory/v1
release 0.1.0 sha256:3d7efaec14b97d656d65423822cea50e770a172e01956e4985a9ef9ec29f177c sha256:e6a4410e6261b9f77a5c90ec3e60fcd56ff25726207abf23e586c7398dffc9a9
```

These are the version and fingerprints 0.29.0's inventory records. The comparison found the 0.29.0
snapshot across the two empty inventories, judged the runbook unchanged, and let it keep 0.1.0.

The release check on the archived copy:

```
  ok  39 archived change(s) record a version, and 33 version(s) are in the changelog
  ok  declared version 0.30.1 has a changelog entry
exit 0
```

It read the `release` lineage from 0.15.0 to 0.30.1 as one release recorded again, with the two empty
inventories between, and found no missing record.

## What this does not show

The real archival runs after the code review, against whatever the change then holds. A finding that
edits the runbook's canonical documents would make this rehearsal's result wrong, and archival would
then refuse 0.1.0 for changed content, which is the check doing its job. Task 5.2 records the real
result.
