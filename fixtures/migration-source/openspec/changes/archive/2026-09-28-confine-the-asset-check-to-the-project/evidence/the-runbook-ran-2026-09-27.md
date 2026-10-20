# The release runbook's run for 0.17.0

Tasks 6.1 and 6.2, against `runbooks/release/tasks.html` at runbook version `0.1.0`. **It worked as
written, with the known binary defect avoided rather than fixed, and one hazard now smaller.**

## The increment (task 6.2)

**`0.16.0` → `0.17.0`, a minor increment, confirmed by the contributor before the version moved.**
`htmlspec assets` changes what it reports: a repository holding a project beside fixtures or a
toolchain copy now passes. The payload gains two keys, and the report gains a line. Nothing that
passed before fails now. A patch was the alternative. The one patch in the history, 0.6.1, changed
only the repository's own process, and this changes the product.

## The four steps

1. **Advance the version and relock offline.** `cargo update --workspace --offline` moved exactly
   the seven workspace members to 0.17.0.
2. **Regenerate the instruction files.** Run as `./target/debug/htmlspec init --tools claude,codex,pi`,
   which reported `htmlspec 0.17.0`. The bare `htmlspec` on the `PATH` is still the profile's
   0.12.0, the defect the 0.16.0 run recorded. No workflow body changed in this change, so the
   diff is the version stamp alone, one line in each of the 45 files, which is the hazard
   `req-version-before-regenerate` describes in its purest form. `scripts/check-installed` then
   reported 45 files matching.
3. **Write the changelog entry.** `## 0.17.0 — a project's stores are the ones inside it`.
4. **Confirm `scripts/check-release` passes.** `ok 22 archived change(s) record a version, and 17
   version(s) are in the changelog` and `ok declared version 0.17.0 has a changelog entry`. The
   archive's `VERSION` and its `runbooks.json` are written at archival (task 6.3).

## The second hazard, measured with this change in place

`init` bootstrapped the stray `htmlspec/` and a `spec.nix` at `0.0.0`, as
`req-init-bootstraps-a-stray-project` says. **With the stray present, `htmlspec assets .` now
exits 0**, where before this change it reported nine duplicates:

```
note: 1 store(s) within the project's htmlspec/ were compared with one another. 5 store(s)
outside it belong to something else and were each checked on its own: ./assets, …
checking 22 stored asset(s) in 6 store(s) under .
each stored asset appears once
```

The runbook's removal step is still needed, because `spec.nix` declares a project at `0.0.0` and
the repository is not meant to be one yet. That is the proposal's recorded non-goal. What changed
is that forgetting the removal no longer turns `self-assets` red. Both were removed, and
`scripts/check` then passed all 13 stages.

## Task 6.3, after archival

The archive records `VERSION` `0.17.0` and a `runbooks.json`, written as the 0.16.0 archive's
were, from the committed runbook. It is byte-identical to that one, because the runbook did not
change. `scripts/check-release` then reported `ok 23 archived change(s) record a version, and 17
version(s) are in the changelog` and exited 0. The gap the 0.16.0 run recorded still stands: neither
the runbook's fourth step nor the OpenSpec archive workflow says to write these two files.
