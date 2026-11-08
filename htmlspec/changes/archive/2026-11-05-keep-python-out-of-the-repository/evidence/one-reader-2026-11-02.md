# One reader: the grammars that disagreed, the ports' parity, and the stages they left

Tasks 3.1 to 3.8, recorded on 2026-11-02 on the branch of the third pull request, which ports
`scripts/check-release`, `scripts/set-version`, `scripts/check-installed`, and
`scripts/release-notes` to Rust, reads every version, changelog heading, and runbook release through
`htmlspec-cli`'s library, and deletes the first three. `scripts/release-notes` stays until the fourth
pull request wires its port into the release job. The machine is the one
`the-suite-before-2026-10-30.md` names, an Apple M5 Max laptop, 18 cores and 64 GB, on macOS 27.2,
with rustc 1.90.0. Nothing here is a goal or a gate: task 6.5 measures the suite after the last pull
request, by task 1.2's method, against decision 18's goals.

## The grammars, and how the one reader settles each disagreement

Before this pull request a version was read four ways, and a changelog heading three: by
`scripts/check-release`'s `parse_version`, which took any count of parts, by `scripts/set-version`'s
`RELEASE_VERSION`, by `scripts/release-notes`, which compared a heading's text with its argument,
and by `Version::parse` in the program, with two copies of a heading reader in `runbook_release.rs`
and `staging.rs`, and a third reader of an archived change's `VERSION` in the migration survey.
Now the program, archival, migration, and the ported tools read every version by `Version::parse`'s
grammar, through one of its three kinds, `Version`, `RunbookVersion`, or `ProjectVersion`; every
heading by `runbook_release::heading_versions`; and a version a release tool takes from an argument
or a tag by `ReleaseVersion`, which refuses any text that renders otherwise than it was written.

**What still reads a version outside the one reader, until the fourth pull request.** The release
workflow's tools that group 4 ports: `scripts/pack-release`, which reads the workspace version with
`release_facts.py`'s line reader and a tarball's version with its own pattern; `scripts/render-formula`,
which reads the version from `SHA256SUMS` with a pattern of three numbers; the Python
`scripts/release-notes`, which the tag-only `release` job still runs, and which reads the
changelog's headings with its own pattern, so its port, which reads them through the one reader and
printed the same bytes for every heading, does not run at a release until task 4.5 wires it in;
and `release.yml`'s own `sort -V`, which orders tag names to decide whether a tag is the newest
release, in the `release` and `install` jobs. Tasks 4.2, 4.3, 4.5, and 4.6 replace the first three;
the `sort -V` steps are workflow shell, which the ban does not reach, and the fourth pull request
decides whether they move onto the tool. None of them reads an archive or a runbook's release.

**Versions.** The explainer found the first five; the rest were found in the port. Where the one
reader's answer differs from a Python reader's, the case names the test or fixture that holds it.

| Input | check-release (Python) | set-version (Python) | release-notes (Python) | The one reader | Held by |
| --- | --- | --- | --- | --- | --- |
| `0.1` | a version, (0, 1) | refused | refused | refused: two parts | `inventory-two-part-version.json`, `adoption-two-part-version.json`, the release check's question 1 |
| `0.1.0.0` | a version, (0, 1, 0, 0) | refused | refused | refused: four parts | `inventory-four-part-version.json`, `adoption-four-part-version.json` |
| `01.2.3` | 1.2.3 | refused | `01.2.3`, as text | 1.2.3 in a file or heading; refused as an argument or tag | `a_release_version_is_its_own_canonical_rendering` |
| `1. 2.3` | refused | refused | refused | 1.2.3 in a file; refused as an argument | `inventory-spaced-version.json` |
| `1.2.3` | 1.2.3 | 1.2.3 | `1.2.3` | 1.2.3 | |
| `+1.2.3` | refused | refused | refused | 1.2.3 in a file, as `u64`'s parser reads `+1`; refused as an argument | `inventory-signed-version.json` |
| ` 1.2.3`, `1.2.3` and a newline | 1.2.3, stripped | refused | refused | 1.2.3 in a file, trimmed; refused as an argument | `a_release_version_is_its_own_canonical_rendering` |
| `١.٢.٣`, Arabic-Indic digits | 1.2.3: `\d` matches them and `int` reads them | refused | as text | refused | `inventory-other-script-numerals.json` |
| `18446744073709551616.0.0` | a version | a version | as text | refused: past `u64` | `inventory-oversized-version.json` |
| a JSON number as an inventory's version | an exception, exit 1 | – | – | refused | `inventory-numeric-version.json` |
| the tag `v01.2.3`, declared 1.2.3 | passed | – | – | refused: not a release tag | `a_tag_that_does_not_name_the_declared_version_fails` |

