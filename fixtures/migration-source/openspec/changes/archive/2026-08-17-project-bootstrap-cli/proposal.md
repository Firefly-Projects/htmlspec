## Why

The first thing anyone does with htmlspec is the thing htmlspec does worst.

OpenSpec's `openspec init` is the reason people adopt it. It detects which AI coding tools a project already uses, offers a searchable multi-select of them, writes the directory structure, installs a skill and a slash command per workflow, and closes with the exact command to type next. A new user goes from an empty repository to `/opsx:new "your idea"` without reading documentation.

`htmlspec` has no equivalent, and the gap is not cosmetic. Running the program in a fresh repository produces this:

```
$ htmlspec
Validate, format, and create HTMLSpec documents
Usage: htmlspec [OPTIONS] <COMMAND>
...
$ htmlspec init
error: the following required arguments were not provided:
  <PATH>
```

Both are correct given the current design and both are dead ends. The program's `init` creates *one document*; it has no concept of a project. So there is nothing to run that establishes an htmlspec project, and the three behaviors a new user actually meets are these:

1. **`htmlspec init` demands a path with no default.** There is no argument that means "set this repository up," because that operation does not exist.

2. **`htmlspec init . --change` refuses.** The obvious guess — initialize a change here — fails with `. already exists; init will not write into an existing change directory`. A contributor pointing at their own repository root gets told it exists, which they knew.

3. **Every change carries its own 6 MB of vendored runtime.** `init --change` calls `establish_assets` against the new document's own directory unless `--assets` is passed. Two changes in one project were measured at 6.0 MB each — the same Mermaid and Vega bytes twice — and `htmlspec assets`, the command that exists to catch exactly this, reports `no asset store under .` because it looks only at the root the user named and the stores are two directories down. The duplicate-detection requirement is satisfied by a command that cannot see the duplicates the tool itself just created.

None of this is a defect in the pieces. `validate`, `format`, and the four templates work. What is missing is the layer above them: an operation whose subject is a *project*, which is the layer OpenSpec's `init` occupies and the layer where a project's asset store, directory shape, and agent instructions are decided together.

There is a second reason to build this now. The `.claude/skills/` directory in this repository holds eleven `openspec-*` skills and one hand-written `htmlspec-archive-change`. The htmlspec workflow is currently delivered to agents by a file someone wrote by hand and copied. Until `htmlspec init` installs those skills, every htmlspec project is bootstrapped by imitation, and the archival instructions — which the artifact-kinds change made load-bearing, since archival is agent-driven by design — have no distribution mechanism at all.

## What Changes

- **`init` becomes the project bootstrap; `new` creates documents.** `htmlspec init` takes no required path and sets up the project. Document and change creation moves to `htmlspec new`, matching `openspec new change`. This is the verb OpenSpec users already have in their fingers, and the collision is resolved in favor of the operation a new user needs first.

  | Was | Becomes |
  | --- | --- |
  | `htmlspec init doc.html --kind proposal` | `htmlspec new proposal doc.html` |
  | `htmlspec init dir --change` | `htmlspec new change <name>` |
  | *(did not exist)* | `htmlspec init` |

  **BREAKING.** The old spellings are removed rather than aliased. Keeping `init <path>` working would leave one verb meaning both "set up this project" and "write this file," and the ambiguity is the thing this change exists to remove. The project has no external consumers.

- **A project has a declared root, and one asset store in it.** `htmlspec init` creates `htmlspec/` with `specs/`, `changes/`, and `changes/archive/`, and establishes **one** asset store at `htmlspec/assets/`. Every document `new` writes afterward references that store relatively. Two changes then share one 6 MB runtime instead of carrying one each.

- **Commands discover the project root by walking upward.** `new`, `validate`, `format`, `assets`, and `archive` locate the nearest ancestor directory containing `htmlspec/` and operate on it, so they work from any subdirectory. `assets` checks the whole project rather than only the directory named, which is what makes it able to report the duplication described above.

- **`init` detects, offers, and installs agent instructions.** It scans for the tool directories a project already has, presents them in an interactive multi-select with detected tools preselected, and writes a skill per workflow plus a slash command where the tool supports one. Five targets in this change: **Claude Code** (`.claude`), **Codex** (`.codex`), **Cursor** (`.cursor`), **Pi** (`.pi`), and a generic **AGENTS.md** fallback. The tool table is data, designed so a sixth is a table entry rather than a code change.

- **The interactive surface degrades to flags.** `--tools claude,codex`, `--tools all`, and `--tools none` skip the prompt entirely. When standard input is not a terminal, `init` uses detected tools without prompting and never blocks — an `init` that hung waiting for a keypress inside a continuous-integration job would be worse than one that did nothing.

- **`init` is idempotent and safe to re-run.** Re-running it on an established project refreshes skills and commands, leaves `specs/` and `changes/` untouched, and never overwrites a config file that exists. This is how a project picks up a newer version's instructions, and it is the common case rather than the exception.

- **`init` ends by naming the next command.** The closing output states the literal thing to type, spelled for the tools that were selected, because the spelling differs per tool and a generic form would be wrong for most of them.

