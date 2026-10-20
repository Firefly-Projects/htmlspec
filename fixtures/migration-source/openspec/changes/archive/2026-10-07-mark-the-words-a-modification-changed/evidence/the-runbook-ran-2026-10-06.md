# The release runbook's run for 0.23.0

**Date:** 2026-10-06
**Tasks:** 9.1 and 9.2, against `runbooks/release/tasks.html` at runbook version `0.1.0`

**It worked as written, in its order.**

## The increment (task 9.2)

**`0.22.0` → `0.23.0`, a minor increment, confirmed by the contributor before the version moved.**
The document contract moves from `0.3.0` to `0.4.0`, for the `deletions` field and projection rule
8.6. `validate` accepts change markup it refused, in a modified requirement or scenario of a change's
specification, and its refusal everywhere else now says why. The one construct that can newly fail,
a figure whose caption or diagram source is wholly inside a `del`, is carried by no document, and the
`0.x` clause allows it in a minor, as `0.19.0` used it. The alternative was a patch, `0.22.1`. It was
not taken, because the contract version and the projection both move.

## The four steps

1. **Advance the version and relock offline.** `version` under `[workspace.package]` moved to
   `0.23.0`, and `cargo update --workspace --offline` moved exactly the seven workspace members.
2. **Regenerate the instruction files.** Run as `./target/debug/htmlspec init --tools
   claude,codex,pi` from a binary rebuilt at `0.23.0`. All 45 installed files changed their
   `generatedBy` stamp, and the ten `sync` and `archive` copies had already taken this change's text
   at task 7.3. `init` created the stray `htmlspec/` and a `spec.nix` at `0.0.0`, and both were
   removed. `scripts/check-installed` then reported 45 files matching.
3. **Write the changelog entry.** `## 0.23.0 — a modified requirement can mark the words it changed`.
4. **Confirm `scripts/check-release` passes.** `ok 28 archived change(s) record a version, and 23
   version(s) are in the changelog` and `ok declared version 0.23.0 has a changelog entry`. The
   archive's `VERSION` and its `runbooks.json` are written at archival (task 9.3).

`scripts/check` then passed every stage at `0.23.0`, 1,246 tests among them: nine stages in
`nix develop`, and the four Python stages outside it.

## The gap recorded before is still open

Neither the runbook's fourth step nor the archive workflow says that archival in this repository also
needs a hand-written `runbooks.json` beside `VERSION`. Task 9.3 writes both, as the last eight
archives did.
