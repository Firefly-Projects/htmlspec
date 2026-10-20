## Context

See proposal.md for why. The shape of the current system that the approach depends on:

- **One subcommand per operation, and no aliases.** `crates/htmlspec-cli/src/main.rs` declares 14
  subcommands with clap's derive API, three of which take subcommands of their own, and declares no
  alias (`evidence/the-measurement-2026-10-05.md`, sections 1 and 6).
- **The spellings are contract.** toolchain-packaging's "The command surface names its operations as
  OpenSpec names them" fixes `validate`, `archive`, `init`, `new`, `list`, `status`, `sync`,
  `instructions`, and `store` because the installed instructions invoke them by name, and fixes
  `format` and `assets` as this project's own.
- **The spellings are everywhere.** 1,075 occurrences in tracked files, 300 of them in the
  instructions and their installed copies (evidence, section 5).
- **What each operation writes is already decided**, by its requirement and its options: nine
  never write, and four of the nine that can have an invocation that writes nothing,
  `format --check`, `archive --check` but for its settling of an interrupted archival, `run
  --check`, and `migrate promote` without `--staged` (evidence, section 2).
- **`show-what-a-modification-changed` adds `show`**, which reads and never writes, to the same
  requirement.

## Goals / Non-Goals

**Goals:**

- Every operation can be invoked by a verb that says what it does to the tree.
- Every current spelling keeps working, byte for byte.
- One place states each operation's two spellings, so they cannot drift apart.

**Non-Goals:**

- Withdrawing a spelling. The contributor decided to keep them.
- Changing what any operation does.
- htmx itself: no script, no server, no request.

## Decisions

### 1. A second spelling, and the OpenSpec one stays primary

Each operation answers to its request-verb spelling and its OpenSpec spelling, and both run the same
handler with the same arguments. The OpenSpec spelling is the one the program prints in its own
messages, the one the installed instructions use, and the one listed first in help.

**Considered: replacing the OpenSpec spellings.** Rejected by the contributor on 2026-10-05. It would
also break every instruction line that invokes a command, every runbook step, and every archived
change's record of what it ran (evidence, section 5), and it would contradict the requirement's own
reason for fixing the spellings.

**Considered: writing the verbs into the instructions.** Rejected. The instructions are read by
agents that arrive from OpenSpec, and the requirement's promise is that they have one spelling to
learn. The verbs are for a reader who wants the effect on the face of the command. Revisiting this
is a change of its own, once the verbs have been used.

### 2. The verb names the most an operation writes, and options carry over

| Verb | Meaning, as HTTP gives it | Operations |
| --- | --- | --- |
| `get` | never writes | `validate`, `list`, `status`, `sync`, `show`, `assets`, `preflight`, `instructions`, `store list`, `migrate survey` |
| `post` | creates something that was not there | `new change`, `new document`, `store add`, `run`, `migrate promote` |
| `put` | creates or replaces a whole thing, and leaves it as one run did when run again | `init`, `format` |
| `patch` | changes something that exists | `archive` |
| `delete` | removes something | `store remove` |

The verb is the most the operation writes in any invocation. So `get` is a promise about every
invocation: an operation spelled with it never writes. Four operations that write have an invocation
that does not: `format --check`, `archive --check`, `run --check`, and `migrate promote` without
`--staged` or with `--check`. Each takes the verb of what it writes when it does, and under the verb
spelling each option means what it means today, so `put format --check` writes nothing.
`archive --check` keeps its one exception under both spellings: before it reports, it settles an
archival interrupted earlier, completing it or rolling it back.

**Considered: the effect of the default invocation.** Rejected after the review of this proposal,
where the first version stood. `migrate promote` with no option checks a plan's coverage and writes
nothing, and moves a project into place only with `--staged`, an option that adds writes. Judged by
its default it would have been `get`, a verb that promises no writes, for an operation that can put a
whole project where there was none.

**Considered: `get` for every `--check` invocation**, so `format --check` would be `get format`.
Rejected. Two spellings of one operation would then differ in which options they accept and in what
an option means, and the spelling would stop being a spelling. An option is the same option under
both.

`init` is `put` and not `post`. It creates the project where there is none and refreshes the
installed instructions where there is one ("Safe to re-run"), and running it twice leaves what running
it once did, which is what HTTP's `PUT` means. `post` is kept for an operation that makes something
that was not there: a change, a document, a registration, a run record, a promoted project. The first
version of this proposal had `post project`, and the contributor chose `put` after the review.
`format` is `put` for the same reason: it replaces each document whole, and a second run changes
nothing.

