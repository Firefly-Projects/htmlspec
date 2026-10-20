# Where testing stands, and what is untested

Measured 2026-08-17 against `main` at `d88ca93`, and against the installed
`~/.nix-profile/bin/htmlspec` reporting `0.1.0 (unknown)`.

## 1. Automated tests: 300 pass, 0 fail

`cargo test --workspace` over 14 test binaries. No failures, no ignored tests.

| Binary | Tests |
| --- | --- |
| `htmlspec-cli` unit | 97 |
| `htmlspec-validate` unit | 53 |
| `htmlspec-validate` rules | 54 |
| `htmlspec-cli` integration (`cli.rs`) | 26 |
| `conformance` | 15 |
| `htmlspec-cli` integration (`archive.rs`) | 15 |
| remaining 7 binaries | 40 |

`scripts/check` is the gate that caught the cross-store false positive during
implementation, and it stays the pre-commit command.

## 2. What the version gate has

Six integration tests in `crates/htmlspec-cli/tests/archive.rs:379-474`, one per
refusal path:

- `the_version_gate_refuses_a_version_that_has_not_moved`
- `a_projects_first_archival_is_an_ordinary_comparison`
- `the_version_gate_refuses_a_version_that_went_backwards`
- `the_version_gate_refuses_an_unorderable_version`
- `the_version_gate_refuses_an_absent_declaration`
- `the_archive_check_does_not_write_the_declaration`

Every accepted scenario under "Archival requires the project version to advance"
has a test. This surface is done.

## 3. The gap: no test reaches the prompt

`crates/htmlspec-cli/src/prompt.rs` has four unit tests. All four test
`can_prompt`, which is the *decision* to prompt. None reaches `select_tools`,
which is the prompt.

```
$ grep -rl "openpty\|pty" --include='*.rs' crates/
(no matches)
```

The design accepted this split deliberately: `prompt.rs` produces a `Selection`
and writes no files, so `commands::bootstrap` is testable from a constructed
value. That reasoning holds for what bootstrap *does* with a selection. It leaves
two things unmeasured by any automated test.

### 3.1 An empty interactive confirmation

I drove the prompt through a pty on 2026-08-14:

```
$ printf '\r' | script -q /dev/null htmlspec init
? Which tools should htmlspec set up?
> [ ] Claude Code
  [ ] Codex
  [ ] Cursor
  [ ] Pi
  [ ] AGENTS.md
```

Five unchecked boxes in a fresh repository, because nothing is detected there.
Pressing Enter confirms an empty selection.

### 3.2 Exit code 3

```
$ grep -rn "Cancelled\|, 3)" crates/htmlspec-cli/tests/*.rs
(no matches)
```

`Outcome::Cancelled` is code 3 in `exit.rs`. The unit test
`cancellation_is_a_distinct_value` asserts the value is distinct. No test runs a
process and reads 3 from it. Three of the four exit codes have integration tests
(`cli.rs:120`, `:128`, `:139`). The fourth does not.

## 4. OpenSpec comparison: two guards we lack

Read from `/opt/homebrew/lib/node_modules/@fission-ai/openspec/dist/core/init.js`
and confirmed by running the program.

### 4.1 Preselection matches

My earlier claim that OpenSpec never preselects was wrong. It preselects
detected tools:

```
$ mkdir .claude .codex && openspec init
Selected:  Claude Code   Codex        <- both [x]
```

We match here.

### 4.2 OpenSpec refuses an empty interactive selection

`init.js:361`:

```js
validate: (selected) => selected.length > 0 || 'Select at least one tool',
```

with a `throw` at `:365` as a backstop. Measured:

```
$ openspec init          # Enter on an empty list
Selected: (none selected)
Select at least one tool
$ ls -a                  # nothing written
```

Ours accepts it and reports `the project is ready`.

### 4.3 OpenSpec errors non-interactively with nothing detected

`init.js:310`. Measured:

```
$ openspec init < /dev/null
No tools detected and no --tools flag provided. Valid tools: ...
```

Ours prints an advisory and proceeds. **I recommend keeping ours.** An error here
breaks `htmlspec init` inside continuous integration, and our accepted contract
requires that init not block and not fail without a terminal.

### 4.4 The distinction we do not draw

`init.js:303`:

```js
const shouldPreselectDetected = !extendMode && configuredToolIds.size === 0;
```

OpenSpec separates *configured* (it installed there before) from *detected* (the
directory exists). Detected tools preselect on a first run only. On a re-run,
only configured tools preselect, so a re-run does not adopt a tool the
contributor never chose. It labels both states in the output:
`pre-selected for first-time setup` against `not pre-selected`.

Our `tools.rs` has one predicate, `Tool::detected`. We cannot express this.

## 5. The Pi finding, and what it was not

Two independent causes, both measured 2026-08-14.

1. **Nothing was installed.** The toy project had no `.pi/`, so nothing was
   detected, so the prompt opened empty and an empty selection was confirmed.
   The tell is the closing output: the `When the work is done, archive it:` line
   prints only when a tool was selected, and it was absent.

2. **Pi's trust gate.** Pi ignores project-local `.pi/` until the directory is
   trusted. In `/tmp`, `/htmlspec-archive` reported nothing registered. The same
   command with `--approve` loaded the skill and recited all six archive steps.
   `toy_project` inherits trust from `.` through
   `~/.pi/agent/trust.json`, which is why it works there now.

**Our paths were correct throughout.** Pi's
`dist/core/resource-loader.js:628-629` discovers `.pi/skills` and `.pi/prompts`,
which is where we write. `dist/core/prompt-templates.js:85` derives the command
name from the file name, so `htmlspec-archive.md` becomes `/htmlspec-archive`.

## 6. Open decisions

1. **Refuse an empty interactive selection?** This is additive, not an
   amendment. The accepted contract requires only that `--tools none` succeed and
   that a bootstrap installing for no tools name no command. It says nothing
   about an interactive empty confirmation. My recommendation: refuse the
   interactive empty selection, keep `--tools none` succeeding.

2. **Preselect everything when nothing is detected?** A fresh project detects
   nothing, and offering nothing is the least useful default there.

3. **Configured against detected** (section 4.4). A separate change. It only
   bites on re-runs.

4. **`htmlspec = null` in `spec.nix`.** `init` writes it, the reader parses it,
   no command reads it. The accepted contract states that at least one command
   must depend on the declaration, and `version` satisfies that. The pin does
   not. Deferred deliberately.

5. **Archive `project-bootstrap-cli`.** All 70 tasks are complete and the change
   is merged. It is still in `openspec/changes/`. Archiving it needs a version
   bump in this project's own declaration, which exercises the gate this change
   added.

## 7. One workflow ships, not five

```
$ ls crates/htmlspec-assets/instructions/
archive.md
```

`WORKFLOWS` has one entry. The proposal states this as a non-goal:
`explore`, `propose`, `apply`, and `verify` are deferred because archival is the
one operation the program already gates. So `htmlspec init` installs one skill
and one command per tool. That is correct against the current contract, and it
is worth knowing before testing a full lifecycle that does not exist yet.
