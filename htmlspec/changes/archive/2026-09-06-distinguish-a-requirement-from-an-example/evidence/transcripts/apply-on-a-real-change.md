# Apply transcript — task 6.2

**Change:** `distinguish-a-requirement-from-an-example`
**Task worked:** 6.2 — Advance the workspace version and update `Cargo.lock` offline
**Date:** 2026-09-06
**Repository:** `.` (governs itself with `openspec/`, not `htmlspec/`)
**Commit:** none. The working tree is left dirty deliberately, as instructed.

## What the task asked, and what it turned out to include

The task's own words are "advance the workspace version and update `Cargo.lock` offline". Taken
literally that is two file edits. It is not, and the reason is this change's own subject: the
version is an input to generated output, so advancing it invalidates 40 committed files.

That consequence is recorded below rather than folded silently into the version bump, because
`apply.md` directs that work no task describes is either added as its own task or recorded. Here
it is neither new work nor a divergence — task 6.1 and 6.5 already own regeneration and the check
that enforces it. What 6.2 did was *trigger* it.

## The increment: 0.9.0 to 0.10.0

A minor increment. The reasoning, stated because the contract leaves it to judgment:

`project-bootstrap`'s requirement *Archival requires the project version to advance* states that
"deciding what a version increment signifies, and by how much, is judgment", and that "advancing
the version SHALL be the contributor's or the agent's act, and the program's part SHALL be the
comparison alone". So the contract requires the version to advance and orders the result; it does
not state minor versus patch. That choice is mine, and I am marking it as mine.

What I chose on: this change adds a capability (`contract-authority`), adds a diagnostic under
`review/`, and amends all eight workflow bodies. The project has made exactly one patch release —
`0.6.1`, "work reaches main through a pull request" — and it changed no crate. Every release that
changed behaviour took a minor increment. Preserving that pattern is route 1 of the three the
design names for a question the contract leaves open.

`0.10.0` is also the increment the project's own test suite singles out.
`crates/htmlspec-cli/src/version.rs` carries `ten_is_ordered_after_nine`, and its module docstring
says why it exists: "a string comparison orders `0.10.0` before `0.9.0` — and a version gate that
[does so] is not a gate". `README.md:251` states the same for a reader. So this is the first
increment in the project's history where the componentwise ordering is load-bearing rather than
incidental, and the gate that depends on it is tested.

## What I changed

### 1. `Cargo.toml` — one line

```diff
 [workspace.package]
-version = "0.9.0"
+version = "0.10.0"
```

### 2. `Cargo.lock` — 14 lines, via `cargo update --workspace --offline`

```
Locking 7 packages to latest compatible versions
    Updating htmlspec-assets      v0.9.0 -> v0.10.0
    Updating htmlspec-cli         v0.9.0 -> v0.10.0
    Updating htmlspec-conformance v0.9.0 -> v0.10.0
    Updating htmlspec-format      v0.9.0 -> v0.10.0
    Updating htmlspec-parse       v0.9.0 -> v0.10.0
    Updating htmlspec-schema      v0.9.0 -> v0.10.0
    Updating htmlspec-validate    v0.9.0 -> v0.10.0
note: pass `--verbose` to see 5 unchanged dependencies behind latest
```

Seven workspace members relocked and no third-party dependency moved, which is what `--offline`
guarantees and what the manifest's exact pins require. `git diff --stat` reports 14 lines in
`Cargo.lock` and 1 in `Cargo.toml` — byte-for-byte the same shape as `80558b5`'s bump to `0.9.0`,
which was also `Cargo.lock | 14 +++++-----` and `Cargo.toml | 2 +-`. That correspondence is the
check that the bump did nothing extra.

`cargo` is not on `PATH` in this environment; every command was run with
`export PATH="$HOME/.rustup/toolchains/1.90.0-aarch64-apple-darwin/bin:$PATH"` first.

### 3. All 40 installed instruction files — one line each

**This is the part the task text does not mention, and it is the interesting finding.**

`generatedBy` is written from `CARGO_PKG_VERSION` at run time — `install.rs:106` and
`install.rs:161` interpolate a `{version}` that `commands.rs:34` binds to
`env!("CARGO_PKG_VERSION")`. So the moment `Cargo.toml` said `0.10.0`, all 40 committed files
still said `generatedBy: htmlspec 0.9.0`, and every one of them disagreed with the binary.

`scripts/check-installed` — the stage task 6.5 added after review of PR #21 — went red
immediately, reporting all 40 files as differing from what `init` writes.

**The stage caught, on the first release after it shipped, the exact class of drift it was written
for.** Its own docstring lists five commits where a body changed and the copies were not
regenerated, and names the consequence: `sync` "existed in the binary and in no file an agent
could read, for three releases". This time the drift lasted the length of one task.

I confirmed the stamp was the *only* difference before regenerating, by running `init` into a
scratch directory and filtering the diff for anything that was not a `generatedBy` line:

```
.claude/commands/htmlspec: 8 differing line(s)
.claude/skills:            8 differing line(s)
.codex/skills:             8 differing line(s)
.pi/prompts:               8 differing line(s)
.pi/skills:                8 differing line(s)
--- any difference that is NOT the generatedBy stamp:
(none)
```

