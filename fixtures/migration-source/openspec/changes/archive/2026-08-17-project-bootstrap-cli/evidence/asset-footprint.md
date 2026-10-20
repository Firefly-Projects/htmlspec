# Asset footprint, before and after

Measured 2026-08-14 with `du -sh`, in fresh repositories.

## Before

Each change established its own store, because `init` derived the store location
from the *document's* directory when `--assets` was absent:

```rust
let assets_directory = match assets_root {
    Some(root) => root.join("assets"),
    None => document_directory.join("assets"),
};
```

```
$ htmlspec init htmlspec/changes/a --change
$ htmlspec init htmlspec/changes/b --change
$ du -sh htmlspec/changes/a/assets htmlspec/changes/b/assets
6.0M	htmlspec/changes/a/assets
6.0M	htmlspec/changes/b/assets
```

One store per project was available, but only by passing `--assets` on every
invocation. The default was a copy per change.

## After

The store is derived from the project — in `ProjectRoot::assets()`, and nowhere
else — so one store per project is the only reachable outcome.

```
$ htmlspec init --tools claude
$ htmlspec new change add-dark-mode
$ htmlspec new change b && htmlspec new change c && htmlspec new change d
$ du -sh htmlspec
6.1M	htmlspec
$ ls htmlspec/changes/*/assets
(no matches)
```

| Changes | Before | After |
| --- | --- | --- |
| 1 | 6.0 MB | 6.0 MB |
| 2 | 12 MB | 6.0 MB |
| 4 | 24 MB | 6.1 MB |
| *n* | *n* × 6 MB | 6 MB |

The 0.1 MB above the store is the documents themselves.

## The check that could not see it

`htmlspec assets` existed to report exactly this duplication and could not
observe it, because it called `Store::discover` on the directory it was handed
and did not descend:

```
$ htmlspec assets          # with 12 MB duplicated two directories down
no asset store under ., so there is nothing to check
$ echo $?
0
```

After descending:

```
$ htmlspec assets          # same project, two per-change stores
error: asset/duplicate at /assets/c36254270219eee5...: these 2 locations hold
identical bytes (c36254270219eee5), in 2 separate store(s): .../changes/a/assets/...,
.../changes/b/assets/...
8 duplicated asset(s): the project stores bytes it could share
```

**8 duplicated assets reported where the old path reported none.** Pinned by
`the_asset_check_reports_duplication_between_two_stores`.

### Why a second function was needed

`Store::duplicates` compares within one store and cannot see this: two addressed
copies in two different stores each sit alone at their own address, so every
store reports itself clean while the project holds the bytes twice. That is why
`topology::duplicates_across` exists. The design said "assets checks the whole
project" without noticing that needed a second comparison.

### And why it is scoped to one project

Cross-store comparison is a statement about **one project**. Run over a directory
that merely contains many independent projects it is wrong, and it was: the first
version reported this repository's own conformance fixtures as duplicated, since
49 of them carry a deliberate store apiece and two hold the same stylesheet
bytes. Those are separate projects' single copies, not one project's second copy.

The check now applies cross-store comparison only where a project resolves, and
says so otherwise:

```
$ ./scripts/check
=== self-assets ===
note: . is not a project, so its 5 stores were each checked on their own. Bytes
shared between separate projects are not duplication
checking 12 stored asset(s) in 5 store(s) under .
each stored asset appears once
```

`scripts/check` caught this, which is the argument for having a self-check that
runs the tool against its own repository: the unit tests all passed.

## The property given up

A change directory is no longer self-contained. Copied out of its project, its
references still climb three levels for a store that is not there:

```
$ cp -R htmlspec/changes/dark-mode orphan && cd orphan
$ htmlspec validate . --strict
3 document(s): 21 error(s), 0 warning(s)
```

Before this change the same orphan reported **0 errors and was accepted** — 21
dead references, silently. That gap is closed by the ascending-reference rule,
and the trade is deliberate: the alternative is 6 MB per change.
