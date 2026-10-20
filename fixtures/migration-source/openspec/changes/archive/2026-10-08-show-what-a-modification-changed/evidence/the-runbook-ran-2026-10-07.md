# The release runbook's run for 0.24.0

**Date:** 2026-10-07
**Tasks:** 8.1 and 8.2, against `runbooks/release/tasks.html` at runbook version `0.1.0`

**It worked as written, in its order.**

## The increment (task 8.2)

**`0.23.0` → `0.24.0`, a minor increment, confirmed by the contributor before the version moved.**
The command surface gains an operation, `show --diff`, and toolchain-packaging gains one accepted
requirement. The document contract stays at `0.4.0`: no unit, field, attribute, or projection rule is
added, and the marks reader adds nothing to the model. `sync`, archival, and their payloads give the
same results, which task 3.1 checked byte for byte. Under `0.x` a feature moves the minor, as `sync`
did at `0.7.0`. The alternative was a patch, `0.23.1`, and it was not taken, because it would
understate a new command.

## The four steps

1. **Advance the version and relock offline.** `version` under `[workspace.package]` moved to
   `0.24.0`, and `cargo update --workspace --offline` moved exactly the seven workspace members.
2. **Regenerate the instruction files.** Run as `./target/debug/htmlspec init --tools
   claude,codex,pi` from a binary rebuilt at `0.24.0`. All 45 installed files changed their
   `generatedBy` stamp, and the ten `verify` and `archive` copies had already taken this change's
   text at task 6.3. `init` created the stray `htmlspec/` and a `spec.nix`, and both were removed.
   `scripts/check-installed` then reported 45 files matching.
3. **Write the changelog entry.** `## 0.24.0 — see what a modification changed`. The README's list of
   commands gains `htmlspec show add-dark-mode --diff`.
4. **Confirm `scripts/check-release` passes.** `ok 29 archived change(s) record a version, and 24
   version(s) are in the changelog` and `ok declared version 0.24.0 has a changelog entry`. The
   archive's `VERSION` and its `runbooks.json` are written at archival (task 8.3).

`scripts/check` then passed every stage at `0.24.0`, 1,286 tests among them: nine stages in
`nix develop`, and the four Python stages outside it.

## The gap recorded before is still open

Neither the runbook's fourth step nor the archive workflow says that archival in this repository also
needs a hand-written `runbooks.json` beside `VERSION`. Task 8.3 writes both, as the last nine
archives did.
