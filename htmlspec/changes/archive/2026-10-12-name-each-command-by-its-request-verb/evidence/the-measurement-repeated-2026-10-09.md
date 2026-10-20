# The measurement repeated, and what each operation writes measured

**Date:** 2026-10-09
**Tasks:** 1.1 and 1.2
**Tree:** branch `name-each-command-by-its-request-verb` at `219d1b6` ("Fill only the references a
document loads", `0.28.1`), before any code change. Every `htmlspec` below is this worktree's
`./target/debug/htmlspec`, which prints `htmlspec 0.28.1 (219d1b6f1bc7)`. `openspec` is the one on
this machine's `PATH`, OpenSpec 1.13.2.

## Summary

| Question | 2026-10-05, at `e112067` | Now, at `219d1b6` |
| --- | --- | --- |
| Subcommands, and operations once `new`, `store`, and `migrate` are split | 14 and 18 | **15 and 19**: `show` was added by `show-what-a-modification-changed` |
| Is any OpenSpec command named `get`, `post`, `put`, `patch`, or `delete`? | No | No. OpenSpec 1.13.2 answers `error: unknown command` to each |
| Occurrences of `htmlspec <command>` in tracked files | 1,075 | 1,189 for the same 14 commands, and 1,253 with `show` |
| Aliases in `main.rs` | none | none |
| Does any operation spelled `get` write, in any invocation tried? | read from help: no | **measured: no**, in 25 invocations of the 10 `get` operations |

Neither stopping condition of task 1.1 holds. The one new operation, `show`, was expected by the
design and is given `get show` at task 1.0. No OpenSpec command takes one of the five words.

## Section 1 repeated: the operations

`htmlspec --help`, at `219d1b6`:

```
  validate      Validate documents against the contract
  format        Rewrite documents into their canonical form
  assets        Check a project's asset topology
  archive       Archive a completed change
  init          Set up an HTMLSpec project in this repository
  list          List what the project holds
  run           Start a run of a runbook, or check what its runs recorded
  preflight     Report whether a runbook can start in this working environment
  status        Report which of a change's artifacts exist and how much of its work is done
  sync          Report whether a change's contract reached the accepted specifications
  show          Show each requirement a change adds or modifies beside the accepted one
  store         Register and manage standalone projects on this machine
  instructions  Print the workflow instructions for a stage
  new           Create a document or a change that validates immediately
  migrate       Bring an existing OpenSpec project into HTMLSpec
  help          Print this message or the help of the given subcommand(s)
```

**One difference: `show`**, between `sync` and `store`. Its help says it "writes nothing", and its
one view is `--diff`; `show <change>` without it is refused with exit 2. The nested subcommands are
unchanged: `new` (`change`, `document`), `store` (`add`, `list`, `remove`), and `migrate`
(`survey`, `promote`).

## Section 3 repeated: two names, two effects

Unchanged in substance. OpenSpec's `archive` is still "Archive a completed change and update main
specs". The installed `openspec-sync-specs` skill still describes itself as "Sync delta specs from a
change to main specs", and still calls the operation agent-driven. `htmlspec sync` and `htmlspec
archive` still write no accepted specification.

OpenSpec on this machine is now **1.13.2**, where the measurement read 1.13.1. Its command list holds
the 19 commands section 3 names and three more: `templates`, `schemas`, and `new`. None is one of the
five verbs, and each of `openspec get`, `post`, `put`, `patch`, and `delete` prints
`error: unknown command '<verb>'`.

## Section 5 repeated: where today's spellings are used

The same method: `git grep -o -E 'htmlspec (<command>|…)'` at a revision, each match classified by
its path. Run against `e112067` first, it reproduces every number section 5 gives, 1,075 in total.

| Where | 2026-10-05, 14 commands | Now, 14 commands | Now, with `show` |
| --- | --- | --- | --- |
| archived changes | 549 | 671 | 710 |
| installed instruction copies (`.claude/`, `.codex/`, `.pi/`) | 250 | 280 | 290 |
| crate sources, outside tests | 83 | 83 | 89 |
| README and changelogs | 54 | 56 | 58 |
| instruction sources (`crates/htmlspec-assets/instructions/`) | 50 | 56 | 58 |
| tests | 20 | 21 | 22 |
| active changes | 57 | 10 | 10 |
| accepted specifications | 4 | 4 | 8 |
| scripts, CI, flake | 6 | 6 | 6 |
| runbooks | 2 | 2 | 2 |
| **total** | **1,075** | **1,189** | **1,253** |

By command, now: `validate` 245, `init` 201, `new` 158, `archive` 126, `migrate` 85, `sync` 84,
`list` 70, `show` 64, `format` 64, `assets` 51, `status` 36, `run` 25, `preflight` 24,
`instructions` 16, `store` 4.

The growth is the six changes archived since, and their records. The instruction lines that invoke a
command are now **348** (290 installed and 58 in the sources), where the proposal counted 300. The
proposal's argument stands with the larger number: none of them changes under this change.

## Section 6 repeated: aliases

`grep -n 'alia[s]' crates/htmlspec-cli/src/main.rs` prints nothing. The one match under
`crates/htmlspec-cli/src/` is a doc comment in `preflight/host.rs` about a shell alias. Each
operation still has exactly one spelling.

## Section 2, measured: what each operation writes

Task 1.2 asks for this to be measured rather than read from help. A script in the session scratchpad
built a fresh tree for every case with `./target/debug/htmlspec`, committed it to a Git repository of
its own, ran one invocation, and recorded `git status --porcelain --untracked-files=all` and any
directory that appeared or vanished. Each tree held:

- `project/`, made by `init --tools none` and `new change demo`, with one accepted specification
  (`htmlspec/specs/surface/spec.html`), a change specification modifying its one requirement by one
  word, a runbook `release` with a `VERSION` and a `CHANGELOG.md`, a second copy of the accepted
  specification with a blank line `format` removes, and the declared version advanced to `0.7.0`;
- `config/`, the `XDG_CONFIG_HOME` and `HOME` of every run, holding one registration, `demo-store`;
- `source/`, a two-file OpenSpec project for `migrate survey`;
- `migration/`, a staged project, a plan covering both surveyed entities, its survey payload, and
  an empty destination, for `migrate promote`.

42 invocations ran. **31 changed nothing**, and those are every invocation of a `get` operation and
every invocation with an option that withholds writes.

| Operation | Verb | Invocations measured | What the tree shows |
| --- | --- | --- | --- |
| `validate` | `get` | bare, `--strict`, `--json`, one path | nothing, in all four |
| `list` | `get` | bare, `--specs`, `--runbooks`, `--json` | nothing |
| `status` | `get` | `--change demo`, with and without `--json` | nothing |
| `sync` | `get` | `--change demo`, with and without `--json` | nothing |
| `show` | `get` | `demo --diff`, with and without `--json`; `demo` alone, refused with exit 2 | nothing |
| `assets` | `get` | bare, `--json` | nothing |
| `preflight` | `get` | `release`, with and without `--json` | nothing |
| `instructions` | `get` | `apply`, with and without `--json` | nothing |
| `store list` | `get` | bare, `--json` | nothing |
| `migrate survey` | `get` | bare, `--json`, in `source/` | nothing |
| `new change` | `post` | `another --subject …` | 3 documents created in a new `changes/another/` |
| `new document` | `post` | `proposal htmlspec/changes/demo/notes.html` | 1 document created |
| `store add` | `post` | `second ../project` | `config/htmlspec/stores.json` modified, outside the project |
| `run` | `post` | `release --as first` | `runbooks/release/runs/first.html` created, in a new `runs/` |
| | | `release --check` | nothing |
| `migrate promote` | `post` | the plan and survey, no `--staged` | nothing |
| | | with `--staged staging --into repository --check` | nothing |
| | | with `--staged staging --into repository` | 12 files moved from `staging/` to `repository/` |
| `init` | `put` | `--tools none` on the project | nothing |
| | | `--tools claude` on the project | 18 instruction files created under `.claude/` |
| | | `--tools none --name fresh` in an empty directory | a project created: `spec.nix`, 8 stored assets, and `htmlspec/` with its `specs/`, `changes/archive/`, and `runbooks/` |
| `format` | `put` | bare | the one non-canonical document rewritten |
| | | `--check` | nothing, exit 1 |
| `archive` | `patch` | `demo --yes` | the change's 4 documents moved into `changes/archive/2026-10-09-demo/`, with `VERSION` and `runbooks.json` written there, and `htmlspec/runbook-releases.json` written at the planning root |
| | | `demo --check` | nothing |
| | | `demo --check`, over an archival interrupted before its move | the journal and the 2 files it staged removed: "rolled back an interrupted archival" |
| `store remove` | `delete` | `demo-store` | `config/htmlspec/stores.json` modified |

**The `put` claim, measured as well.** `init --tools claude,codex,pi` and `format`, each run twice
with a commit between, changed nothing on the second run. That is what design decision 2 means by
"leaves it as one run did when run again".

### Where the measurement differs from section 2

- **`archive` writes more than its `VERSION`.** In a project holding a runbook, it also writes the
  inventory of the runbooks it publishes, `runbooks.json`, into the archived change, and on the
  project's first such publication the adoption marker `htmlspec/runbook-releases.json` at the
  planning root. Both are archival's bookkeeping, and neither is a document a reader asks for. The
  verb does not move: `archive` still changes the standing of a change that exists, and design
  decision 2 now names the two files.
- **`store add` and `store remove` write outside the project**, in the machine's registry under
  `XDG_CONFIG_HOME`. Section 2 says "adds a registration" and "removes a registration", which is
  right, and the measurement adds where the registration lives. Because the base already held one
  registration, `store add` shows as a modification of `stores.json`; on a machine with none it
  creates the file.

Nothing else differs. In particular, no operation spelled `get` wrote in any of its 25 invocations,
and each of the four operations with a withholding option wrote nothing under it: `format --check`,
`archive --check` on a project with no interrupted archival, `run --check`, and `migrate promote`
without `--staged` and with `--check`.
