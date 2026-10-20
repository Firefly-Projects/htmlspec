# The release runbook read against the repository it releases

**Date:** 2026-10-14
**Tree:** `govern-this-repository-from-htmlspec` at `3b45ccb`, with `htmlspec 0.30.1 (3b45ccb06d69)`
built from it. The runbook is `htmlspec/runbooks/release/` at version 0.1.0, byte-identical to what
0.29.0's archive published: `specification.html` hashes to `sha256:3d7efaec…` and `tasks.html` to
`sha256:e6a4410e…`, the fingerprints that inventory records.

This reads every statement of the runbook's two documents and its changelog against the repository as
it stands, and measures what following each step does. Every command that writes was run in a scratch
copy extracted from `git archive HEAD`, beside a second, untouched copy to diff against. Nothing here
touched the repository.

## What the runbook says that is no longer true

Searched for `openspec`, `runbooks/release`, `not an html-spec`, `not already`, `stray`, `0.0.0`,
`nine`, `tasks.md`, `three questions`, `spec.nix`, `htmlspec/`, and `self-assets`, then read both
documents whole.

| Where | What it says | As the repository stands |
|---|---|---|
| tasks, `task-regenerate-the-instructions` | "`init` here also bootstraps a stray `htmlspec/` and a `spec.nix` at `0.0.0`. Remove both, and run `scripts/check` afterwards — `self-assets` is the stage that catches them." | `htmlspec/` and `spec.nix` are the project and its declaration. `init` creates neither (M2). Removing them deletes the project (M4), and `self-assets` passes afterwards (M4). **Destructive.** |
| specification, `req-init-bootstraps-a-stray-project` | "A run of this procedure MUST remove the `htmlspec/` directory and the `spec.nix` …" because "This repository governs itself with `openspec/` markdown, so it is not an html-spec project". | The premise has been false since #55 promoted the project at 0.29.0. `openspec/` is now `fixtures/migration-source/openspec/` (moved by `govern-this-repository-from-htmlspec`). The MUST is the destructive step's authority. |
| specification, that requirement's two scenarios | Nine duplicate assets with the stray project left, and a failure with only the empty directory left. | The repository resolves as a project, and the asset check compares only the stores inside it with one another (M3). Neither failure can occur. |
| specification, `evidence-nine-duplicate-assets` | "Running the bootstrap command in this repository and then the asset-topology check reports exactly nine duplicated assets". | True of the repository before 0.29.0, and false now (M2, M3). It supports the requirement above. |
| specification, `verify-the-stray-project` | Four readings of the asset check, from a clean tree with no project. | No clean tree of this repository is without a project. It verifies the requirement above. |
| tasks, "What this procedure consumes" | "**A repository that is not already an html-spec project.** That assumption is what hazard two is about". | Inverted: the procedure now consumes a repository whose project is in `htmlspec/` and declared in `spec.nix`. |
| specification, `risk-the-inputs-are-prose` | "a repository that is not already a project", and "the pinned development shell, which supplies all four when a run happens inside it". | The first is inverted. The second is false: `flake.nix`'s `devShells.default` lists `rustToolchain`, `cargo-nextest`, and `jq`, and no `python3`. Its "Not mitigated" was also overtaken by 0.1.0's own preflight declaration, which checks the toolchain before a run starts. |
| tasks, `task-confirm-the-release-check` | "It asks three questions of the archive … plus one about the version the project currently declares." | `scripts/check-release` asks four of the archive, the fourth being "Every published runbook release is accounted for", which `carry-a-runbook-to-another-machine` added at 0.15.0, the release that adopted 0.1.0. Stale since then, not because of the move. |
| specification, `verify-the-increment-is-not-derived` | "It asks three of the archive … None of the four computes what the next version should be". | Same count, same cause. Re-read: none of the five questions computes a next version, so the verification's conclusion holds. |
| specification, `req-the-recorded-version-waits-for-archival`, and the same task's note | A run "MUST NOT attempt to write the `VERSION` file … before archival, and MUST leave that step open until archival has run". | This is the OpenSpec workflow's archival, in which a run wrote `VERSION` by hand. `htmlspec archive` writes it itself, from the version `spec.nix` declares (`archive.md`: "records the declared version in a `VERSION` file"), and the runbook names `spec.nix` nowhere. The three releases archived with `htmlspec archive`, 0.29.1, 0.30.0, and 0.30.1, each advanced `Cargo.toml` during implementation and left `spec.nix` to archival (their tasks 4.1). Nothing compares the two versions. |

## What the runbook says that still holds

- **`task-advance-the-version`, `task-write-the-changelog`, and `req-version-before-regenerate`,
  `req-the-increment-is-judgment`, `req-the-changelog-heading-names-the-version`,
  `req-the-pinned-toolchain-is-what-runs`.** The three releases since promotion each regenerated after
  the version advanced and changed one line in each of 45 files. `scripts/check-release` still reads
  headings only.