Then I regenerated **through `init`** rather than editing the stamp with `sed`. That choice is not
cosmetic. Task 5.1 established that the two `.pi` files commit `cf11132` updated were hand-edited
rather than regenerated — after that commit they read `generatedBy: htmlspec 0.5.0` while the
workspace was at `0.6.0`, and since the stamp comes from `CARGO_PKG_VERSION` at run time, no real
`init` run could have printed it. Hand-editing a stamp is the precise failure this change's task
5.1 found by hand and 6.5 made mechanical. Doing it here would have produced a file that passes
the diff and that the real writer never wrote.

The resulting diff over all 40 files contains nothing but the stamp:

```
  24 -  generatedBy: htmlspec 0.9.0     (indented — the skill files' YAML block)
  16 -generatedBy: htmlspec 0.9.0       (unindented — commands and prompts)
  24 +  generatedBy: htmlspec 0.10.0
  16 +generatedBy: htmlspec 0.10.0
```

40 files, 40 insertions, 40 deletions. The 24/16 split is the two frontmatter shapes `install.rs`
writes, not a discrepancy: 24 skill files across `.claude/skills`, `.codex/skills`, and
`.pi/skills` carry the stamp indented inside a YAML block, and 16 files across
`.claude/commands/htmlspec` and `.pi/prompts` carry it at the left margin.

`init` was invoked as `--tools claude,codex,pi`, which is this repository's set. Not `all`: task
5.1 and `check-installed`'s docstring both record that `all` writes an `htmlspec/` project and a
`spec.nix` this repository does not use and appends 1,503 lines to the hand-authored `AGENTS.md`.
I ran `init` into a scratch directory and copied the five installed trees back, so the byproducts
(`htmlspec/`, `spec.nix`) were created in `/tmp` and never in the repository — confirmed absent
from the working tree afterwards by `ls` and by `git status`.

### 4. `tasks.md` — task 6.2 set to `done`, with what the work revealed

The note records the increment and its reasoning, the relock's shape, and the regeneration the
bump forced, including that the `installed` stage found it.

## Checks

Every command below was run after the change was complete.

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | pass |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | pass, no warnings |
| `cargo test --workspace --all-targets --locked` | pass. No `test result` line reports a non-zero failure count |
| `cargo test --workspace --doc --locked` | pass (via `scripts/check`'s `doctest` stage) |
| `cargo build --locked --bin htmlspec` then `--version` | prints `htmlspec 0.10.0 (5bb4664103a5-dirty)` |
| `scripts/check-installed` | `ok  40 installed instruction file(s) match what `init` writes` |
| `scripts/check-release` | **fails**, on one fault. See below |
| `scripts/check` | 10 of 11 stages green; `release` is the one failure |

The version-specific tests were run individually as well, since they are the ones this edit could
plausibly break. Nine passed, including `ten_is_ordered_after_nine` (the componentwise ordering
that makes `0.10.0` follow `0.9.0`), `the_reported_version_names_the_crate_version`, and
`generated_files_record_the_generating_version` — that last one is the assertion that the stamp
tracks the crate version, so it is the test that corresponds to the regeneration above. The seven
archive-side version-gate tests passed too, including
`the_version_gate_reads_the_recorded_version_not_the_directory_name`.

### The one expected failure

```
release hygiene:
!!! no CHANGELOG.md heading names version 0.10.0. A shipped version records what changed in it

1 fault(s) across 15 archived change(s), declared version 0.10.0
```

**Left in place deliberately. It is task 6.3's work and I was scoped out of it.**

`scripts/check-release` requires a changelog heading for the version the workspace declares, so it
necessarily fails between advancing the version (6.2) and writing the entry (6.3). The two tasks
are ordered that way in the change, so this red stage is the expected intermediate state and not a
defect. It clears when 6.3 adds the `## 0.10.0` heading.

Worth noting for whoever picks up 6.3: `check-release`'s own docstring anticipates a gap like this
in the other direction — "a change under way with no changelog entry passes" — because it asks
about the archive rather than the working change. What it does *not* let pass is a *declared*
version with no entry, which is what fires here. So the ordering of 6.2 before 6.3 guarantees one
red run of this stage, by design.

## What I did not touch

- No other task. 6.3 (changelog), 6.4 (`VERSION` at archival), and 5.5 through 5.8 are untouched
  and still `todo`.
- No commit, no branch, no push.
- No `CHANGELOG.md` edit of any kind.
- `VERSION` for this change is not written — task 6.4 states it is recorded at archival.
- No crate source. The only Rust-adjacent files changed are the two manifests.
- `gpt_chats/` and `notes_from_humans/` were already untracked before I started and remain so.

## Assertions in this document, and what backs them

Following the obligation the change itself adds — name the requirement or state the silence:

- That advancing the version is the agent's act and the program only compares, and that the size
  of the increment is judgment: `project-bootstrap`'s requirement *Archival requires the project
  version to advance*.
- That a change records its version in a `VERSION` file beside its documents: stated by
  `CHANGELOG.md`'s preamble and by `scripts/check-release`, and attributed to `toolchain-packaging`
  by the `project-bootstrap` requirement above. I did not act on it here — 6.4 owns it.
- That instructions record the generating version: `project-bootstrap`'s scenario *Generated
  instructions name their version*.
- **Silent:** the contract states nothing about minor versus patch, nothing about `--offline`
  versus a networked relock, and nothing requiring regeneration in the same commit as a version
  bump. The first I decided and marked as mine above. The second the task text specified. The
  third is settled by `scripts/check-installed` being a stage in `scripts/check` rather than by any
  requirement — a green suite requires them together, and that is a property of the check rather
  than of the contract.
