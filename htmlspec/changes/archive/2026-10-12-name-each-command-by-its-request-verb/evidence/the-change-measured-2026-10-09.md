# The measurement repeated with both spellings

**Date:** 2026-10-09
**Task:** 5.1
**Binary:** `./target/debug/htmlspec`, built from this branch's working tree. `--version` prints
`htmlspec 0.28.1 (219d1b6f1bc7-dirty)`, because the change is uncommitted.

## Method

The script of task 1.2 ran twice against the built change, with the same 42 cases, each in a fresh
committed tree from the same builder:

- once with each case's **OpenSpec spelling**, exactly as at HEAD;
- once with its **request-verb spelling**: the leading words rewritten by the table of design
  decision 3 (`validate` → `get validity`, `store add` → `post registration`, `init` → `put
  project`, …), and every argument after them passed as it was.

Each case records the exit status, `git status --porcelain --untracked-files=all` afterwards, and
any directory that appeared or vanished. The comparison drops each case's command line, which
differs by design, and compares the rest byte for byte.

## Result

| | OpenSpec spelling | Request-verb spelling |
| --- | --- | --- |
| Cases | 42 | 42 |
| Cases that changed nothing | 31 | 31 |
| Exit statuses and tree differences | equal to the other column in every case | |

**Every case's exit status and tree difference are equal under both spellings.** The verb run
reached all 19 operations: `get validity` 4 times, `get list` 4, `get show` 3, `get status`,
`get sync`, `get assets`, `get preflight`, `get instructions`, `get registrations`, and `get survey`
twice each; `post change`, `post document`, and `post registration` once each, `post run` twice,
`post promotion` 3 times; `put project` 3 times and `put format` twice; `patch archive` 3 times;
and `delete registration` once.

Among them, the cases the requirement's scenarios name:

| Request-verb invocation | Exit | Tree |
| --- | --- | --- |
| `put format --check` on a project with a non-canonical document | 1 | unchanged, as `format --check` |
| `post run release --check` | 0 | unchanged |
| `post promotion plan.html --survey survey.json` | 0 | unchanged |
| `post promotion … --staged staging --into repository --check` | 0 | unchanged |
| `patch archive demo --check`, no interrupted archival | 0 | unchanged |
| `patch archive demo --check`, an archival interrupted before its move | 0 | the journal and the 2 staged files removed, as `archive --check` removes them |
| `get show demo`, without `--diff` | 2 | unchanged, refused as `show demo` is |

**The OpenSpec spellings did not move either.** The OpenSpec-spelling summary taken with the built
change is byte for byte the one taken at HEAD in task 1.2, over all 42 cases: the restructuring of
`main.rs` changed nothing any operation writes or how it exits.

What each spelling prints is compared by the parity tests in `crates/htmlspec-cli/tests/verbs.rs`
rather than here: 59 pairs, run in copies at one location so that absolute paths agree.
