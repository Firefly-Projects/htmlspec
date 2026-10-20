## Why

**A command's name says what it is about, and nothing says whether it changes the tree.**
`evidence/the-measurement-2026-10-05.md` splits the command line into its 18 operations and reads
what each one writes. Nine never write. Of the nine that can, five create something that was not
there, two create or replace a whole thing, one changes something that exists, and one removes
something (section 2). None of
that is in a name: `status` and `format` read alike, and only the help text says that one of them
rewrites every document it is given.

**Two names already mean a different effect here than where they came from.** toolchain-packaging's
"The command surface names its operations as OpenSpec names them" fixes the spellings so that a
contributor arriving from OpenSpec meets the names they know. OpenSpec's sync and archive both write
the accepted specifications. `htmlspec sync` only reports, and toolchain-packaging says it "SHALL
NOT modify an accepted specification"; `htmlspec archive` "assesses the merge and never performs
it" (section 3). The shared spelling carries the contributor's expectation across with it, and the
expectation is wrong for both.

**htmx already gives HTML five words for exactly this.** Its request keywords, `hx-get`, `hx-post`,
`hx-put`, `hx-patch`, and `hx-delete`, each issue the HTTP request of that verb, and HTTP fixes what
each verb does: `GET` changes nothing, `POST` creates, `PUT` replaces a whole thing, `PATCH`
changes part of one, and `DELETE` removes it (section 4). An HTML project's command line can borrow
the same words, so that `get sync` says on its face that it writes nothing.

**The contributor asked for it.** On 2026-10-05 the contributor asked for the command line to move
from its command names to htmx's keywords, chose the verb-and-object form, and chose to keep every
current spelling working. On 2026-10-06, after the review of this proposal, they chose `put project`
for `init` and `registration` as the object of the three `store` operations.

**Why now.** It is a proposal now and is implemented after the ten changes queued for the
self-migration. It modifies the same requirement as `show-what-a-modification-changed`, which adds
`show`, so it lands after that change and gives `show` its verb.

## What Changes

- **Every operation gains a request-verb spelling**: one of `get`, `post`, `put`, `patch`, or
  `delete`, chosen by what the operation does to the files it acts on, followed by its object.
  - `get` (never writes): `get validity` (`validate`), `get list`, `get status`, `get sync`,
    `get show`, `get assets`, `get preflight`, `get instructions`, `get registrations`
    (`store list`), and `get survey` (`migrate survey`).
  - `post` (creates something new): `post change` (`new change`), `post document` (`new document`),
    `post registration` (`store add`), `post run` (`run`), and `post promotion` (`migrate promote`).
  - `put` (creates or replaces a whole thing, and is safe to repeat): `put project` (`init`) and
    `put format`.
  - `patch` (changes what exists): `patch archive`.
  - `delete` (removes): `delete registration` (`store remove`), which removes the registration and
    never the project it names.
- **The verb names the most an operation writes, and options carry over unchanged.** `get` spells
  only an operation that writes nothing in any invocation. An operation that writes only sometimes
  takes the verb of its writes: `migrate promote` writes only with `--staged`, and is `post
  promotion`. `put format --check` writes nothing, as `format --check` does. `patch archive
  --check` writes nothing either, except that it first settles an archival interrupted earlier, as
  `archive --check` does. The verb names the operation, not one invocation of it.
- **Both spellings run one operation**, with the same arguments, options, output, `--json` payload,
  and exit status.
- **The OpenSpec spellings stay, and stay primary.** The program's messages and the workflow
  instructions it installs keep naming them, so nobody has to learn the verbs and none of the 300
  instruction lines that invoke a command changes (section 5).
- **Help lists both.** `htmlspec --help` names the five verbs beside the operations, and
  `htmlspec get --help` lists each object with the operation it runs.
- **Unchanged:** the document contract, whose version does not move; every operation's behavior,
  options, payload, and exit codes; the OpenSpec spellings and selectors; the installed instructions.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `toolchain-packaging`: **modified: "The command surface names its operations as OpenSpec names
  them".** Two paragraphs are added after the paragraph that fixes the OpenSpec spellings: one fixes
  the request-verb spellings and what each verb means, and one makes them a second spelling of the
  same operation, never a replacement, with the OpenSpec spellings still the ones the program and its
  instructions name. Five scenarios are added. Every other paragraph and scenario is the accepted
  text, unchanged. The heading stays, because the OpenSpec spellings are still the fixed ones.
  `show-what-a-modification-changed` modifies this requirement first, adding `show` and `--diff`, so
  task 1.0 refreshes this block from the accepted text once that change archives and gives `show` a
  verb.

## Impact

- `crates/htmlspec-cli/src/main.rs`: five verb subcommands, `get`, `post`, `put`, `patch`, and
  `delete`, each taking an object that carries the same arguments as the operation it runs and
  dispatching to the same handler.
- `crates/htmlspec-cli/src/operation.rs`, new: one closed `Operation` type naming every operation,
  with its OpenSpec spelling, its verb, and its object, so an operation added later cannot compile
  without both spellings.
- Tests: the operation table, CLI parity between the two spellings of every operation, a check that
  every `get` operation leaves a project byte for byte unchanged, and the help.
- `README.md`: the command list gains the request-verb spellings.
- **Not in this change**, and recorded so it is not assumed to follow:
  - Withdrawing or renaming any OpenSpec spelling. The contributor chose to keep them.
  - Writing the verbs into the installed instructions, `htmlspec`'s own messages, or the runbooks.
  - htmx as a dependency, a server, or any request. The verbs are words on the command line.
  - Verbs for options. `--check` and `--json` stay as they are under both spellings.
  - A verb for clap's `help`.