- **`archive.md`.** Both documents cite it for the increment being judgment. It is
  `crates/htmlspec-assets/instructions/archive.md`, the htmlspec workflow, not an OpenSpec file, and
  line 307 still says so: "Deciding whether the change earned a patch, a minor, or a major increment is
  judgment".
- **History.** The five archived release sections, `tasks.md`, "6, 8, 6, 9, 7", and the 0.1.0
  changelog entry describe the procedure before 0.1.0. They direct no step. Those archives are now held
  as HTML in `htmlspec/changes/archive/`, where the release sections are `<h2>6. Release</h2>` and so
  on, and as the Markdown they were in `fixtures/migration-source/openspec/changes/archive/`.
- **Nothing names `openspec archive`, an OpenSpec workflow by name, or the root `runbooks/release/`.**
  The documents link `../../assets/`, which from `htmlspec/runbooks/release/` is the project's store,
  and the move copied the two revisions they need into it.

## Measurements

Each was run with the binary above, from the scratch copy's root unless stated, after extracting a
fresh copy where an earlier measurement had written into one. Output is quoted with the scratch path
elided as `…`.

**M1. What the removal names.** `htmlspec/` holds 504 files: 13 accepted specifications, 459 files of
38 archived changes, the 4 files of this runbook, 10 files of the asset store, and the active change
`govern-this-repository-from-htmlspec`. `spec.nix` declares `version = "0.30.0"`.

**M2. `init` in the project, from the root.**

```
extending the project in …/measure
…/measure/spec.nix already exists, so it was left as it is
Claude Code: 18 file(s) in .claude
Codex: 9 file(s) in .codex
Pi: 18 file(s) in .pi
```

`diff -rq` against the untouched copy reports nothing. At an unchanged version the 45 installed files
are already what `init` writes, so nothing at all changed. At a release, the three html-spec releases
record the 45 files' `generatedBy` line as the only change.

**M3. The asset check after `init`.**

```
note: 1 store(s) within the project's htmlspec/ were compared with one another. 5 store(s) outside it belong to something else and were each checked on its own: …
checking 26 stored asset(s) in 6 store(s) under .
each stored asset appears once
```

**M4. The removal 0.1.0 prescribes, followed: `rm -rf htmlspec spec.nix`.**

| Check | Exit | Reports |
|---|---|---|
| `scripts/check-release` | 2 | `no archived changes under …/htmlspec/changes/archive` |
| `htmlspec validate htmlspec/runbooks` (the `self-runbooks` stage) | 2 | `no such file or directory: htmlspec/runbooks` |
| `htmlspec list --runbooks` | 2 | `no htmlspec/ directory … so there is no project here` |
| `htmlspec archive govern-this-repository-from-htmlspec --check` | 2 | the same |
| `htmlspec assets .` (the `self-assets` stage) | **0** | `checking 16 stored asset(s) in 5 store(s) under .` / `each stored asset appears once` |

The stage 0.1.0 names as the one that catches a mistake at this step passes. Two other stages fail.
Version control restores what a commit holds. It does not restore a change under way that no commit
holds.

**M5 and M7. `init` with the project gone.** It establishes a new one:

```
…/htmlspec: created the project structure
established 8 shared asset(s) in …/htmlspec/assets
…/spec.nix: declared the project at version 0.0.0
```

`htmlspec assets .` then passes, with 24 assets in 6 stores. `scripts/check-release` exits 2.

**M6. `init` from `crates/`.** It does not find the project above it and establishes one in `crates/`:

```
…/crates/htmlspec: created the project structure
established 8 shared asset(s) in …/crates/htmlspec/assets
…/crates/spec.nix: declared the project at version 0.0.0
```

`diff -rq` reports `crates/.claude`, `crates/.codex`, `crates/.pi`, `crates/htmlspec`, and
`crates/spec.nix`, each new. The root's project is unchanged. So "created the project structure" is
what `init` says when it is not extending this repository's project, from the wrong directory (M6) or
with the project missing (M7). "extending the project in" the root is what it says when it is (M2).

## Found, and left to other changes

- **The four Python stages fail inside `nix develop` on macOS.** `prompt`, `installed`,
  `release-fixtures`, and `release` report `error: tool 'python3' not found`, because the development
  shell supplies no `python3` and macOS's `/usr/bin/python3` is a shim that fails there. Each release
  since promotion recorded it and ran the four outside the shell with the shell's `cargo` on the path.
  It is a hazard of the machine rather than of the repository's layout, and it destroys nothing. This
  change corrects only the runbook's false statement that the shell supplies all four inputs.
- **`init` run beneath a project establishes a second one** (M6). project-bootstrap's "Commands resolve
  the project from any directory beneath it" requires that of "every command whose subject is a
  project", and the contract does not say whether `init` is one. The runbook avoids it by naming the
  root.
