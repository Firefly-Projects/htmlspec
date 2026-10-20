# The release runbook's run for 0.28.1

**Date:** 2026-10-09
**Tasks:** 6.1 and 6.2, against `runbooks/release/tasks.html` at runbook version `0.1.0`

**It worked as written, in its order.**

## The increment (task 6.2)

**`0.28.0` → `0.28.1`, a patch increment, confirmed by the contributor on 2026-10-09 before the
version moved.** The formatter now does what deterministic-formatting's "Formatting preserves
meaning" already required. The change adds nothing to the document contract, which stays at
`0.7.0`, and it reformats no committed document.

The alternative put to the contributor was a minor increment. It would have treated the change in
which attributes are filled as new behaviour. The contributor chose the patch.

## The four steps

1. **Advance the version and relock offline.** `version` under `[workspace.package]` moved to
   `0.28.1`, and `cargo update --workspace --offline` moved exactly the seven workspace members.
2. **Regenerate the instruction files.** This ran as `./target/debug/htmlspec init --tools
   claude,codex,pi`, from a binary rebuilt at `0.28.1`.
   - All 45 installed files changed in one line each, the `generatedBy` stamp, from `0.28.0` to
     `0.28.1`. No instruction text changed, because no instruction describes the fill.
   - `init` created the stray `htmlspec/` and a `spec.nix`. Neither existed before, and both were
     removed.
   - `scripts/check-installed` reported 45 files matching.
3. **Write the changelog entry**, with the version in its heading. It was written before step 1,
   while the mutation run held the build. The order matters only between steps 1 and 2.
4. **Confirm `scripts/check-release` passes.**
   - `ok 34 archived change(s) record a version, and 29 version(s) are in the changelog`;
   - `ok declared version 0.28.1 has a changelog entry`.

   The archive's `VERSION` and its `runbooks.json` are written at archival (task 6.3).

At `0.28.1`, the four Python stages pass outside `nix develop`:

- `check-prompt`: 4 cases;
- `check-installed`: 45 files;
- `check-release --self-test`: 23 shared cases;
- `check-release`.

The nine stages inside `nix develop` are recorded at task 5.3.

## The gap recorded before is still open

Neither the runbook's fourth step nor the archive workflow says that archival in this repository also
needs a hand-written `runbooks.json` beside `VERSION`. Task 6.3 writes both.
