# The command surface, what each command writes, and where its spellings are used

**Date:** 2026-10-05
**Tree:** `main` at `e112067`. Every `htmlspec` below is this repository's `./target/debug/htmlspec`,
`htmlspec 0.22.0`. `openspec` is OpenSpec 1.13.1.

## Summary

| Question | Answer |
| --- | --- |
| How many operations does the command line offer? | 14 subcommands, 18 operations once `new`, `store`, and `migrate` are split into theirs (section 1). |
| Does a command's name say whether it writes? | No. 9 of the 18 never write, and the names do not say which (section 2). |
| Does a shared name mean a shared effect? | Not for two of them. OpenSpec's sync and archive write the accepted specifications. `htmlspec sync` and `htmlspec archive` never do (section 3). |
| What are htmx's request keywords? | `hx-get`, `hx-post`, `hx-put`, `hx-patch`, and `hx-delete`, each issuing the HTTP request of that verb (section 4). |
| How widely are today's spellings used? | 1,075 occurrences of `htmlspec <command>` in tracked files: 250 in installed instruction copies, 50 in their sources, 549 in archived changes (section 5). |
| Does the command line accept a second spelling today? | No. `main.rs` declares no alias (section 6). |

## 1. The operations

`htmlspec --help`:

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
  store         Register and manage standalone projects on this machine
  instructions  Print the workflow instructions for a stage
  new           Create a document or a change that validates immediately
  migrate       Bring an existing OpenSpec project into HTMLSpec
```

Three of them take a subcommand: `new` (`change`, `document`), `store` (`add`, `list`, `remove`), and
`migrate` (`survey`, `promote`). `help` is clap's own.

## 2. What each operation writes

Read from each operation's `--help`. `run` requires `--as` for a record's name and refuses without
it, so its invocations are with `--as` or with `--check`. The review of this proposal corrected the
first version of this table, which called `migrate promote`'s write its default when it writes only
with `--staged`, and missed `archive --check`'s settling of an interrupted archival.

| Operation | The most it writes | Writes nothing when |
| --- | --- | --- |
| `validate` | nothing | always |
| `format` | rewrites each document whole | `--check` |
| `assets` | nothing ("Reports each set of locations holding identical asset bytes") | always |
| `archive` | moves a change into the archive and writes its `VERSION` | `--check`, except that it first settles an archival interrupted earlier, completing it or rolling it back |
| `init` | establishes a project, or, on one that exists, refreshes its instructions ("Safe to re-run") | never |
| `list` | nothing | always |
| `run` | starts a run, writing its record | `--check` ("write nothing") |
| `preflight` | nothing | always |
| `status` | nothing | always |
| `sync` | nothing | always |
| `store add` | adds a registration | never |
| `store list` | nothing | always |
| `store remove` | removes a registration ("The project it named is untouched") | never |
| `instructions` | nothing | always |
| `new change` | creates a change directory and its documents | never |
| `new document` | creates one document | never |
| `migrate survey` | nothing ("without writing anything") | always |
| `migrate promote` | with `--staged`, moves the staged project into place | without `--staged`, or with `--check` |

Nine operations never write: `validate`, `assets`, `list`, `preflight`, `status`, `sync`, `store
list`, `instructions`, and `migrate survey`. Of the nine that can write, five create something that
was not there (`new change`, `new document`, `store add`, `run`, and `migrate promote`), two create or
replace a whole thing and leave it as one run did when run again (`init` and `format`), one changes the
standing of something that exists (`archive`), and one removes something (`store remove`). Four of
them have an invocation that writes nothing: `format --check`, `archive --check` but for its
settling, `run --check`, and `migrate promote` without `--staged` or with `--check`. The names carry
none of this: `sync` and `archive` read like operations that change the contract, and `format` and
`status` read alike.

## 3. Two names, two effects

OpenSpec's command list (`openspec --help`) has no `sync`. Its sync is an agent workflow, the
`openspec-sync-specs` skill this repository installs, whose description is "Sync delta specs from a
change to main specs" and which says: "This is an **agent-driven** operation - you will read delta
specs and directly edit main specs to apply the changes." OpenSpec's `archive` is "Archive a completed
change and update main specs".

`htmlspec sync` is "Report whether a change's contract reached the accepted specifications", and
toolchain-packaging's "The command surface reports whether a change reached the accepted contract"
says it "SHALL NOT modify an accepted specification". `htmlspec archive` never updates them either:
"Archival assesses the merge and never performs it".

So a contributor who arrives from OpenSpec, as the requirement that fixes these spellings expects,
reads `sync` and `archive` as operations that write the accepted contract, and in this program
neither does. The spelling is shared and the effect is not.

OpenSpec 1.13.1's commands are `init`, `update`, `list`, `view`, `change`, `archive`, `spec`,
`config`, `schema`, `store`, `doctor`, `context`, `workset`, `validate`, `show`, `feedback`,
`completion`, `status`, and `instructions`. None of them is `get`, `post`, `put`, `patch`, or
`delete`.

## 4. htmx's request keywords

htmx 2.x (`https://htmx.org/docs/` says "the latest version of htmx is 2.x"). The reference,
`https://htmx.org/reference/`, read 2026-10-05:

- `hx-get`: "issues a `GET` to the specified URL"
- `hx-post`: "issues a `POST` to the specified URL"
- `hx-put`: "issues a `PUT` to the specified URL"
- `hx-patch`: "issues a `PATCH` to the specified URL"
- `hx-delete`: "issues a `DELETE` to the specified URL"

`https://htmx.org/docs/#ajax`: "The core of htmx is a set of attributes that allow you to issue AJAX
requests directly from HTML", and "Each of these attributes takes a URL to issue an AJAX request to."

Five keywords, one for each HTTP verb, and HTTP gives each a fixed meaning about effect: `GET`
retrieves and changes nothing, `POST` creates, `PUT` replaces the whole of a thing, `PATCH` changes
part of it, and `DELETE` removes it.

## 5. Where today's spellings are used

Every occurrence of `htmlspec <command>`, for the 14 commands, in `git ls-files`:

| Where | Occurrences |
| --- | --- |
| archived changes | 549 |
| installed instruction copies (`.claude/`, `.codex/`, `.pi/`) | 250 |
| crate sources, outside tests | 83 |
| active changes | 57 |
| README and changelogs | 54 |
| instruction sources (`crates/htmlspec-assets/instructions/`) | 50 |
| tests | 20 |
| scripts, CI, flake | 6 |
| accepted specifications | 4 |
| runbooks | 2 |
| **total** | **1,075** |

By command: `validate` 222, `init` 186, `new` 150, `archive` 123, `migrate` 81, `sync` 81, `list` 58,
`format` 51, `assets` 50, `run` 25, `preflight` 17, `status` 15, `instructions` 12, `store` 4.

The installed instructions invoke the commands by name, which is the reason toolchain-packaging gives
for fixing the spellings. Withdrawing a spelling would break every one of those 300 lines and every
archived change's record of what it ran.

## 6. Aliases today

`grep -n alias crates/htmlspec-cli/src/main.rs` prints nothing. Each operation has exactly one
spelling. The one precedent for a spelling that changed is the bootstrap subcommand, whose former
arguments are refused with a message naming the subcommand that now does that work (the scenario "A
withdrawn spelling names its replacement").
