## Context

`scripts/check` runs a list of `name:command` pairs, reports each under a `=== name ===`
banner, collects failures, and exits 1 if any stage failed. A stage is a shell command. It
passes or it fails — the script has no warning level and no way to represent "reportable but
acceptable".

That constraint decides most of this design. The earlier draft of this work proposed a lint
inside the `htmlspec` binary, where `Severity::Warning` exists and `review/unverified-requirement`
is the precedent for advisory-and-never-gating. None of that is available in a shell stage.

The other decisive constraint is the sandbox. `flake.nix:39` filters the build source and the
derivation gets a snapshot, not a repository, so `git log` and `git diff` do not run where CI
runs the suite. The filter keeps `*.md` and `scripts/`, so the stage can be a script that
reads files.

## Goals

- The suite fails when a change is archived without a version advance.
- The suite fails when the current version has no changelog entry.
- The stage runs identically on a contributor's machine and in the sandbox.
- No change to any crate.

## Non-Goals

- Changing the shipped program, or deciding the increment. See the proposal.

## Decisions

### The check is on the archive, not on the working change

This is the decision that resolves the pass-or-fail problem, and it took the most work to
reach.

The obvious framing — "fail if the version has not advanced" — cannot work in a shell stage.
A change under way legitimately has not advanced the version yet, because the patch-versus-minor
decision is best made once the work's shape is known. A stage that failed on that state would
fail from a change's first commit to its last, and a contributor would learn to expect a red
suite. Since the script has no warning level, there is no way to soften it.

So the check does not ask about work in progress. It asks a question about the archive, which
is a settled record:

**Every archived change records a version, every recorded version is distinct, and every
recorded version has a changelog entry.**

That question has a correct answer at every moment. An unfinished change is not in the archive
and is not asked about. Archiving is the act that must satisfy the rule, which is also the
moment the increment is known — so the check fires exactly when the answer is available and
the decision is best informed.

Rejected: comparing `Cargo.toml`'s version against the newest archive and failing when they
match. That fails throughout a change's life, for the reason above.

Rejected: an advisory stage that prints and always exits zero. `scripts/check` gives a stage
no way to say "reported but passing", and a stage that never fails is a stage the suite's
`all N checks passed` line makes a false claim about.

### Each archived change records its version in a file

The check needs a version per archived change. Two places it could live:

**In the directory name** — `2026-08-21-0.2.0-distinguish-command-names`. This is what the
shipped program's `last_archived_version` already reads, so the two conventions would agree.
It also renames every future archive directory and reads awkwardly with a date already in
front.

**In a file inside the archive** — a `version` line the archival step writes. It survives a
directory rename, it is greppable, and it does not compete with the date prefix for position.

Taking the file. The convention the shipped program reads is for projects it bootstraps, and
this repository is not one of those — it has no `spec.nix` for that gate to read at all. Making
this repository's archive names match a convention that cannot apply here would be
cargo-culting the product's shape into its own repository.

The seven existing archives get their versions recorded retroactively, all `0.1.0`, which is
the truth: they all shipped under that version. That makes the "every recorded version is
distinct" rule fail on them — deliberately, and the next decision handles it.

### Seven archives at one version: a baseline, stated once

Recording the truth about the seven means seven changes share `0.1.0`, which the distinctness
rule rejects. Three options:

**Backfill invented versions.** `0.1.0` through `0.7.0`, say. Rejected: it writes a release
history that never happened, and the changelog would then describe seven releases nobody could
have installed.

**Exempt the seven by name.** A list in the script. Rejected: a list of exceptions is a list
that grows.

**Record a baseline.** The changelog's first entry covers `0.1.0` and names the seven changes
it contains, and the check treats versions at or below the baseline as one release rather than
seven. One line in the script, one paragraph in the changelog, and it says something true: the
project shipped `0.1.0` and everything before this change is in it.

Taking the baseline. It is honest about the past without inventing it, and it stops being
special the moment the eighth change advances the version.

### A script, and Python for the same reason as `check-prompt`

The stage reads a version from `Cargo.toml`, versions from the archive, and headings from a
changelog, then compares three sets. That is more than a comfortable amount of `grep` and
`sed`, and this project has already met the cost of a subtle shell pipeline being wrong.

`scripts/check-prompt` is the precedent: Python, standard library only, `#!/usr/bin/env python3`
so `patchShebangs` can rewrite it. The sandbox already has `pkgs.python3` for that script, so
this stage needs **no new build input** — which is worth stating, because the last time
`scripts/check` gained a stage the missing build input took CI red for five merges.

Rejected: Rust, as a test in `htmlspec-cli`. The version and archive layout are facts about
this repository, and a test in a shipped crate asserting them would make the product's test
suite depend on its own repository's release history.

### The stage is called `release`

Not `lint` — that is `cargo clippy`, and a second stage by that name would make
`FAILED: lint` ambiguous. `release` says what it checks. Placed last in the step list, after
`prompt`: the ordering comment in `scripts/check` says cheapest-first so a contributor is not
waiting on the suite to hear about a missing newline, and this stage is fast but least likely
to be what someone is iterating on.

## Risks

**The check passes while a change under way has no changelog entry.** By construction — it
asks about the archive, not the working tree. A contributor can therefore reach the moment of
archiving and discover the obligation then. That is the accepted cost of a stage with no
warning level, and it is mitigated by the archive workflow's own instructions, which are where
a contributor is told what archiving requires. If the miss turns out to hurt, the repair is a
line in `archive.md` rather than a stage that cries wolf.

**The baseline is a special case in the script.** One comparison against one constant, and it
retires itself: the moment the version advances past `0.1.0`, every subsequent change is
checked normally. Recorded here so a later reader does not delete it as dead code while the
seven archives still exist.

**Retroactive changelog entries are reconstructions.** Written from what each archive records,
by someone who was not there for the earliest ones. The first entry says so. A reconstruction
labelled as one is better than an absence, and better than a confident fiction.
