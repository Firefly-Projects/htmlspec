# The release runbook's run for 0.22.0

**Date:** 2026-10-03
**Tasks:** 6.1 and 6.2, against `runbooks/release/tasks.html` at runbook version `0.1.0`

**It worked as written, in its order.**

## The increment (task 6.2)

**`0.21.0` → `0.22.0`, a minor increment, confirmed by the contributor before the version moved.**
`validate` now accepts a `data-refs` entry that reaches a change's specification or the accepted
specifications, which it rejected before, and its report of an unresolved reference names the places
it searched. `engine::run` gains a scope argument, which every library caller must now pass. No document
that validated before fails, so nothing breaks for an author. The alternative was a patch, `0.21.1`,
treating this as a fix, since the contract already promised resolution beyond the document. It was not
taken, because the command's behavior and a public signature both change. The document contract stays
`0.3.0`.

## The four steps

1. **Advance the version and relock offline.** `version` under `[workspace.package]` moved to
   `0.22.0`, and `cargo update --workspace --offline` moved exactly the seven workspace members.
2. **Regenerate the instruction files.** Run as `./target/debug/htmlspec init --tools
   claude,codex,pi` from a binary rebuilt at `0.22.0`. All 45 installed files changed their
   `generatedBy` stamp. The five `new` files had already taken this change's sentence at task 4.2.
   `init` created the stray `htmlspec/` and a `spec.nix` at `0.0.0`, and both were removed.
   `scripts/check-installed` then reported 45 files matching.
3. **Write the changelog entry.** `## 0.22.0 — a task can reference the requirement it serves`.
4. **Confirm `scripts/check-release` passes.** `ok 27 archived change(s) record a version, and 22
   version(s) are in the changelog` and `ok declared version 0.22.0 has a changelog entry`. The
   archive's `VERSION` and its `runbooks.json` are written at archival (task 6.3).

`scripts/check` then passed every stage at `0.22.0`: nine in `nix develop`, and the four Python stages
outside it, for the reason `what-the-implementation-revealed-2026-10-03.md` records.

## The gap recorded before is still open

Neither the runbook's fourth step nor the archive workflow says that archival in this repository also
needs a hand-written `runbooks.json` beside `VERSION`. Task 6.3 writes both, as the last seven archives
did.
