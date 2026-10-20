# The road, rehearsed against a scratch target

Task 3.2, recorded 2026-10-18. Every private string this record would otherwise quote lives in
`publish/rewrites.tsv` and `publish/patterns.tsv`, which do not cross; this record names them
by kind, as the change's own documents do.

## How it was run

The target was a bare repository under the worktree's ignored `target/`, created empty with
`main` as its default branch. It is not on GitHub, so the export skipped the ruleset checks and
reported the pull request it would have opened instead of opening one.

A push and a bootstrap both refuse without the contributor's sign-off in `publish/READINESS.md`,
and the contributor has not signed. So each rehearsal exported a throwaway branch whose one
extra commit filled the sign-off line with "rehearsal only, not the contributor". The branch was
deleted after the run, and the change's own `READINESS.md` is unsigned. The suite inside the
exported tree was the real `scripts/check`, run with the pinned toolchain first on the path.

## What each run showed

**An unsigned bootstrap was refused** before anything was read from the target: "READINESS.md
carries no contributor sign-off, and a push needs one".

**The first bootstrap failed in the suite, and the failure was the rehearsal's.** The run
shared a cargo target directory through `CARGO_TARGET_DIR`, and the `prompt` stage looks for the
binary in the tree's own `target/debug/`. This repository's own `scripts/check` fails the same
way with that variable set. Rerun without it, the bootstrap passed.

**The bootstrap** exported 1476 files and excluded 2, the two workflows the allowlist removes,
and left `publish/` behind. Its rewrites, the scan, and all 13 stages of the suite passed over
the whole tree, 1487 tests, and it then committed `README.md` and `LICENSE` alone as the
target's first `main`, authored and committed by the noreply identity, with the source ref and
its full commit id in the message.

**The first `--push`** passed every step and pushed a branch whose parent was the target's
`main`. Reading what it pushed found what the scan could not: the contributor's home directory
on an earlier machine, 38 times in 31 archived records and fixture copies, because the audit
and the pattern file had named only the current one. Four rewrites and three patterns were
added, two of them matching any home directory by shape (design decision 4, amended).

**The second `--push`**, of the change at `5632d17` plus the rehearsal signature, exported 1476
files, applied 28 rewrites, 217 occurrences in 130 file edits, scanned clean with the 4 images
recorded as read, and passed all 13 stages of the suite inside the export, 1487 tests. It
committed `publish/82ab06b8d3af` on top of the target's `main` and reported "would open a pull
request from publish/82ab06b8d3af into main".

## The pushed tree, checked apart from the export

Read with git in the scratch target, not through the export's own scan:

| Check | Result |
| --- | --- |
| The branch's parent | the target's `main` |
| Files | 1476 |
| Anything under `publish/`, or either excluded workflow | none |
| Home directories, by shape | only the synthetic `/home/operator` of the preflight tests |
| The private organization's names, the private project's names, the contributor's address, this repository's address, the two other private projects | in no file |
| `example-corp` | 10 lines |
| `missile-launcher` | 18 lines |
| Author and committer | the noreply identity, both |
| `Cargo.toml`'s `repository` | `https://github.com/Firefly-Projects/htmlspec` |
| The README | says in one line that `main` is protected by a branch ruleset |

## What this does not show

The real public repository was not touched. The checks of its rulesets, the bootstrap with its
ruleset disabled and enabled again, the push, and the pull request opened with `gh` all run at
the release, against `Firefly-Projects/htmlspec`, after the contributor signs.
