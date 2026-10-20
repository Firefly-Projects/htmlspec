# The release runbook's run for 0.21.0

**Date:** 2026-10-02
**Tasks:** 6.1 and 6.2, against `runbooks/release/tasks.html` at runbook version `0.1.0`

**It worked as written, in its order.**

## The increment (task 6.2)

**`0.20.0` → `0.21.0`, a minor increment, confirmed by the contributor before the version moved.**
`sync` and archival report findings they did not report before: content and withdrawals the accepted
specifications have not taken. Their structured payloads gain two keys, and two existing sets change
members: `reached` narrows to what arrived, and `absent` and `unmergedRequirements` no longer hold a
requirement the change marks `removed`. So a change archival passed quietly before can now be asked
about, and four accepted requirements in toolchain-packaging change. The alternative was a patch,
`0.20.1`, treating this as a fix to a comparison that gave wrong answers. It was not taken, because an
agent reading either payload sees a different shape. The document contract stays `0.3.0`.

## The four steps

1. **Advance the version and relock offline.** `version` under `[workspace.package]` moved to
   `0.21.0`, and `cargo update --workspace --offline` moved exactly the seven workspace members.
2. **Regenerate the instruction files.** Run as `./target/debug/htmlspec init --tools
   claude,codex,pi` from a binary rebuilt at `0.21.0`. All 45 installed files changed their
   `generatedBy` stamp. The ten `sync` and `archive` files had already taken this change's bodies at
   task 4.4. `init` created the stray `htmlspec/` and a `spec.nix` at `0.0.0`, and both were removed.
   `scripts/check-installed` then reported 45 files matching.
3. **Write the changelog entry.** `## 0.21.0 — sync sees a modification nobody merged`.
4. **Confirm `scripts/check-release` passes.** `ok 26 archived change(s) record a version, and 21
   version(s) are in the changelog` and `ok declared version 0.21.0 has a changelog entry`. The
   archive's `VERSION` and its `runbooks.json` are written at archival (task 6.3).

`scripts/check` then passed every stage at `0.21.0`: nine in `nix develop`, and the four Python stages
outside it, for the reason `what-the-implementation-revealed-2026-10-02.md` records.

## The gap recorded before is still open

Neither the runbook's fourth step nor the archive workflow says that archival in this repository also
needs a hand-written `runbooks.json` beside `VERSION`. Task 6.3 writes both, as the last six archives
did.