So the forgiving readings `Version::parse` already gave a `spec.nix` or a runbook's `VERSION` stay,
as task 3.2 requires, and `the_one_grammar_still_forgives_what_it_forgave` holds them; and every
version a release tool is given, which a tag, a crate, and a formula must all spell alike, is
`ReleaseVersion`'s.

**Headings.** A heading is a line beginning `#`, and a version in it is a maximal run of ASCII
digits and dots holding exactly two dots that reads as a version.

| Heading | check-release (Python) | release-notes (Python) | staging.rs's copy | The one reader | Held by |
| --- | --- | --- | --- | --- | --- |
| `## 1.2.3.` or `## .1.2.3` | 1.2.3 | none | none | none | `a-heading-whose-version-ends-in-a-dot` |
| `## 1.2.3..4` | 1.2.3 | none | none | none | `a-heading-whose-version-runs-into-more-dots` |
| `## 0.1`, `## 1.2.3.4`, in the project's changelog | (0, 1), (1, 2, 3, 4) | none | none | none | `a_heading_names_the_same_versions_whoever_reads_it` |
| `## 01.2.3` | 1.2.3 | none for `1.2.3` | `01.2.3`, as text, so it explained no archive's 1.2.3 | 1.2.3 | `a_heading_names_its_version_as_the_one_reading_reads_it` |
| `## ١.٢.٣` | 1.2.3 | none | none | none | `a-heading-in-another-script-s-numerals` |

**The archive's order.** The Python ordered an archive whose `VERSION` it could not read before
every other, and compared the runbooks its inventory held as if they were the first releases. The
one reader, `runbook_release::history`, leaves such an archive out of the history, as archival does,
and the release check reports its `VERSION` under question 1.

**The manifest.** `scripts/set-version` read a table header only as `[name]` with nothing after it,
so a `[[name]]` array of tables after `[workspace.dependencies]` read as part of it. The Rust reads
both kinds of header, and a comment after one.

## The fixtures

The 23 cases the two readers shared stay in `fixtures/release/cases.json`, and twelve join them:
seven inventories, two adoption markers, and three comparisons, one for each disagreement above that
the format can hold. `the_release_fixtures_say_what_this_reader_decides` reads all 35 through
`runbook_release.rs` and passes. `scripts/check-release --self-test` passed the 23 before the new
cases joined. With them, it stopped with an `AttributeError` on the numeric version; with that case
left out of a scratch copy of the manifest, it reported 11 disagreements, every other new case:

```
!!! inventory-two-part-version.json: parsed here and the manifest says it should not
!!! inventory-four-part-version.json: parsed here and the manifest says it should not
!!! inventory-spaced-version.json: refused here and the manifest says it parses: [... states no parseable version]
!!! inventory-signed-version.json: refused here and the manifest says it parses: [... states no parseable version]
!!! inventory-other-script-numerals.json: parsed here and the manifest says it should not
!!! inventory-oversized-version.json: parsed here and the manifest says it should not
!!! adoption-two-part-version.json: parsed here and the manifest says it should not
!!! adoption-four-part-version.json: parsed here and the manifest says it should not
!!! a-heading-whose-version-ends-in-a-dot: this reader published and the manifest says refuse: []
!!! a-heading-whose-version-runs-into-more-dots: this reader published and the manifest says refuse: []
!!! a-heading-in-another-script-s-numerals: this reader published and the manifest says refuse: []

11 disagreement(s) with the shared fixtures
```

## Parity, tool by tool

Each Python tool was run beside its port, on this repository and on scratch copies of it, before
the Python was deleted. Every exit code matched.

**`scripts/check-release` and `cargo xtask check-release`.** On this repository both printed the same
four lines and exited 0:

```
  ok  52 archived change(s) record a version, and 47 version(s) are in the changelog
  ok  declared version 1.3.1 has a changelog entry
  ok  5 internal dependency pin(s) name =1.3.1
  ok  6 published crate(s) carry the root LICENSE
```