- **The workflow skills become shipped artifacts.** The archival instructions currently living in a hand-written `.claude/skills/htmlspec-archive-change/SKILL.md` move into the binary as the source of what `init` installs. This repository's own copy then comes from `htmlspec init` like everyone else's.

- **`init` writes `spec.nix`, and archival depends on it.** The file carries the project's name, its version, and a pinned reference to the `htmlspec` binary the project expects. Archival reads the version and refuses unless it is ordered after the last archived one, so the accepted contract acquires a version history rather than a sequence of undated merges.

  The starting version is `0.0.0`, which makes the comparison total. A project always has a version on record, so the first archival is the same operation as the thousandth and there is no absent-previous-version branch — code that would run once per project and be tested once.

  The comparison is an ordering rather than a test for difference, because a difference test accepts a downgrade: archiving at `0.1.0` after `0.2.0` would pass. A version ordered backwards is reported separately from one that has not moved, since one is a mistake and the other is work not yet done.

  The bump is the contributor's or the agent's; the program only compares. This is the division archival already uses — an agent produces the specification and the program validates it — and it keeps `archive` a command that reads and checks rather than one that edits the project it is checking. Deciding whether a change earned a minor or a major bump is judgment. Asking whether `0.1.0` follows `0.0.0` is not.

  Nix rather than YAML, because the binary pin is a flake input with a hash and the project already builds with a flake. The alternative invents a syntax for something Nix states directly, then needs a resolver for it.

- **An asset reference that ascends with no store above it becomes an error.** Every asset rule currently returns early when no store is present, which is correct for the 41 conformance fixtures whose references are symbolic. The consequence is that `htmlspec validate --strict` accepts a change directory copied out of its project with all seven of its asset references dead, reporting zero diagnostics. Measured, not inferred.

  One store per project makes that case reachable, so this change closes it. A reference containing `../` asserts a store sits above the document, and **none** of the 341 asset references in the conformance fixtures ascends while every generated one does. So ascent is the signal: an ascending reference with no store found above it is reported, and a non-ascending one stays silent.

Non-goals, deferred deliberately: the rest of the `/htmlspec:*` command lifecycle (`explore`, `propose`, `apply`, `verify`) as *implemented commands* — this change installs the instruction files and ships only the `archive` skill's content, since archival is the one operation the program already gates; a global user-level configuration; authoring rules and project context in `spec.nix`, which stays to the three fields above; the remaining thirty tools in OpenSpec's table; and the Markdown-to-HTML converter.

## Capabilities

### New Capabilities

- `project-bootstrap`: Defines what an htmlspec project is — a declared root, one asset store, and a directory shape — how commands find it from a subdirectory, and the bootstrap operation that establishes it. Defines agent-instruction installation: tool detection, selection, per-tool delivery, idempotent re-run, and the non-interactive fallback.

### Modified Capabilities

- `toolchain-packaging`: Splits the current single requirement covering `validate`, `format`, and `init`. `init`'s subject becomes the project; document and change creation moves to `new`. Adds the requirement that commands resolve the project root by walking upward, and that `assets` checks a project rather than a directory.

## Impact

- **Every documented invocation of `init` changes.** `README.md`, `AGENTS.md`, the CLI integration tests, and the `htmlspec-archive-change` skill all spell `init <path>`. The rename is mechanical but it is not small, and the tests in `crates/htmlspec-cli/tests/cli.rs` pin the old argument shape directly.

- **One new pinned dependency.** `inquire`, exact-pinned, for the multi-select and confirm prompts; the banner and status lines stay hand-rolled ANSI with no dependency. This adds a subtree (`bitflags`, `dyn-clone`, `unicode-segmentation`, `unicode-width`) the flake must vendor, and the reproducible-build requirement means the pin is exact and the lockfile is committed. Hand-rolling raw-mode terminal handling was considered and rejected: TTY detection, arrow-key decoding, redraw, and terminal restore on signal are four well-understood ways to get a first impression wrong.

- **The program acquires a terminal-interactive mode, which the test strategy must reach.** Every command until now has been a pure function over paths returning an `Outcome`, testable without a process. A prompt is not. The design must keep the decision — which tools, which delivery — separable from the prompt that collects it, or the interactive path becomes the one part of the program with no test.

- **Asset storage per project drops by a factor of the number of changes.** A project with five changes stores 6 MB rather than 30 MB. This also makes `htmlspec assets` meaningful for the first time: with one store per project, a duplicate is a real defect rather than the tool's own default behavior.

- **The `.claude/skills/openspec-*` directory becomes ambiguous.** This repository currently uses OpenSpec to develop htmlspec while also carrying one htmlspec skill. Once `htmlspec init` installs `htmlspec-*` skills here, both sets are present and an agent must not confuse them. Naming keeps them apart, but the overlap is worth stating rather than discovering.

- **Exit-code discipline extends to a new failure class.** A user who declines the cleanup prompt, or interrupts the selection, has not failed an operation — but the process must still exit with something. The existing three outcomes were designed for document violations and operational failures, and cancellation is neither.
