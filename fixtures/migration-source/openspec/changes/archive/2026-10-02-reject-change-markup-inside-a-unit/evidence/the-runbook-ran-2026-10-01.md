# The release runbook's run for 0.19.0

**Date:** 2026-10-01
**Tasks:** 6.1 and 6.2, against `runbooks/release/tasks.html` at runbook version `0.1.0`

**It worked as written, in its order.**

## The increment (task 6.2)

**`0.18.0` → `0.19.0`, a minor increment, confirmed by the contributor before the version moved.**
`validate` reports a new error, `schema/change-markup`, so it rejects documents it accepted before,
and the document contract moves from `0.2.0` to `0.3.0`. That is a change to what the product
accepts, which is what 0.17.0 and 0.18.0 were minor increments for. A patch was the alternative. The
one patch in the history, 0.6.1, changed only the repository's own process. A major increment would
commit the vocabulary, and that is `1.0.0`'s decision, not this change's.

## The four steps

1. **Advance the version and relock offline.** `version` under `[workspace.package]` moved to
   `0.19.0`, and `cargo update --workspace --offline` moved exactly the seven workspace members.
2. **Regenerate the instruction files.** Run as `./target/debug/htmlspec init --tools
   claude,codex,pi` from a binary rebuilt at `0.19.0`. No workflow body changed in this change, so
   the diff is the `generatedBy` stamp alone, one line in each of the 45 files.
   `scripts/check-installed` then reported 45 files matching. `init` created the stray `htmlspec/`
   and `spec.nix` the runbook warns about, and both were removed.
3. **Write the changelog entry.** `## 0.19.0 — change markup in a unit is an error`.
4. **Confirm `scripts/check-release` passes.** `ok 24 archived change(s) record a version, and 19
   version(s) are in the changelog` and `ok declared version 0.19.0 has a changelog entry`. The
   archive's `VERSION` and its `runbooks.json` are written at archival (task 6.3).

`scripts/check` then passed every stage: nine in `nix develop`, and the four Python stages outside
it, for the reason `what-the-implementation-revealed-2026-10-01.md` records.

## The gap recorded before is still open

Neither the runbook's fourth step nor the archive workflow says that archival in this repository
also needs a hand-written `runbooks.json` beside `VERSION`. Task 6.3 writes both, as the last three
archives did.