and with `--tag v1.3.1` the same five, the fifth `ok  tag v1.3.1 names the declared version`. On
eight scratch copies, each broken once, both exited 1 naming the same thing:

| Broken | Both named | Where they differed |
| --- | --- | --- |
| 1. an archive's `VERSION` removed | `2026-10-27-require-a-run-of-another-runbook records no version` | the Python also reported two downgrades of the runbooks that archive's inventory holds, having ordered it first |
| 2. an archive's version made another's | `... records version 1.3.0, which 2026-10-28-upgrade-runbooks-with-their-versions already records` | none |
| 3. the 1.3.1 heading renamed | `no CHANGELOG.md heading names version 1.3.1` | none |
| 4a. the release runbook's `VERSION` raised to 0.6.0 | `the runbook release declares 0.6.0 and its CHANGELOG.md has no heading naming it` | the Rust's sentence goes on, in archival's words |
| 4b. an inventory entry's version made `0.1` | the archive `2026-10-29-show-how-far-a-run-has-come` and the runbook `publish` | the Python read `0.1` as a version that had no heading and moved the runbook backwards; the one reader refuses the inventory |
| 5. `htmlspec-parse` pinned at `=1.3.0` | the pin, `"=1.3.0"`, the declared 1.3.1, and the version tool | the Rust names the four crates that depend on the pin |
| 6. a published crate's `LICENSE` edited | `crates/htmlspec-format/LICENSE differs from the root LICENSE` | none |
| 7. `--tag v1.3.0` | `tag v1.3.0 names 1.3.0, and Cargo.toml declares 1.3.1` | none |

**`scripts/set-version` and `cargo xtask set-version`.** On two scratch copies of the repository at
1.3.1, `scripts/set-version 1.3.2` and `cargo xtask set-version 1.3.2` each printed `version 1.3.2,
and 5 pin(s): htmlspec-schema, htmlspec-parse, htmlspec-format, htmlspec-validate, htmlspec-assets`
and `Cargo.lock relocked offline`, and the two trees' `Cargo.toml` and `Cargo.lock` were identical:
the version and five pins, and the eight workspace entries of the lockfile. Both refused `01.2.3`
and `0.1`, exiting 2.

**`scripts/check-installed` and `cargo xtask check-installed`.** On the tree both printed `ok  45
installed instruction file(s) match what `init` writes` and exited 0. With
`.pi/prompts/htmlspec-apply.md`'s stamp edited by hand to 1.3.0, both exited 1 and printed the same
seven lines, byte for byte, the first `FAIL  .pi/prompts/htmlspec-apply.md: differs from what
`init` writes`.

**`scripts/release-notes` and `cargo xtask release-notes`.** For each of the 48 versions
`CHANGELOG.md`'s headings name, and for 0.0.9 and 9.9.9, which none names, the two printed the same
188,506 bytes: 48 entries exiting 0, and two refusals exiting 1 with `release-notes: no
CHANGELOG.md heading names` and the version.

## The cases, pair by pair

`scripts/check-release`'s only tests were its self-test of the shared fixtures; the other three tools
had none. So the pairs are the fixtures, and the questions the check asks, each now a test:

| Python | Rust |
| --- | --- |
| `--self-test`, 23 cases | `runbook_release::tests::the_release_fixtures_say_what_this_reader_decides`, 35 cases |
| question 1, a change recording no version | `check_release::tests::an_archived_change_that_records_no_version_fails` |
| question 2, distinct versions above the baseline | `a_version_two_changes_record_above_the_baseline_fails` |
| question 3, a changelog heading for each version | `a_version_no_changelog_heading_names_fails` |
| question 4, the runbooks held | `a_runbook_that_cannot_name_its_release_fails` |
| question 4, the published history | `a_published_history_archival_would_refuse_fails` |
| question 5, the pins | `a_pin_that_is_not_the_declared_version_fails` |
| question 6, the licences | `a_published_crate_without_the_root_s_licence_fails` |
| question 7, the tag | `a_tag_that_does_not_name_the_declared_version_fails` |
| the passing report | `a_coherent_record_passes_and_says_what_it_counted` |
| exits 1 and 2 | `a_fault_is_named_on_standard_error_and_exits_one`, `what_cannot_be_asked_exits_two` |
| `scripts/set-version`, untested | the edit's four tests, the release runbook's two scenarios with the real cargo, the restore, the private lockfile, exit 2 |
| `scripts/check-installed`, untested | five tests on an `init` that writes what each says |
| `scripts/release-notes`, untested | four tests, and the command line's refusal of a version not written as a release writes it |

