# What still reads the OpenSpec project, measured before the change

**Date:** 2026-10-14
**Tree:** this repository at `4fa4e96`, with `htmlspec 0.30.0 (4fa4e96992f8)` built from it.

The migration plan's decision 4 says the change after promotion "points `scripts/check-release`, the
`self-runbooks` stages, and the migration tests at `htmlspec/`, and removes `openspec/`". This
records what reads `openspec/` and the root `runbooks/` today, what removing `openspec/` does to each
reader, and the facts the change's decisions rest on.

## The two projects

The repository holds its project twice. `openspec/` has 473 files and 36 archived changes, the last
recording 0.29.0. `htmlspec/` has 38 archived changes: the same 36, and the two archived since
promotion with `htmlspec archive`, recording 0.29.1 and 0.30.0. `openspec/runbook-releases.json` and
`htmlspec/runbook-releases.json` are byte-identical (adoption at 0.15.0).

## What reads `openspec/` or the root `runbooks/`

Found by searching every tracked file outside `openspec/` and the archive for `openspec` and
`runbooks/`, then reading each hit. Mentions of OpenSpec as the migration's source format, in the
program and its tests, are not readers of this repository's directory and are not listed.

| Reader | What it reads |
| --- | --- |
| `scripts/check-release` | `openspec/changes/archive/` (line 638), `openspec/runbook-releases.json` (line 454), and runbooks under `runbooks/` (`RUNBOOKS`, line 116) |
| `scripts/check`, `self-runbooks` and `self-runbooks-format` | `htmlspec validate runbooks` and `htmlspec format --check runbooks` |
| `crates/htmlspec-cli/tests/migrate.rs` | `htmlspec migrate survey --json` on the repository root, and an independent walk of `openspec/` |
| `conformance/tests/presentation.rs` | `openspec/changes/*/documents/` and `openspec/changes/archive/*/documents/` |
| `crates/htmlspec-validate/src/rules/assets.rs`, `repository_documents` | every `.html` under `README.html`, `assets`, `fixtures`, `openspec`, `runbooks`, and nothing under `htmlspec/` |
| `flake.nix` | nothing; its comment says `openspec/` is what the tests survey, and keeps `yaml`, `txt`, `py` for it |
| `scripts/check-installed` | nothing; its docstring and two comments say the repository governs itself with `openspec/` |

## What removing `openspec/` does to each reader

`openspec/` moved aside, each reader run, and `openspec/` put back (473 files, `git status` clean):

```
cargo test --locked -p htmlspec-cli --test migrate
  test result: FAILED. 2 passed; 9 failed
  the survey failed: htmlspec: …/crates/htmlspec-cli/../.. is not an OpenSpec project:
  no openspec/specs or openspec/changes directory at …
cargo test --locked -p htmlspec-conformance --test presentation
  test result: ok. 7 passed; 0 failed
cargo test --locked -p htmlspec-validate --lib the_formatter_finds
  test result: ok. 2 passed; 0 failed
python3 scripts/check-release
  no archived changes under …/openspec/changes/archive
  exit 2
```

With `openspec/` present, all eleven migration tests pass. The presentation and asset tests pass
either way, and that is the defect the table above predicts. Each skips a place it cannot read, so
without `openspec/` the presentation checks read 4 fewer documents (the change documents under
`openspec/changes/archive/*/documents/`, 11 down to 7, and the count assertion asks for 5) and the
asset test reads 7 fewer. Neither test reads `htmlspec/`, which holds 172 `.html` files.

**The migration tests cannot be pointed at `htmlspec/`.** The survey refuses a directory that is not
an OpenSpec project, and `htmlspec/` is not one. The tests need an OpenSpec source. They assert facts
about this repository's: that `2026-08-17-project-bootstrap-cli` and `2026-08-30-finish-the-archival`
rename a requirement, that the version several changes share is the baseline
`scripts/check-release` names (`(0, 1, 0)`), and that the greatest recorded version has a changelog
heading.

## The retargeted release check

`scripts/check-release` with lines 454 and 638 pointed at `htmlspec/`, copied to a scratch file and
run against this tree:

```
  ok  38 archived change(s) record a version, and 32 version(s) are in the changelog
  ok  declared version 0.30.0 has a changelog entry
```

