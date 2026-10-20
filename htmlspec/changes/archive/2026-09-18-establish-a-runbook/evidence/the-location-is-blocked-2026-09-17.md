# This repository cannot hold its own runbook at `htmlspec/runbooks/`

**The proposal's Impact section names `htmlspec/runbooks/` as the location, and this
repository's own release runbook inside it. The first half is right and shipped. The second
half is measurably impossible, and this file is the measurement.**

The obstruction has nothing to do with runbooks. It is that **creating an `htmlspec/`
directory at this repository's root turns the repository into an html-spec project**, and
this repository is deliberately not one — it governs itself with `openspec/` markdown.

## Measurement 1: an empty `htmlspec/runbooks/` turns `self-assets` red

One `mkdir`, no files, no `init`, nothing else:

```
$ mkdir -p htmlspec/runbooks
$ ./target/debug/htmlspec assets .
error: asset/duplicate at /assets/d8779069bc923a3c…: these 2 locations hold identical
  bytes (d8779069bc923a3c), in 2 separate store(s):
  ./fixtures/conformance/invalid-asset-unresolved/assets/sha256-d8779069bc923a3c/htmlspec.css,
  ./fixtures/conformance/valid-asset-reference-matches-stored-bytes/assets/sha256-…/htmlspec.css

1 duplicated asset(s): the project stores bytes it could share
exit 1
```

Before the `mkdir`, the same command exits 0 with `each stored asset appears once`.

**The mechanism, read out of the code rather than guessed.** `commands.rs:323` sets
`one_project = resolved.is_ok()`, where `resolved` is `ProjectRoot::resolve(root)`. That
resolution succeeds the moment `htmlspec/` is a directory — `ProjectRoot::exists` tests
`root.join(DIRECTORY).is_dir()` and nothing more. With `one_project` true,
`rules::topology::duplicates_across` runs over every store found beneath the root, and this
repository holds five: its own `assets/` plus four inside conformance fixtures that
*deliberately* carry their own stores.

The comment at `commands.rs:317` states exactly why those five must not be compared:

> A directory that merely contains several independent projects — this repository's 49
> conformance fixtures, each carrying its own deliberate store — holds no duplication to
> report: those bytes are separate projects' single copies, not one project's second copy.
> Comparing across them reported the fixture suite as duplicated, which is the check
> misreading its own subject.

So the check is right and the fixtures are right. What is wrong is the repository claiming to
be a project. `self-assets` is a stage of `scripts/check`, so this is the suite red.

## Measurement 2: `htmlspec init` here produces the nine duplicate-asset errors

This is hazard (b) from the proposal, reproduced exactly rather than quoted:

```
$ mkdir -p htmlspec/runbooks && ./target/debug/htmlspec init --tools none
established 8 shared asset(s) in htmlspec/assets
spec.nix: declared the project at version 0.0.0
the project is ready.

$ ./target/debug/htmlspec assets .
… 9 duplicated asset(s): the project stores bytes it could share
exit 1
```

**Nine, confirmed by count** (`grep -c '^error'` on the output = 9): the one from
measurement 1, plus the eight assets `init` copied into `htmlspec/assets/` that already
exist byte-identically in the repository's own `assets/`. A `spec.nix` at `0.0.0` is written
too.

The archived record at `2026-09-15-catch-a-templates-placeholder-prose` task 7.1 said "9
duplicate-asset errors", and that number is exact.

## Measurement 3: `openspec/runbooks/` fails a different check

The obvious alternative — put the runbook inside the OpenSpec mirror — fails
`every_file_in_this_repository_classifies` in `crates/htmlspec-cli/tests/migrate.rs:97`:

```
$ mkdir -p openspec/runbooks/x && echo hi > openspec/runbooks/x/tasks.html
$ cargo test -p htmlspec-cli --test migrate
assertion `left == right` failed: some files classify as no known kind:
  ["openspec/runbooks/x/tasks.html"]
  left: 1
 right: 0
```

An empty `openspec/runbooks/` passes, because the survey enumerates files rather than
directories. The moment it holds a document, it fails.

**This is the same finding `log-a-runbook-run`'s proposal already records about `FOLLOWS`**,
arriving by a second route: this repository's `openspec/` is an OpenSpec markdown mirror, so
an html-spec artifact written into it classifies as no OpenSpec kind and the coverage gate
reports it.

**A fix was considered and rejected.** Adding a `Kind::Runbook` to
`crates/htmlspec-cli/src/migrate.rs` would satisfy the test. It would also be false:
`migrate.rs`'s subject is what an *OpenSpec* project holds, and OpenSpec has no notion of a
runbook. Teaching the OpenSpec classifier to recognize one would make the survey claim a
source kind that does not exist, which is the opposite of the discipline that module's
header states.

Also rejected: a runbook inside a change directory (`openspec/changes/<change>/runbooks/`)
fails the same test. `documents/` passes, because `Kind::ConvertedDocument` already covers
it — but a change's `documents/` directory is a change's, not a runbook location.

## Measurement 4: what does pass

```
$ mkdir -p runbooks/release && echo hi > runbooks/release/tasks.html
$ ./target/debug/htmlspec assets .            exit 0
$ cargo test -p htmlspec-cli --test migrate   8 passed; 0 failed
```

A `runbooks/` directory at the **repository root** passes both. It creates no `htmlspec/`, so
the repository stays not-a-project and `self-assets` keeps its subject. It is outside
`openspec/`, so the survey never walks it and the classifier is never asked a question it has
no honest answer to.

## What was done, and whose choice it is

**The contract and the CLI say `htmlspec/runbooks/<name>/`.** That is what
`ProjectRoot::runbooks()` returns, what `htmlspec list --runbooks` reads, and what the
`runbook-authority` delta states. It is verified working in a scratch project — see
`the-cookie-recipe-2026-09-17.md`, where the recipe validates from that location with its
asset references resolving `../../assets/`.

**This repository's own release runbook is at `runbooks/release/`, at the repository root.**
The contract is silent on where a repository that is not an html-spec project keeps a
runbook, and `contract-authority`'s *Silence is recorded and not filled* gives three routes.
This is route 2 — choose reversibly and own the choice. **The choice is the agent's and not
the project's**: moving two documents is the cheapest correction available, and nothing
derives the location from anything.

The precedent is exact and already recorded: this repository records a change's predecessor
in `.openspec.yaml` rather than in the `FOLLOWS` file `project.rs` defines, for this same
reason. Two mechanisms for one fact, and the repository dogfooding OpenSpec cannot use the
html-spec form.

**The cost, stated rather than discovered later.** This repository's own release runbook does
not sit where the contract says a runbook sits, so it is not a dogfooding of the location —
only of the documents. The location is dogfooded in a scratch project instead, which is
weaker. What would close it is migrating this repository to `htmlspec/`, and that is
`migrate-an-openspec-project`'s unfinished half rather than this change's work: doing it here
would mean converting 60-odd prose documents inside a change about runbooks.
