# Steps 1, 2, and 4, rehearsed against a local target

Task 3.1, recorded on 2026-10-20. The publish runbook's steps that need no public service were run
as its tasks state them, with the first run's bootstrap in its place, against a bare repository on
this machine. Nothing reached GitHub. As in `publish-a-public-tree`'s rehearsals, every private
string lives in `publish/rewrites.tsv` and `publish/patterns.tsv`, and this record names none.

## How it was run

- **The export was this change's `publish/export`**, which compares reads by content. The suite
  inside each exported tree was the real `scripts/check`, with the pinned toolchain,
  `rust-minimal-1.90.0`, first on the path.
- **The target was a bare repository, created empty with `main` as its default branch**, under
  this worktree's git-ignored `target/` directory rather than the session scratchpad the task
  names. The agent ran in an isolated worktree whose guard refuses a git-running tool aimed outside
  it. The export's own scratch repositories were in the session scratchpad. The target is not on
  GitHub, so the export skipped the ruleset checks and reported the pull request it would have
  opened.
- **A push and a bootstrap need the contributor's sign-off**, which this branch rightly lacks. So
  the deliveries exported `rehearsal-signoff`, a throwaway ref at `4e92325` whose one commit on
  `08bede9` filled the sign-off line with "rehearsal only, not the contributor". It was made with
  git's plumbing, so the working tree never changed, and it was deleted after the run with the
  target.

## Step 1: start from the archived release

| `SOURCE` candidate | `git grep -l -E '^X\.Y\.Z$' SOURCE -- 'htmlspec/changes/archive/*/VERSION'` | `Cargo.toml` | Step 1 |
| --- | --- | --- | --- |
| `main` at `ca99317`, for 0.33.0, before #68 merged | nothing, exit 1 | 0.33.0 | stops: the release is not archived |
| this branch at `08bede9`, for 0.34.0 | nothing, exit 1 | 0.34.0 | stops: this release is not archived yet |
| this branch at `08bede9`, for 0.33.0 | the archive of `ship-through-four-channels` | 0.34.0 | stops: the workspace declares another version |
| `main` at `c6dd3b6`, for 0.33.0, after #68 merged | the archive of `ship-through-four-channels` | 0.33.0 | passes |

Step 1 stopped wherever the release was not archived and merged, which is the runbook's
"A run publishes only an archived and merged release". So the rehearsal of steps 2 and 4 took
`08bede9`, this branch's commit with every read recorded, as a stand-in for `SOURCE`. The
publication of 0.33.0 from `c6dd3b6` would stop at step 2, because the reads `main` holds there
still name a commit. This change converts them.

## Refused before anything reached the target

| Run | What the export said |
| --- | --- |
| `--bootstrap` with no `--reviewed` | "--bootstrap needs --reviewed TREE: whatever reaches the public repository is public from that moment, so review the exact export first" |
| `--bootstrap --reviewed` on `08bede9`, unsigned | "READINESS.md carries no contributor sign-off, and a push needs one; dry runs and local exports do not" |
| `--push` on the signed ref, before any bootstrap | "the target has no main yet; run once with --bootstrap" |
| `--dry-run` on `main` at `ca99317` | "READINESS.md records the read of '.claude' by the commit 99c0fe455071; a read names the content it read, which a squash merge keeps and a commit on the merged branch loses. Record the content that commit holds, `git rev-parse 99c0fe455071:.claude`" |

## Step 2: the private review

`publish/export --ref 08bede9`, against the empty target. The export took 1517 files, excluded
one, and applied 23 rewrites at 212 occurrences. The scan was clean, with 4 binary files, each read
at its exact blob id. The reads held: every allowlisted path's content was the content its read
names. The first attempt's suite failed one stage, `test`, with three tests of the command line
(`a_clean_document_validates_and_exits_zero`, `a_project_run_reports_every_document`,
`a_document_with_no_store_above_it_has_none`). Each makes a temporary directory and expects no
project above it, and the export's scratch repository had been put inside this worktree. With the
scratch repositories in the session scratchpad, outside any project, as a run's default temporary
directory is, all 14 stages passed. The target was empty, so the review built the bootstrap commit
alone: no parent, `LICENSE` and `README.md`, tree `282cd3f4cedbbf5c6023f8bce7569e1285371a57`.

## Step 3, locally: the bootstrap

`publish/export --ref rehearsal-signoff --bootstrap --reviewed 282cd3f…`. All 14 stages passed. The
tree was again `282cd3f…`, from the new `SOURCE`, because the sign-off commit changes only
`publish/`, which never crosses. The export pushed the bootstrap commit, `13ec7e4`, as the target's
`main`. A local target has no ruleset, so the ruleset's two switches and its check were not part of
this run.

## Step 4: review again, refuse the wrong tree, push the reviewed one

- **The second review**, `publish/export --ref rehearsal-signoff`, now built on the target's
  `main`. All 14 stages passed, and the tree was `4acec3497c7a7b8a7d83d98d0468cf1fec5bb9d4`. Its
  diff against `refs/publish/main` added 1515 files. No path under `publish/` was among them, nor
  `.github/workflows/publish.yml`, and the publish runbook's four files were.
- **`--push --reviewed 282cd3f…`**, the bootstrap's tree, ran the road and refused: "the export's
  tree is 4acec34…, and the tree reviewed is 282cd3f…; … only a reviewed tree may reach the public
  repository". The target held only `main` afterwards. For this run alone the suite was replaced
  with `true`, through `PUBLISH_CHECK_COMMAND`, because the comparison comes after the suite and the
  suite had just passed on the same tree.
- **`--push --reviewed 4acec34…`** passed all 14 stages and pushed `publish/4e923253aef2`. It
  reported "would open a pull request from publish/4e923253aef2 into main", as it does for a target
  not on GitHub.

The target then held two commits, and no commit of this repository:

| Ref | Commit | Parents | Tree |
| --- | --- | --- | --- |
| `main` | `13ec7e4`, "Begin htmlspec with its README and license" | none | `282cd3f…` |
| `publish/4e923253aef2` | `29e32bb`, "Export htmlspec from rehearsal-signoff at 4e923253aef2" | `13ec7e4` | `4acec34…` |

It had one root, and none of `publish/`'s files was among its objects. Both commits carry the
noreply identity `publish/identity` names, as author and as committer. The export commit's tree is
the reviewed tree and its parent is the target's `main`. So a squash merge onto a `main` that has
not moved makes a commit whose tree is the reviewed one, which step 4's check of `MERGE` confirms on
GitHub.

## Not rehearsed

The ruleset's switches, the merge, the tag, the release workflow, the co-owner, and the channels
need the public services, so the first publication is their first run. Task 3.2 read the commands
of steps 3, 5, and 7 against GitHub's and cargo's documentation.
