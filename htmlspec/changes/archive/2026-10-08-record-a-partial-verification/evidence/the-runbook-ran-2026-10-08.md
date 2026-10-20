# The release runbook's run for 0.26.0

**Date:** 2026-10-08
**Tasks:** 8.1 and 8.2, against `runbooks/release/tasks.html` at runbook version `0.1.0`

**It worked as written, in its order.** This change was built in parallel with the three changes
stacked with it, and released after the stack was assembled, bottom first, so each layer's release
follows the one below it.

## The increment (task 8.2)

**`0.25.0` → `0.26.0`, a minor increment, confirmed by the contributor on 2026-10-08 before
the version moved**, together with the other three releases of the stack. A verification outcome, a field, and a validation rule are additions, and every document that validated still validates. The document contract moves from `0.5.0` to `0.6.0` for the `partial` outcome and the `unchecked` field.

## The four steps

1. **Advance the version and relock offline.** `version` under `[workspace.package]` moved to
   `0.26.0`, and `cargo update --workspace --offline` moved exactly the seven workspace members.
2. **Regenerate the instruction files.** Run as `./target/debug/htmlspec init --tools
   claude,codex,pi` from a binary rebuilt at `0.26.0`. All 45 installed files took the new
   `generatedBy` stamp. `init` created the stray `htmlspec/` and a `spec.nix`, and both were
   removed. `scripts/check-installed` then reported 45 files matching.
3. **Write the changelog entry**, with the version in its heading.
4. **Confirm `scripts/check-release` passes.** `ok 30 archived change(s) record a version, and
   25 version(s) are in the changelog` and `ok declared version 0.26.0 has a changelog
   entry`. The archive's `VERSION` and its `runbooks.json` are written at archival (task 8.3).

`scripts/check` then passed every stage at `0.26.0`, 1314 tests among them: nine stages in
`nix develop`, and the four Python stages outside it.

## The gap recorded before is still open

Neither the runbook's fourth step nor the archive workflow says that archival in this repository also
needs a hand-written `runbooks.json` beside `VERSION`. Task 8.3 writes both.
