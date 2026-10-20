# The measurement, repeated against HEAD before any change

**Date:** 2026-10-07
**Tasks:** 1.2, which repeats sections 1, 2, and 3 of `the-measurement-2026-09-30.md`, and 1.3, which
repeats sections 4, 5, and 6
**Binary:** `htmlspec 0.23.0 (bdbab7f5fbc5-dirty)`, built from `main` at `bdbab7f` with this change's
task 1.0 edit to its own delta specification uncommitted. Every `htmlspec` below is
`./target/debug/htmlspec`, called by its path.
**OpenSpec:** `openspec` 1.13.1, from `PATH`.
**Projection:** `project-model`, rebuilt today against this repository's `crates/htmlspec-parse` at
`bdbab7f`, so it reads the projection `mark-the-words-a-modification-changed` left.

Scratch files are in the session scratchpad, under `c6/`. Nothing in the repository or in the staged
migration was written.

## Summary

Every result in sections 1 to 6 of the original measurement holds. The only differences are line
numbers in `crates/htmlspec-cli/src/commands.rs`, which moved because three changes landed in it
since. No design decision rests on a number that moved.

## Sections 1 and 2: OpenSpec's view

Both trees were extracted again with `git archive 8ca4b59 openspec` and `git archive ee4301c openspec`.

| Run | Result | As recorded |
| --- | --- | --- |
| `openspec show sync-a-change-into-the-accepted-contract --diff --no-interactive` at `8ca4b59` | exit 0, 153 lines, nothing on standard error. The first 69 lines equal `proposal.md` byte for byte (`cmp` reports no difference). Two `ADDED:` blocks, then `MODIFIED: The command surface names its operations as OpenSpec names them` with one hunk, `@@ -3,9 +3,9 @@`, whose `-` and `+` lines differ by `` `sync`, `` | yes |
| the same with `--json` | exit 1, `{"status": [{"severity": "error", "code": "show_error", "message": "Change must have a What Changes section"}]}` | yes |
| without `--diff` | exit 0, 70 lines. `diff` against `proposal.md` reports only `70d69 <`, the trailing empty line | yes |
| at `8ca4b59` with `ee4301c`'s `openspec/specs/toolchain-packaging/spec.md` | `MODIFIED: …` followed by `(no textual changes)` | yes |
| at `ee4301c`, by the name, the dated name, and `archive/<dated name>`, with and without `--json` | exit 1 each time: `Unknown item '…'. Did you mean: …`, and with `--json` the status code `unknown_item` | yes |

## Section 3: html-spec at HEAD

```
$ ./target/debug/htmlspec show                         # exit 2
error: unrecognized subcommand 'show'

Usage: htmlspec [OPTIONS] <COMMAND>

For more information, try '--help'.

$ ./target/debug/htmlspec show some-change --diff      # exit 2, the same four lines
$ ./target/debug/htmlspec --json show some-change --diff 2>/dev/null | wc -c
       0
```

`--help` lists the same fifteen subcommands as the original measurement, with no `show`. The three
`grep`s for `htmlspec show`, `show --diff`, `"--diff"`, and `Command::Show` in `crates/`, for `--diff`
in `openspec/specs/`, and for `openspec show` in the instructions print nothing.

## Section 4: the size of a modified requirement in the staged migration

```
$ cd ~/projects/html-spec-migration/staged/htmlspec
$ project-model changes/archive/*/specification.html > pm-staged-specifications.json   # exit 0
$ python3 sizes.py pm-staged-specifications.json
34 modified requirements in 19 changes
statement paragraphs: min 2, median 7.0, max 15, total 234
scenarios: min 3, median 7.0, max 18, total 286
statement words: min 62, median 364.0, max 893, total 12083
```

The same counts, and the same five largest rows in the same order. The one-word case is still 10
paragraphs, 491 words, and 18 scenarios, all `unchanged`. The staged migration is where it was, all
24 changes are archived, and `htmlspec list --json` in `~/projects/html-spec-migration/staged` gives
`{"changes": [], "followsCycles": []}`.

## Section 5: what the program derives, with today's line numbers

| What | Recorded at | Now at |
| --- | --- | --- |
| `merge_state` | `commands.rs:3545` | `commands.rs:3919` |
| the accepted map it builds, `held` | 3561 | 3935 |
| the lookup, `held.get(id)` | 3590 | 3961 |
| `resolve_change` | 939 | 1103 |
| `resolve_change_in` | 1010 | 1174 |
| `last_archived_version`, which takes `.max()` | 4647 | 5216 |
| `runbook_release::history`, its sort, and `predecessor` | `runbook_release.rs:1052`, 1162, 1209 | unchanged |

The facts at those lines are the same, with one that has grown: `held` now maps each identifier to
every accepted requirement that carries it, as projected, where at the original measurement it held
heading names only. `see-a-modification-the-merge-did-not-take` made that change, as its design
planned. `what-it-builds-on-2026-10-07.md` reads it in full.

## Section 6: the prototype

```
$ project-model specs/*/spec.html changes/archive/*/specification.html > pm-staged-all.json
$ python3 prototype.py pm-staged-all.json \
    changes/archive/2026-08-30-finish-the-archival/specification.html \
    changes/archive/2026-08-31-sync-a-change-into-the-accepted-contract/specification.html \
    req-a-single-command-exposes-validate-format-and-init
paragraph 3 (accepted paragraph 3), 48 words in common
  the accepted text holds: The spellings are therefore fixed … status, instructions, and store …
  the change states:       The spellings are therefore fixed … status, {+sync,+} instructions, and store …
statements: 10 accepted, 10 in the change, 9 held by both
scenarios: 18 accepted, 18 in the change, 18 held by both
```

Against today's staged `specs/toolchain-packaging/spec.html`, the sync change compares equal (10 of
10 paragraphs and 18 of 18 scenarios held by both), and `2026-08-30-finish-the-archival` shows only
`[-sync,-]` on the accepted side, which is the next change's word. Both runs are as recorded.