**The breaks.** Each was made, run, and undone:

- `Version::parse` made to read two parts: `the_release_fixtures_say_what_this_reader_decides` failed,
  `inventory-two-part-version.json parsed here and the shared manifest says it should not`.
- The baseline's exemption made to stop below 0.1.0 rather than at it: eight of the release check's
  eleven tests failed, the coherent record among them.
- set-version made to restore the manifest alone: `a_failure_after_the_edit_restores_every_file`
  failed.
- check-installed made to call every pair of files the same: `a_stamp_edited_by_hand_fails_naming_the_file`
  and `every_kind_of_drift_is_named` failed.
- release-notes made to start the entry at its heading: three of its four tests failed.

## How long they take

**The stages,** in two warm passes of `scripts/check` by task 1.2's method, in the development shell
under the suite lock, at `fca9877`, beside the two warm passes `the-clocks-2026-10-31.md` recorded
at the second pull request's head. Both passes passed all 16 stages.

| Stage | Before, first warm | Before, warm | After, first warm | After, warm |
| --- | ---: | ---: | ---: | ---: |
| `format` | 0.4 | 0.5 | 0.4 | 0.4 |
| `readme-install` | 1.2 | 1.1 | 1.3 | 1.1 |
| `lint` | 4.7 | 1.5 | 1.6 | 1.7 |
| `test` | 140.7 | 129.2 | 139.7 | 131.2 |
| `doctest` | 1.2 | 1.3 | 1.4 | 1.5 |
| `self-conformance` | 1.9 | 1.6 | 2.1 | 1.7 |
| `self-format` | 1.5 | 1.5 | 1.5 | 1.5 |
| `self-runbooks` | 3.1 | 3.2 | 3.2 | 3.2 |
| `self-runbooks-format` | 3.1 | 3.1 | 3.1 | 3.2 |
| `self-assets` | 1.8 | 1.8 | 1.8 | 1.9 |
| `no-python` | 0.8 | 0.6 | 1.9 | 1.8 |
| `installed` | 1.9 | 1.8 | 3.6 | 3.3 |
| `notices-self-test` | 0.2 | 0.1 | 0.2 | 0.1 |
| `notices` | 0.4 | 0.4 | 0.7 | 0.4 |
| `formula-self-test` | 2.1 | 1.8 | 2.5 | 1.9 |
| `release-fixtures` | 0.1 | 0.0 | – | – |
| `release` | 0.0 | 0.0 | 1.5 | 1.5 |
| **The pass** | **165.1** | **149.6** | **166.4** | **156.5** |

The three stages this pull request changed, and `no-python`, which runs the same tools binary, each
take about a second more than they did. The checks themselves are fast: the built binary's
`check-release` took 0.04 seconds, and `cargo metadata` 0.05, in each of three timed rounds. The
added second is `cargo xtask`'s start: `cargo xtask --help` took 1.45 to 1.52 seconds, where the
whole `no-python` stage, the tools crate's start and its check, took 0.6 to 0.8 before the crate
depended on `htmlspec-cli`. `cargo build -v` says why: `Dirty htmlspec-cli ...: the file crates/htmlspec-cli/../../.git/HEAD is missing`. The program's
build script asks to rerun when `.git/HEAD` changes, so the revision it records stays current, and
in a Git worktree such as this one `.git` is a file and that path does not exist, so every `cargo
run` recompiles the library, about a second. The `self-*` stages, which run `cargo run --bin
htmlspec`, have paid the same second in every worktree measurement, the base's included; the tools
stages pay it now because they link the library. In a checkout whose `.git` is a directory, as
continuous integration's is, the path exists and nothing recompiles. Whether the suite's sandbox,
which has no `.git` and sets no revision for the suite, pays it too was not measured here. It did,
and the review's fix, under "After the review", removes the recompile in both.
`installed` runs two binaries, the tools and the program, and so pays two starts.

So the work the nine former Python stages did took, on this warm pass, 1.1 (`readme-install`), 3.3
(`installed`), 0.1 and 0.4 (the notices), 1.9 (`formula-self-test`), and 1.5 (`release`), 8.3
seconds, with the prompt's and the publisher's cases inside `test` in under a second, against 72.2
and 72.3 at the base. Task 6.5 measures it formally after the last pull request.

