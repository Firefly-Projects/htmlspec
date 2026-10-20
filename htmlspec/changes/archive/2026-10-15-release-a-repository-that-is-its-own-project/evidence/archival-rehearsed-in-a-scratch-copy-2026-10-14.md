# What archival makes of the revised runbook, rehearsed in a scratch copy

**Date:** 2026-10-14
**Trees:** the predecessor's, `govern-this-repository-from-htmlspec` at `3b45ccb`, exported with
`git archive`, and this change's runbook and change directory at `f74626f`, laid over it after the
predecessor was archived. The binary is `htmlspec 0.30.1`, built from this branch. The program does
not change in either change, so its archival code is the code that will archive both.

Design decision 5 says this change publishes `release` at 0.2.0 under 0.30.2, provided the predecessor
is archived first at 0.30.1. Design decision 3 and the new scenario say archival writes `VERSION` from
`spec.nix`, and that a `spec.nix` raised to a version `Cargo.toml` does not declare fails the release
check unless a heading names it. This rehearses all of that in scratch copies. Nothing here touched the
repository.

## Method

Each change got a placeholder `specification.html` stating no requirement, because archival needs one
and the real one is written at archival.

1. **R1.** In the predecessor's tree, set `spec.nix` to 0.30.1 and run
   `htmlspec archive govern-this-repository-from-htmlspec --yes`, then `scripts/check-release`.
2. **R2.** Copy R1's result, lay this change's runbook and change directory over it, set the workspace
   version to 0.30.2, add a `## 0.30.2` heading, set `spec.nix` to 0.30.2, and run
   `htmlspec archive release-a-repository-that-is-its-own-project --check`, then `--yes`, then
   `scripts/check-release`.
3. **R3.** As R2, with the runbook's `VERSION` put back to 0.1.0 and its documents revised.
4. **R4.** As R2, with `spec.nix` raised to 0.31.0 while `Cargo.toml` declares 0.30.2, then the same
   with a `## 0.31.0` heading added.

## Result

**R1.** The predecessor publishes the runbook as it was:

```
version 0.30.1 follows 0.30.0
archived to …/htmlspec/changes/archive/2026-10-14-govern-this-repository-from-htmlspec at version 0.30.1
  published 1 runbook release(s) in runbooks.json
    release 0.1.0 sha256:3d7efaec14b9… sha256:e6a4410e6261…
   VERSION: 0.30.1
  ok  39 archived change(s) record a version, and 33 version(s) are in the changelog
  ok  declared version 0.30.1 has a changelog entry
```

**R2.** This change publishes 0.2.0, with the fingerprints of the revised documents:

```
version 0.30.2 follows 0.30.1
staging 1 runbook release(s)
ready with questions: archival would report the findings above and ask
archived to …/2026-10-14-release-a-repository-that-is-its-own-project at version 0.30.2
  published 1 runbook release(s) in runbooks.json
    release 0.2.0 sha256:d1d43b45080d… sha256:0120231b137b…
   VERSION: 0.30.2
  ok  40 archived change(s) record a version, and 34 version(s) are in the changelog
  ok  declared version 0.30.2 has a changelog entry
```

`sha256:d1d43b45080d…` and `sha256:0120231b137b…` are `specification.html` and `tasks.html` as
committed at `f74626f`. The `VERSION` archival wrote is the version `spec.nix` declared.

**R3.** With the version left at 0.1.0, archival refuses, before and at the real archival:

```
htmlspec: archival does not proceed while a runbook's release does not follow the one before it. …
the runbook release declares 0.1.0, which 2026-10-14-govern-this-repository-from-htmlspec already published with different canonical content. …
exit 1
```

Nothing was archived. The refusal names the predecessor's archive, which is the snapshot the comparison
found.

**R4.** Archival accepts a `spec.nix` the workspace does not agree with, and records it. The release
check is what fails:

```
archived to …/2026-10-14-release-a-repository-that-is-its-own-project at version 0.31.0
   VERSION: 0.31.0
!!! no CHANGELOG.md heading names version 0.31.0. A shipped version records what changed in it
1 fault(s) across 40 archived change(s), declared version 0.30.2
exit 1
```

With a `## 0.31.0` heading added as well, it passes:

```
  ok  40 archived change(s) record a version, and 35 version(s) are in the changelog
  ok  declared version 0.30.2 has a changelog entry
```

So the release check catches a mismatch only while no heading names the stray version, which is what
the design's risk "Nothing compares `spec.nix` with `Cargo.toml`" says.

## What this does not show

The real archivals run after review, against whatever the two changes then hold. A finding that edits
this runbook's documents changes their fingerprints, and one that edits the predecessor's runbook would
make R1 wrong. Archived the other way round, or in one tree, the predecessor would publish this change's
runbook under 0.30.1. That order is the design's first risk, and this rehearsal shows only the right
one.