`archive` is `patch` and not `delete` or `post`: it moves a change that exists into the archive and
writes its `VERSION`, which changes the change's standing, and creates nothing a reader asks for.
Measured at task 1.2, it also writes its bookkeeping where the project holds a runbook: the inventory
`runbooks.json` beside the `VERSION`, and, at the project's first such publication, the adoption
marker `runbook-releases.json` at the planning root. Neither is a document a reader asks for, so the
verb stands.
`run` is `post`, because a run writes a record that was not there. `migrate promote` is `post`,
because with `--staged` it puts a project where there was none. `store remove` is `delete`, and its
object says what it deletes: the registration, never the project it names.

### 3. The objects

The objects are fixed by this table, and the test in task 2.1 asserts that no two operations share a
spelling of either kind.

| Operation | Object | Why |
| --- | --- | --- |
| `validate` | `validity` | the operation's name, as a noun |
| `list`, `status`, `sync`, `show`, `assets`, `preflight`, `instructions`, `run`, `format`, `archive` | the operation's own name | the name already says what it acts on |
| `init` | `project` | what it creates or refreshes |
| `new change`, `new document` | `change`, `document` | what it creates |
| `store add`, `store remove` | `registration` | what it adds or removes, which is never the project it names |
| `store list` | `registrations` | what it lists |
| `migrate survey`, `migrate promote` | `survey`, `promotion` | the subcommand's name, as a noun |

The table follows a pattern, but the pattern is how the objects were chosen and not a rule a program
applies. The first version of this decision stated it as a rule, and the review of this proposal
showed that the rule did not produce the names it listed: applied as written, it gave `init` and
`migrate promote` the same object, and `list` and `store list` the same spelling.

The contributor chose nine in their request: `get validity`, `get list`, `get status`, `get sync`,
`post change`, `post project`, `put format`, `patch archive`, and `delete store`. After the review,
on 2026-10-06, they changed two of them: `init` is `put project`, and the three `store` operations
take `registration`. `show`, which `show-what-a-modification-changed` added and which never writes,
became `get show` at task 1.0.

**Considered: `hx-get` and its siblings verbatim.** Rejected by the contributor's choice of the
verb-and-object form. The prefix says "htmx attribute", which is not what a subcommand is.

### 4. One closed type states both spellings

```rust
/// Every operation the command line offers.
pub enum Operation { Validate, Format, Assets, Archive, Init, List, Run, Preflight, Status, Sync,
    Show, StoreAdd, StoreList, StoreRemove, Instructions, NewChange, NewDocument, MigrateSurvey,
    MigratePromote }

pub enum Verb { Get, Post, Put, Patch, Delete }

impl Operation {
    /// The OpenSpec spelling, as words: `["store", "add"]`.
    pub const fn spelling(self) -> &'static [&'static str];
    /// The request-verb spelling: the verb and its object.
    pub const fn request(self) -> (Verb, &'static str);
}
```

Both functions are exhaustive matches, so an operation added later does not compile until it has
both spellings. A unit test checks that no two operations share a spelling of either kind. In
`main.rs`, each verb is a subcommand whose objects are an enum carrying the same `Args` struct the
OpenSpec subcommand carries, and both reach the handler through `Operation`. Clap's `alias` cannot
do this, because an alias is one word and `get validity` is two.

**Considered: rewriting the arguments before clap parses them.** Rejected. Help and error messages
would then speak of a command the contributor did not type.

### 5. Help

`htmlspec --help` lists the operations as today, then the five verbs under their own heading.
`htmlspec get --help` lists each object with the OpenSpec spelling it runs. An object's own help is
its operation's help, so the options are documented once.

## Risks / Trade-offs

- **[Two spellings double what an agent can call.]** → The instructions name one, and help says the
  two are one operation. The parity tests pin that they are.
- **[The verbs suggest a server.]** → Help says the verbs name the effect and send nothing.
- **[A later OpenSpec operation takes one of the five words.]** → The requirement fixes OpenSpec's
  spelling for an operation OpenSpec provides, so the OpenSpec meaning would win and this change's
  verb would move. None of the five is an OpenSpec command today (evidence, section 3).
- **[An operation's effect changes later, so its verb is wrong.]** → The `get` test runs every `get`
  operation in a project and checks the tree is unchanged, so an operation that starts writing fails
  it.
- **[`show-what-a-modification-changed` lands with a different shape.]** → Task 1.0 reads it as built.

## Migration Plan

**The document contract does not move.** No unit, field, attribute, or projection rule changes.
**The release is a minor version**, the next after whatever precedes it in landing order, because the
command line gains operations' spellings and loses none. The contributor confirms the increment at
the release task. Rollback is a revert, and nothing written with the OpenSpec spellings depends on it.