**The tests,** under `cargo nextest run -p htmlspec-xtask`, one process per test: 169 tests in 0.72
seconds. None of this pull request's takes more than 0.15 seconds, the slowest being
`the_tool_writes_every_pin_and_relocks` at 0.148 and `a_version_raised_by_hand_fails_the_relock` at
0.103, which run the real cargo on a scratch workspace, and then `check-installed`'s five, each 0.08
to 0.09. In one process, `cargo test -p htmlspec-xtask` ran them in 0.45 seconds. No test of this
pull request sleeps.

## The ratchet

Three Python files are deleted: `scripts/check-release` (867 lines), `scripts/set-version` (142),
and `scripts/check-installed` (205), 1,214 lines. Their three program entries leave
`xtask/python-debt`, and so does the release runbook's pinned preflight line, which names
`python3` no more. Five pins moved with their edits, each a docstring that named a deleted script:
`scripts/check-pull-request`, `scripts/pack-release`, `scripts/release-notes`, and
`scripts/release_facts.py` in `xtask/python-debt`, and `publish/publishing.py` in
`publish/no-python`; and three of the flake's pinned comment lines moved, which no longer count the
release check, the installed-file check, or the release runbook among what `python3` is for. The
lists held 36 entries and hold 32: in `xtask/python-debt`, 10 programs (nine Python files and the
bash `scripts/check-formula`) and 15 lines; in `publish/no-python`, 4 programs, 2 lines, and the
permanent shell entry for `publish/export`. Thirteen Python files remain, nine under `scripts/` and
four under `publish/`.

**Group 1's negative cases,** rerun at `fca9877` with the debt lowered, each made, staged where a
case says staged, run with the built tools binary, and undone. Every one failed the stage, exit 1,
naming what group 1's run named:

| Case | The stage's finding |
| --- | --- |
| `scripts/tidy.py` added and staged | a Python file name, and a file under `scripts/` |
| a staged file whose first line is `#!/usr/bin/env python3` | a first line that runs python3, and begins with `#!` |
| a step running `python3 -c 1` added to `check.yml` | `.github/workflows/check.yml:72: names python3` |
| `python3` put back in the release runbook's preflight declaration | `a preflight declaration names python3`, its pin gone |
| an allow-listed `prove-listing.py` deleted | allow-listed and absent |
| a line added to `scripts/pack-release` | holds 146 lines, past the ceiling of 145 |
| `publish/no-python` moved away | `publish.yml`'s two lines and `publish/export`'s first line |
| a staged bash script `scripts/tidy` | a file under `scripts/`, and begins with `#!` |
| a staged file with the executable bit and no `#!` | carries the executable bit |
| a new hook, `.githooks/pre-commit` | begins with `#!`, and is none of the four |
| `scripts/install-hooks` deleted | allow-listed and absent |

`scripts/check-release` put back as it was at the base and staged is refused four ways, by its place
under `scripts/`, its first line, its `#!`, and its mode, and an untracked `scratch-untracked.py`
passes, as no tracked file holds it.

## After the review

Pull request 101's review, one round, found nothing that breaks a release path, the release record,
or the ratchet, and named findings settled on the same branch. The design's decisions 4, 5, 7, 12,
and 13 record each.

**The build script.** `htmlspec-cli`'s build script asked cargo to rerun it when `../../.git/HEAD`
or `../../.git/refs` changed. In a Git worktree `.git` is a file, and in the Nix sandbox there is no
repository, so both paths are missing, and cargo reran the script, and recompiled the library, on
every build: `Dirty htmlspec-cli ...: the file crates/htmlspec-cli/../../.git/HEAD is missing`. It
now asks Git, `git rev-parse --path-format=absolute --git-path HEAD --git-common-dir`, and watches
only what exists of that HEAD, `<common>/refs`, and `<common>/packed-refs`; with no repository it
watches nothing but `HTMLSPEC_REVISION`. Measured on the laptop:

| Tree | Rebuild | `--version` |
| --- | --- | --- |
| this worktree | fresh, 0.06 s, where every build recompiled the library | `9a5e6043c234-dirty`, its HEAD |
| a copy with no repository, as the sandbox has | fresh, 0.06 s | `unknown` |
| the same copy made a normal checkout, the build script rerun | fresh, 0.06 s | its HEAD |
| a commit made in it | reran, `Dirty ...: the file .git/refs has changed`, 0.86 s | the new HEAD |
| `HTMLSPEC_REVISION=abcdef123456` | fresh on the second build | `abcdef123456` |

