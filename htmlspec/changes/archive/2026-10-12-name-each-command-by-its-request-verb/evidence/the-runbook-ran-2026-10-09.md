# The release runbook's run for 0.29.0

**Date:** 2026-10-09
**Tasks:** 6.1 and 6.2, against `runbooks/release/tasks.html` at runbook version `0.1.0`

**It worked as written, in its order.** The change was implemented on a branch from `219d1b6`, the
first commit of `fill-only-the-references-a-document-loads` (PR #53, `0.28.1`). It was rebased onto
that PR's head, `c68cbc9`, before the release. The rebase was clean, because the two changes share
no file. So this release follows `0.28.1`.

## The increment (task 6.2)

**`0.28.1` → `0.29.0`, a minor increment, confirmed by the contributor on 2026-10-09.** They agreed
to it ("okay commit and push") in reply to the question that asked them to confirm `0.29.0`.

- The command line gains a spelling for every operation and loses none.
- Every OpenSpec spelling behaves byte for byte as it did before. Task 5.1's measurement shows this:
  42 invocations, with the same exit status and the same tree difference under both spellings.
- The document contract stays at `0.7.0`.

An added interface that breaks nothing is a minor increment.

## The four steps

1. **Advance the version and relock offline.** `version` under `[workspace.package]` moved to
   `0.29.0`. `cargo update --workspace --offline` moved exactly the seven workspace members.
2. **Regenerate the instruction files.** This ran as `./target/debug/htmlspec init --tools
   claude,codex,pi`, from a binary rebuilt at `0.29.0`.
   - All 45 installed files changed in one line each, the `generatedBy` stamp, from `0.28.1` to
     `0.29.0`. No instruction text changed: the instructions keep naming the OpenSpec spellings, and
     the test from task 2.5 holds them to it.
   - `init` created the stray `htmlspec/` and a `spec.nix`. Neither existed before, and both were
     removed.
   - `scripts/check-installed` reported 45 files matching.
3. **Write the changelog entry**, with the version in its heading.
4. **Confirm `scripts/check-release` passes.**
   - `ok 35 archived change(s) record a version, and 30 version(s) are in the changelog`;
   - `ok declared version 0.29.0 has a changelog entry`.

   The archive's `VERSION` and its `runbooks.json` are written at archival (task 6.3).

## The gap recorded before is still open

Neither the runbook's fourth step nor the archive workflow says that archival in this repository also
needs a hand-written `runbooks.json` beside `VERSION`. Task 6.3 writes both.
