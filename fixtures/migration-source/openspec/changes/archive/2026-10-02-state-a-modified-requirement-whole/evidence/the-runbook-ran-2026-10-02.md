# The release runbook's run for 0.20.0

**Date:** 2026-10-02
**Tasks:** 5.1 and 5.2, against `runbooks/release/tasks.html` at runbook version `0.1.0`

**It worked as written, in its order.**

## The increment (task 5.2)

**`0.19.0` → `0.20.0`, a minor increment, confirmed by the contributor before the version moved.**
artifact-kinds gains a requirement, "A change's specification states a modified requirement whole",
and "Every synthesis and every contract update is agent-driven" changes what the merge does with a
modified requirement. A change's specification now carries an obligation it did not carry before. The
alternative was a patch, `0.19.1`, because no command, rule, unit, or field changes and the document
contract stays `0.3.0`. The one patch in the history, 0.6.1, changed only the repository's own
process. This changes the product's contract and the instructions it installs, so it is not that.

## The four steps

1. **Advance the version and relock offline.** `version` under `[workspace.package]` moved to
   `0.20.0`, and `cargo update --workspace --offline` moved exactly the seven workspace members.
2. **Regenerate the instruction files.** Run as `./target/debug/htmlspec init --tools
   claude,codex,pi` from a binary rebuilt at `0.20.0`. All 45 installed files changed their
   `generatedBy` stamp. The ten `sync` and `archive` files had already taken this change's new
   bodies at task 3.7, so the bump changed only their stamp. `scripts/check-installed` then reported
   45 files matching. `init` created the stray `htmlspec/` and a `spec.nix` at `0.0.0`, as the runbook
   warns, and both were removed.
3. **Write the changelog entry.** `## 0.20.0 — a modified requirement is stated whole`.
4. **Confirm `scripts/check-release` passes.** `ok 25 archived change(s) record a version, and 20
   version(s) are in the changelog` and `ok declared version 0.20.0 has a changelog entry`. The
   archive's `VERSION` and its `runbooks.json` are written at archival (task 5.3).

`scripts/check` then passed every stage at `0.20.0`: nine in `nix develop`, and the four Python stages
outside it, for the reason `what-the-implementation-revealed-2026-10-02.md` records.

## The gap recorded before is still open

Neither the runbook's fourth step nor the archive workflow says that archival in this repository also
needs a hand-written `runbooks.json` beside `VERSION`. Task 5.3 writes both, as the last five archives
did.