A tree that becomes a repository after it was built keeps `unknown` until something reruns the
script, because nothing was watched; the sandbox and a checkout never change between the two.

**The check job,** `check.yml` in the Nix sandbox, where there is no repository, by its stage markers:
at the second pull request's head (run 41256984735), at this pull request's first head before the
fix (run 41540363746, its second attempt, after the first failed on a flaky topology test this pull
request does not touch), and after it (run 41550866210, at `ff6dd7b`):

| Stage | #100 | Before the fix | After the fix |
| --- | ---: | ---: | ---: |
| `self-conformance` | 23.5 | 20.1 | 16.6 |
| `self-format` | 8.0 | 7.1 | 0.1 |
| `self-runbooks` | 10.5 | 9.7 | 1.4 |
| `self-runbooks-format` | 10.2 | 9.5 | 1.3 |
| `self-assets` | 8.3 | 7.8 | 0.3 |
| `no-python` | 6.3 | 10.6 | 3.0 |
| `installed` | 8.7 | 17.2 | 0.4 |
| `release-fixtures` | 0.1 | – | – |
| `release` | 0.1 | 9.6 | 0.1 |
| **These stages** | **75.7** | **91.6** | **23.2** |

`self-conformance` still takes 16.6 seconds because it is the first stage to build the binary in the
dev profile, which `test` does not; every stage after it now finds the build fresh. `no-python`
builds the tools binary. So the fix takes about 68 seconds off the check job against this pull
request's first head, and about 52 against the second pull request's, which paid the same rerun in
its `self-*` stages. `test` took 428.8 seconds against 726.8 before the fix, a difference this change does not account for, since `test` compiles
each crate once either way; the runner's load is the likeliest cause.

**On the laptop, in this worktree,** two passes of `scripts/check` at `13b0085` with the documents
of `ff6dd7b`, by the same method, both passing all 16 stages, against the warm pass above at
`fca9877`:

| Stage | Before the fix, warm | After, first | After, warm |
| --- | ---: | ---: | ---: |
| `self-conformance` | 1.7 | 0.6 | 0.7 |
| `self-format` | 1.5 | 0.5 | 0.5 |
| `self-runbooks` | 3.2 | 2.2 | 2.2 |
| `self-runbooks-format` | 3.2 | 2.1 | 2.1 |
| `self-assets` | 1.9 | 0.8 | 0.8 |
| `no-python` | 1.8 | 1.3 | 0.7 |
| `installed` | 3.3 | 1.1 | 1.2 |
| `release` | 1.5 | 0.3 | 0.3 |
| `test` | 131.2 | 135.5 | 124.3 |
| **The pass** | **156.5** | **151.5** | **138.3** |

Every stage that runs `cargo run` or `cargo xtask` lost the second the library's recompile cost,
and the three tools stages are back near what they took before this pull request: `no-python` 0.7
against 0.6, `installed` 1.2 against 1.8, `release` 0.3 against 0.0. The work the nine former
Python stages did took 5.2 seconds on the warm pass, against 72.2 and 72.3 at the base.

**The migration survey** kept each change's `VERSION` as text, ordered by any count of dotted parts,
and grouped by text: of `01.2.3`, `1.2.3`, and `1.2.3.4`, it reported `1.2.3.4` as the greatest and no
shared version. It now reads through `ProjectVersion::parse`, as archival does, and reports `1.2.3`
as the greatest and `1.2.3` shared by the first two; `recorded_versions_are_read_as_archival_reads_them`
holds it, and the survey's `--json` values change for malformed input only.

**What else the review settled.** The shared release baseline moved from the library to
`xtask/release-facts.json`, and the library's two `const fn new` went with it. `set-version` replaces
each file whole by a rename, and its documentation no longer claims the manifest and the lockfile
move as one. `check_installed`'s and the release check's could-not-ask faults, `set-version`'s
disagreements, and a dependency's requirement are types, and the release check tells an unreadable
archive or root licence from a missing one. The changelog no longer says the release check refuses
what archival refuses, the contract table records the tag requirement's reworded clause, and the
strictness of the one reader is reasoned in decision 4.
