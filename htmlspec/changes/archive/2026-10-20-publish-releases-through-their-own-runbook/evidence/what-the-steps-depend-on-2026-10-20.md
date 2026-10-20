# What the steps depend on, and the accounts as they are

Tasks 1.1 and 1.2, recorded on 2026-10-20. `main` was at `ca99317`, the squash merge of
`ship-through-four-channels` (#67). This change was built on `0d287f0`, that change's archive
(#68, not yet merged), which changes nothing under `publish/`, `scripts/`, `.githooks/`, or
`.github/`.

## Task 1.1: the tooling on `main`

| What the design depends on | What `main` holds |
| --- | --- |
| `publish/export` offers `--dry-run`, `--bootstrap`, and `--push` | It does, as a mutually exclusive group, beside a default local run, `--reviewed TREE`, `--remote-url`, `--ref`, and `--keep`. |
| It refuses to push without the private review | `--push` and `--bootstrap` refuse without `--reviewed TREE` ("whatever reaches the public repository is public from that moment"), and refuse again when the tree they build is not that tree. Both also refuse without the contributor's sign-off in `publish/READINESS.md`. |
| It states its definition of a dirty tree | "a tree where a tracked file differs from HEAD, staged or not. An untracked file does not count, because `git archive` cannot see it and it cannot cross." It checks with `git status --porcelain --untracked-files=no`. Decision 6 holds, and step 9 stays last. |
| `.githooks/pre-push` refuses a push to the public repository | From any checkout holding `publish/export`, it refuses a push to `Firefly-Projects/htmlspec`, whatever ref is named, with no override, under seven exact spellings of the address. The code review, task 5.1, found three more spellings it let through, and design decision 11 makes it compare the repository an address names. |
| `scripts/publish-crates` and `.github/workflows/release.yml` exist | Both do. |

Where the design differed from the tooling, decision 2 was amended on 2026-10-20:

- The review is the export's default local run, `publish/export --ref SOURCE`, which keeps
  the scratch repository and prints the command that shows the diff. `--dry-run` reports a tree
  and keeps nothing to read.
- `--bootstrap` and `--push` each need `--reviewed TREE`.
- Against an empty target, the local review builds the bootstrap commit alone, which holds
  `README.md` and `LICENSE`. So on the first run the whole export is reviewed after the bootstrap.
- The export reads the sign-off and the reads from `publish/READINESS.md` at the ref it exports.
  A sign-off or a read reaches it only through a commit there.

The reads recorded at a commit were found to be orphaned by every squash merge, which decision
10 settles.

## Task 1.2: the accounts

Each was read with a GET request through `gh`, and nothing was written. Only the fields below
were printed, so no reply that could name a person was recorded.

`gh repo view Firefly-Projects/htmlspec --json nameWithOwner,visibility,isEmpty,defaultBranchRef`:

```
{"defaultBranchRef":{"name":""},"isEmpty":true,"nameWithOwner":"Firefly-Projects/htmlspec","visibility":"PUBLIC"}
```

`gh repo view Firefly-Projects/homebrew-tap --json nameWithOwner,visibility,isEmpty,defaultBranchRef`:

```
{"defaultBranchRef":{"name":""},"isEmpty":true,"nameWithOwner":"Firefly-Projects/homebrew-tap","visibility":"PUBLIC"}
```

`gh api repos/Firefly-Projects/htmlspec/rulesets/25362895`, the fields that decide what the
bootstrap and the push meet:

```
{"id":25362895,"name":"Protect main","target":"branch","source_type":"Repository",
 "enforcement":"active",
 "conditions":{"ref_name":{"exclude":[],"include":["~DEFAULT_BRANCH"]}},
 "rules":[
  {"type":"pull_request","parameters":{"allowed_merge_methods":["merge","squash","rebase"],
   "dismiss_stale_reviews_on_push":false,"require_code_owner_review":false,
   "require_extra_approval_for_unattributed_changes":true,"require_last_push_approval":false,
   "required_approving_review_count":0,"required_review_thread_resolution":false,
   "required_reviewers":[],"dismissal_restriction":{"allowed_actors":[],"enabled":false}}},
  {"type":"deletion","parameters":null},
  {"type":"non_fast_forward","parameters":null}],
 "bypass_actor_count":0}
```

`gh api repos/Firefly-Projects/htmlspec/rulesets` lists only this ruleset. It targets the default
branch, which the export's ruleset check reads as `main`. Nobody may bypass it, so the bootstrap's
direct push needs it disabled, as step 3 does. A squash merge is among the allowed methods.
`require_extra_approval_for_unattributed_changes` is on, and the design does not depend on it. The
export's commit is attributed to the contributor's noreply address, and the first run will show
whether GitHub asks for anything more.

`gh secret list --repo Firefly-Projects/htmlspec`, which lists names and dates and never values:

```
CARGO_REGISTRY_TOKEN	2026-10-18T23:26:44Z
HOMEBREW_TAP_TOKEN	2026-10-18T23:31:18Z
```

Both repositories are public and empty, the ruleset is active with the three rules the proposal
names, and both secrets the release workflow reads are set, as the contributor arranged on
2026-10-18.