The unmodified script reports 36 and 31. The two archives `htmlspec archive` wrote are checked by
nothing in the suite today.

## The release runbook

`htmlspec list --runbooks` reports `No active runbooks`. `htmlspec/runbooks/` does not exist, so both
archivals since promotion recorded an empty inventory:

```
htmlspec/changes/archive/2026-10-14-resolve-a-new-documents-project-from-its-destination/runbooks.json
htmlspec/changes/archive/2026-10-14-keep-archived-documents-pointing-at-their-store/runbooks.json
  {"format": "htmlspec-runbook-inventory/v1", "runbooks": []}
```

The last snapshot of `release` is 0.29.0's (`2026-10-12-name-each-command-by-its-request-verb`), and
the runbook's files are still the bytes it fingerprints:

```
release 0.1.0 sha256:3d7efaec14b97d656d65423822cea50e770a172e01956e4985a9ef9ec29f177c sha256:e6a4410e6261b9f77a5c90ec3e60fcd56ff25726207abf23e586c7398dffc9a9
3d7efaec14b97d656d65423822cea50e770a172e01956e4985a9ef9ec29f177c  runbooks/release/specification.html
e6a4410e6261b9f77a5c90ec3e60fcd56ff25726207abf23e586c7398dffc9a9  runbooks/release/tasks.html
```

`predecessor` in `crates/htmlspec-cli/src/runbook_release.rs` takes the last snapshot whose name
matches, across the whole ordered history, so an empty inventory does not interrupt a runbook's
lineage.

Both documents reference `../../assets/` for seven files. From `runbooks/release/` that is the root
`assets/`; from `htmlspec/runbooks/release/` it is `htmlspec/assets/`. `htmlspec/assets/` holds five
of the seven. The two it lacks are the stylesheet revisions `sha256-e4f5de7cecc4f691/htmlspec.css`
and `sha256-0350261bb5666167/htmlspec-print.css`, which the root `assets/` holds and
`assets/vendor.lock.json` records.

**The runbook's text no longer describes this repository.** Its specification says "This repository
governs itself with openspec/ markdown, so it is not an html-spec project and does not want one", and
requires a run to "remove the htmlspec/ directory and the spec.nix that the bootstrap command creates
in this repository's root". Its procedure's step `task-regenerate-the-instructions` says the same. Since
#55 those are the project and its declaration. `htmlspec init --tools claude,codex,pi`, run in a scratch
repository holding a copy of `spec.nix`, `AGENTS.md`, and an `htmlspec/specs/` directory, changed
neither file and wrote only `htmlspec/assets/`. Following the step would delete the project.

## The OpenSpec workflow files

52 tracked files, each instructing an agent to run the `openspec` CLI (`openspec new change`,
`openspec status --change`, `openspec instructions`) against `openspec/`:

```
.claude/skills/openspec-*     10
.claude/commands/opsx/        10
.codex/skills/openspec-*      10
.pi/prompts/opsx-*            11
.pi/skills/openspec-*         11
```

No settings file exists under `.claude/`. One tracked file outside `openspec/` and the archive names
them: a comment in `crates/htmlspec-cli/src/install.rs` cites "`.pi/prompts/opsx-*.md` omit the field
across ten files and load" as the observation behind writing a prompt with no `name:`.

## Prose

`README.md` and `AGENTS.md` name no `openspec/` directory of this repository. `README.md` names
OpenSpec, the project HTMLSpec started from, and shows `runbooks/release/` only inside the layout of a
portable copy, `portable/htmlspec/runbooks/release/`. `AGENTS.md` names neither.

Comments in the program's sources cite evidence by the path it had before its change was archived,
`openspec/changes/<name>/evidence/…`: `crates/htmlspec-cli/src/runs.rs`,
`crates/htmlspec-validate/src/rules/runs.rs`, `crates/htmlspec-schema/src/lib.rs`,
`crates/htmlspec-parse/src/diagnostics.rs`, and a failure message in
`crates/htmlspec-parse/tests/parse_errors.rs`. Each has named a path that does not exist since that
change was archived, in August or September.

## Whole-project checks, for reference

`htmlspec validate htmlspec`: 165 documents, 0 errors, 1073 warnings, exit 0. `htmlspec format --check
htmlspec`: every document canonical, exit 0. No stage of `scripts/check` runs either.
