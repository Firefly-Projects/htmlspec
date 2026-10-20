## Why

Seven changes are archived in this repository. `Cargo.toml` has read `version = "0.1.0"` for
all seven, and `ls CHANGELOG*` finds nothing.

Four of those changes altered output a contributor consumes: the installed workflow count went
from one to seven, the frontmatter of every generated command file changed twice, and `init`
stopped creating a document and started bootstrapping a project. Someone holding `0.1.0`
cannot tell which of those they have, and nothing in the repository records it.

`scripts/check` has eight stages — `format`, `lint`, `test`, `doctest`, `self-conformance`,
`self-format`, `self-assets`, `prompt`. **None reads the version and none looks for a
changelog.** So no check here has ever had an opinion about release hygiene, and the omission
has been invisible for seven changes because nothing was watching.

This is a gap in **this repository's suite**, not in the program it ships. The program has its
own version gate at `commands.rs:1281`, aimed at projects a contributor bootstraps with
`htmlspec init`. It does not apply here: this repository has no `spec.nix` and no `htmlspec/`
directory, so there is nothing for that gate to read, and six of the seven archives were moved
with `git mv`. Whether the shipped gate should be stronger is a separate question about the
product.

## What Changes

- **A new stage in `scripts/check`.** The suite gains a stage that reports a change archived
  without a version advance and a version with no changelog entry. It joins the eight existing
  stages and runs the same way, in the same one command a contributor and CI both invoke.

- **The stage is not called `lint`.** That name is already `cargo clippy`. A second stage
  called `lint` would make the suite's failure output ambiguous about which lint failed.

- **A `CHANGELOG.md` at the repository root**, with one entry per version and a heading that
  names the version so the stage can locate it. Written retroactively for the seven archived
  changes, because a check for entries that has none to find is untested.

- **The version becomes something a check can compare.** The stage needs a previous version to
  compare against. Recording each archived change's version where the stage can read it is
  part of this change, since without a previous value there is nothing to compare and the
  check is decorative.

- **Files, not git history.** The stage reads the working tree. `flake.nix:39` filters the
  build source and the derivation receives a snapshot with no `.git`, so `git log` and
  `git diff` are unavailable in the sandbox where CI runs. A check written against history
  would pass locally and fail in CI — the precise divergence `scripts/check` exists to
  prevent. The filter keeps `*.md` and `scripts/`, which is what makes a file-reading stage
  possible.

## Non-Goals

- **Changing the `htmlspec` program.** No new subcommand, no change to `archive_version_gate`,
  no change to any crate. This is the repository's own suite checking the repository. The
  earlier draft of this change proposed an `htmlspec lint` subcommand, which was the wrong
  scope: it would have shipped a feature to every consumer to solve a problem in one
  repository.

- **Deciding the increment.** Whether a change earned a patch, a minor, or a major stays
  judgment. The stage checks that the version moved and that a changelog entry exists. It does
  not read a diff and form an opinion about severity.

- **Rewriting the seven archived changes.** Their directories stay as they are. The changelog
  is written retroactively from what they record, which is additive; renaming them would edit
  history to satisfy a convention introduced afterward.

- **Blocking on a missing entry mid-change.** The stage's severity is settled in the design
  rather than asserted here, because a stage in `scripts/check` either passes or fails and
  there is no warning level in a shell suite. That constraint is the design's problem to
  solve.

- **Checking anything else about releases.** No tag checking, no crates.io publication check,
  no dependency audit. Two obligations, both named in the request.
