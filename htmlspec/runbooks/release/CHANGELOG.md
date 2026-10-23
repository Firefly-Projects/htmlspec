# Changelog

What this runbook's releases are. A release is the pair of canonical documents —
`specification.html` and `tasks.html` — as they stood when the project archived them, and a version
identifies one such pair. Nothing else moves it: a run's record is history rather than procedure, so
adding or editing one is not a release.

## 0.5.0

Published with the project's 1.1.0 release.

**The procedure renders as the project's specifications do, and draws its order.** Both documents
move onto the current stylesheet, through `htmlspec format --upgrade-assets`, which rewrites their
references to the store's current assets and downloads nothing. The stylesheet they referenced had
no styling for a partial outcome, a renamed requirement's heading, or a change's inserted and
deleted words. The tasks document gains "The order of the steps": the flowchart the change that
established this runbook drew in its design, with the hazard that puts the version ahead of
regeneration on the edge between them.

**What this release does not do.** The four steps, their order, their words, and the requirements
are unchanged. A run reads the same procedure it read at 0.4.0.

## 0.4.0

Published with the project's 1.0.1 release.

**`python3` is declared, and the shell supplies it.** 0.2.0's changelog recorded that the release
check's Python stages fail inside `nix develop` on macOS, where the system `python3` cannot run,
and that this runbook did not yet say so. The development shell now supplies `python3`, and
`req-the-pinned-toolchain-is-what-runs` names it beside `cargo` and `rustc`, so a run whose
`python3` resolves outside the shell is refused before it starts. The specification's paragraph
that kept `python3` undeclared is replaced: its reason was the exact version a `tool` check states,
and the `toolchain` declaration states none. The specification gains the scenario of an interpreter
that resolves outside the shell, the list of what the procedure consumes says the shell supplies
`python3`, and the hazard that the inputs are prose now counts the interpreter among what preflight
checks.

**What this release does not do.** The four steps, their order, and the other requirements are
unchanged. `git` stays undeclared.

## 0.3.0

Published with the project's 0.33.0 release.

**The version step runs `scripts/set-version`.** From 0.33.0 each internal dependency in the
workspace manifest pins the workspace's exact version, so the six crates published together depend
on exactly one another. A version raised by hand leaves every pin naming the old one, and the
offline relock exits 101 before the release check can say why. The step now runs the script, which
writes the version and every pin in one edit and relocks, and the specification gains the
requirement `req-the-version-moves-with-its-pins`, with the version raised by hand and a stale pin
as its scenarios. The release-check step names the two new questions the check asks: the pins, and a
`LICENSE` in every published crate.

**Both documents carry the fallback stylesheet**, added by `htmlspec format --fallback`, so they
stay readable where their stylesheet cannot load. `read-a-document-whose-stylesheet-cannot-load`
left that to this runbook's next release.

**What this release does not do.** The four steps and their order are unchanged, and so are the
other six requirements. Publishing the release to its channels is not a step here: that belongs to
the publish runbook a later change adds.

## 0.2.0

Published with the project's 0.30.2 release.

**This runbook no longer tells a run to delete the project.** 0.1.0 was written when this repository
kept its specifications in OpenSpec's markdown and was not an html-spec project, so the `htmlspec/`
and `spec.nix` that regenerating the instruction files bootstrapped were stray, and its regeneration
step said to remove both. Since the repository became its own project at 0.29.0, those are the
project and its declaration. Measured, the removal deletes 504 files, and the asset-topology stage
that 0.1.0 named as the check that catches a mistake there passes afterwards.

The regeneration step now runs from the repository root and removes nothing. It names what `init`
reports when it extends this repository's project and what it reports when it creates one instead,
and says to stop the run on the second, or on any path `git status` lists after `init` and did not
list before it, other than the installed instruction files and an asset `init` reports establishing.
The listing before is the baseline because the version the step before advanced is not committed yet.
The requirement behind the step is replaced: `req-init-bootstraps-a-stray-project`
is gone, and `req-regenerating-leaves-the-project-in-place` states the opposite obligation, with the
three cases measured as its scenarios.

**The recorded version is described as `htmlspec archive` writes it.** A run no longer writes the
archived change's `VERSION` by hand. Archival writes it from `spec.nix`, and the runbook now requires
a run to raise `spec.nix` at archival to the version it set in `Cargo.toml`, because nothing compares
the two.

Also corrected: the release check asks four questions of the archive, not three; the procedure
consumes a repository whose project is in place; and the pinned development shell supplies the
toolchain but not `python3`.

**What this release does not do.** The four steps, their order, and the other four requirements are
unchanged. The release check's Python stages still fail inside `nix develop` on macOS, where the
system `python3` cannot run, and this runbook does not yet say so. `init` run beneath the project
still establishes a second one there; the step avoids it by naming the root, and nothing prevents
it.

## 0.1.0

The first release of this runbook under a version of its own, published with the project's 0.15.0
release.

**This is an adoption, not a first edition.** The procedure has existed since the project's 0.13.0
release, and it was run five times before that under four different section numbers of four archived
`tasks.md` files. None of those runs is a release of this runbook, and no independent version is
invented for them: there was no version to record, and back-dating one would describe releases
nobody ever installed.

What 0.1.0 adds is the ability to name itself where the project is absent. A runbook carried to
another machine arrives without the archive that would otherwise say where it came from, so its
history cannot be derived — it has to be declared. That is the whole of what this version buys, and
the cost is that every later edit to these two documents owes a greater version and a heading here.

Also new at this release: the specification states the toolchain this procedure needs as a machine
requirement, so `htmlspec preflight release` reports a missing or shadowed Rust toolchain before the
run starts rather than at the step that needs it.
