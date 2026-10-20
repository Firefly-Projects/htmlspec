# The road, rehearsed again after the review

Task 3.4, recorded 2026-10-19, after the revisions the review of the plan (#63) asked for. As in
the first rehearsal, every private string this record would otherwise quote lives in
`publish/rewrites.tsv` and `publish/patterns.tsv`, which do not cross, and this record names
them by kind.

## How it was run

The source was this branch at `549d761`. A push and a bootstrap need the contributor's
sign-off, which this branch rightly lacks, so the deliveries exported a throwaway branch whose
one extra commit filled the sign-off line with "rehearsal only, not the contributor". The
branch was deleted after the run.

The target was a bare repository created empty, with `main` as its default branch, in the
session's scratch directory. It is not on GitHub, so the export skipped the ruleset checks and
reported the pull request it would have opened. The suite inside the export was the real
`scripts/check`, with the pinned toolchain first on the path and `CARGO_TARGET_DIR` unset.

## Refused before anything reached the target

| Run | What the export said |
| --- | --- |
| `--bootstrap` without `--reviewed` | "--bootstrap needs --reviewed TREE: whatever reaches the public repository is public from that moment, so review the exact export first" |
| `--push` without `--reviewed` | the same, for `--push` |
| `--bootstrap --reviewed` on the unsigned branch | "READINESS.md carries no contributor sign-off, and a push needs one" |
| `--push --reviewed` with the bootstrap's tree, once `main` existed | it ran the whole road, then "the export's tree is `6bde544…`, and the tree reviewed is `ab4ce96…`", and pushed nothing |

The target held no ref after the first three, and only `main` after the fourth.

## The road

1. **A local run against the empty target.** It exported 1477 files and excluded 1, the
   publishing workflow, and left `publish/` behind. It applied 23 rewrites, 212 occurrences in
   125 file changes, scanned clean with the 4 images each read at its exact blob id, and passed
   all 13 stages of the suite inside the export, 1487 tests. It printed the tree `ab4ce96` and
   the command to review it. The scratch commit had no parent and held `README.md` and `LICENSE`
   alone, by the noreply identity.
2. **`--bootstrap --reviewed ab4ce96…`** took the same steps, built the same tree, and pushed it
   as the target's `main`, `a3455fe`, a root commit.
3. **A local run against that `main`** gave the same counts and the tree `6bde544`, committed on
   `a3455fe`. Its review command, `diff --stat refs/publish/main publish/106d844c7ce2`, listed
   1475 files changed.
4. **`--push --reviewed 6bde544…`** pushed `publish/106d844c7ce2` at `3554905`, on `a3455fe`, and
   reported the pull request it would open.
5. **The same push again** reported "replacing the earlier export of this source there
   (3554905d6666)" and moved the branch to `3532810` through the lease, with the same parent and
   tree.

## The pushed tree, checked apart from the export

Read with git and a separate script in the scratch target, not through the export's own scan:

| Check | Result |
| --- | --- |
| The branch's parent | the target's `main`, `a3455fe` |
| Root commits in the target | one, the bootstrap |
| Commits of this repository in the target | none, every object compared with `git rev-list --all` here |
| Blobs of `publish/`'s tables, `READINESS.md`, `export`, `identity`, and the deleted `main-guard.yml` | none in the target |
| Files | 1477 |
| `publish/`, `publish.yml`, `main-guard.yml` | none; the workflows are `check.yml` and `pr-guard.yml` |
| The 30 patterns, each matched over every file | one row matched: the synthetic `/home/operator` of the preflight tests, 5 times in 2 files, the occurrence that row allows |
| The expected text of the 23 rewrites | in no file |
| `example-corp` | 12 lines |
| `missile-launcher` | 20 lines |
| Author and committer, both commits | the noreply identity |
| `Cargo.toml`'s `repository` | `https://github.com/Firefly-Projects/htmlspec` |
| The README | crosses unrewritten, and says the ruleset on `main` refuses a direct push |
| `.githooks/pre-push` | crosses unchanged; its refusal of the public repository needs `publish/export` in the checkout, which the public copy lacks |

## Found by the rehearsal

A local export keeps its scratch repository for review, by design, under the system's temporary
directory. The tests review every delivery with a local run and did not point that directory at
their own, so each test run left about two dozen scratch repositories behind: 106 by the time of
this rehearsal. The tests now give the export their own temporary directory, a full run leaves
none, and the 106 were removed.

## What this does not show

The real public repository was not touched. The checks of its rulesets, the bootstrap with its
ruleset disabled and enabled again, the push, and the pull request that `gh` opens and then
updates all run at the release, against `Firefly-Projects/htmlspec`, after the contributor
signs.
