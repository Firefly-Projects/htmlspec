## Context

`htmlspec init` installs instructions for five tools by iterating a table of workflows. The table has one row. Everything downstream of it — skill writing, command writing, the marked region in a shared file — already loops, so the mechanical part of this change is adding rows.

The load-bearing part is not mechanical. Six instruction bodies have to be written, and an instruction body is the project's opinion about how a stage of the lifecycle is performed. `archive.md` is 196 lines and most of its length is reasoning about why the operation is arranged the way it is. Getting six more of those right is the work; the loop count is incidental.

Two constraints come from the existing design and neither is negotiable here:

1. **Bodies are compiled in.** `include_str!` into the binary, because the packaged program runs with no source checkout. Six new files under `crates/htmlspec-assets/instructions/`.
2. **A body must not name a command the program lacks.** Stated as a comment and enforced by one narrow test that greps for `htmlspec init ` with a trailing space.

## Goals

- Seven workflows installed, covering the lifecycle from an idea to an archived change.
- No instruction body can name a nonexistent command, enforced mechanically rather than by review.
- Adding an eighth workflow stays a table entry plus a body file.

## Non-Goals

- New CLI surface. See the proposal.
- Changing how a tool receives instructions. `Delivery` is untouched.

## Decisions

### The rule becomes a test over the CLI's own command tree

The current guard is one test asserting no body contains the literal `htmlspec init `. It was written for a specific regression — `init` took a path until the last release — and it generalizes badly: it would not catch `htmlspec propose`, `htmlspec sync`, or `htmlspec new spec` (the kind is `specification`).

Replace it with a test that extracts every `htmlspec <words>` occurrence from every body and resolves each against clap's command tree, which `htmlspec-cli` can hand over as data. A body naming `htmlspec verify` fails the build.

This inverts where the constraint lives. Today a human reads the comment in `instructions.rs` and is careful. After this, the compiler and the test suite are careful, and the comment describes a checked property rather than a convention. That inversion is the reason six bodies can be added in one change without the review burden scaling with them.

The test lives in `htmlspec-cli`, not `htmlspec-assets`, because that is where the clap definition is and assets must not depend on the CLI. `htmlspec-assets` keeps the two properties that are local to it — a body has length, and a body carries no frontmatter of its own.

**Extraction is deliberately crude.** A regex for `htmlspec` followed by words, then walk the words down the subcommand tree until one is not a subcommand, and treat the prefix as the invocation. A body is prose with fenced code blocks, and a parser sophisticated enough to know which occurrences are invocations would be a parser with its own bugs. Crude and over-eager is the right failure direction: it flags a false positive, someone rewords a sentence, and no false negative reaches a contributor.

### Workflow order is data, and it is lifecycle order

`WORKFLOWS` becomes seven entries and their order is the order a change moves through them: `explore`, `new`, `continue`, `ff`, `apply`, `verify`, `archive`. Nothing currently depends on the order — with one row there was no order to depend on — and two things will: the marked region in `AGENTS.md` writes one section per workflow in table order, and the `--json` payload lists them.

Alphabetical order would put `apply` first and `verify` last, which reads as a list of unrelated commands. Lifecycle order makes the region a document a contributor can read top to bottom, so the ordering is stated as a property of the table with a test that pins the first and last entries.

### `ff` is a driver over the other bodies, not a fourth copy of them

`ff` produces every artifact in one pass. The tempting implementation is a body that restates what `new` and `continue` say. That is three copies of the proposal guidance, and the one that drifts is the one someone reads.

Instead `ff` states the sequence, the checkpoints, and what makes stopping early correct, and defers to the other workflows by name for what each artifact carries. The skill files land in one directory per tool, so a body can name `htmlspec-new-change` and the agent has it.

The cost is honest: an agent running `ff` needs two files rather than one. The alternative cost is three descriptions of a proposal, which the artifact-kinds change exists to have exactly one of.

### `explore` ships despite having no command at all

`explore` names no `htmlspec` invocation. It is a stance — think, read, investigate, do not implement — and OpenSpec's version says so explicitly in its second paragraph.

A workflow with no command is worth installing because the failure it prevents is real and expensive: an agent asked to think about a problem starts writing code, and the change that results has no proposal because the work was done before anyone decided what it was for. The instruction that stops that is worth more than instructions for a stage that has a command to structure it.

It also settles the rule the proposal argues about, in the strongest available form. If a workflow with no command is legitimate, then "installing an instruction file for a command that does nothing" was never the right way to state the hazard. The hazard is naming a command that does not exist, and `explore` names none.

### The closing message names the entry point

`report_next_command` hardcodes `tool.invocation("archive")`. With seven workflows the useful thing to name on a first run is the entry point, so the workflow named becomes a property of the table — one entry flagged as the lifecycle's start — rather than a string literal at the call site.

A flag on the table rather than "the first entry" because those are different facts that happen to coincide today. Order is for reading; the entry point is what a contributor types first. A later change that adds `onboard` at the top of the region would silently repoint the closing message if the two were the same fact.

The message keeps its per-spelling grouping and the archival line moves into a second sentence, since a contributor who has just bootstrapped is not archiving anything yet.

### Bodies are written against the accepted contract

Where the accepted specifications already state what an artifact kind carries, the body says the same thing in the same terms, and names the diagnostic that reports a violation. `archive.md` does this well — it names `kind/task-in-specification` and explains why the rule exists — and the six new bodies follow it.

This is a writing constraint rather than a mechanism, and there is no test for it. Checking that a named diagnostic exists is a smaller version of the command-name test and is left out of scope: identifiers are stable and the six bodies name few of them.

There is a subtler hazard here than a misspelled rule id, and the bodies have to be written against it. `kind/*` has exactly two rules — a specification must not carry a task, a tasks document must carry at least one task — and the absence of any rule about a proposal's or a design's content is a stated design commitment, not a gap. So a body must not imply that a proposal is validated for what it carries. An agent told a rule exists writes to satisfy a check that never runs, and the freedom those two kinds have is the thing it would be writing away.

## Risks

- **Six bodies is a large amount of prose to get right at once, and prose has no compiler.** The command-name test catches the mechanical error and nothing catches a body that is merely bad advice. Mitigation: each body is written against the accepted contract for its artifact kind, and the change is exercised end to end by running the lifecycle in a scratch project — the same way the archive skill's content was verified when it moved into the crate.

- **The `AGENTS.md` region grows from one workflow to seven.** The region is the whole instruction set inlined, so a shared-file tool gets a much longer file. Measure it; if the result is unreasonable the region becomes a summary per workflow with the detail in the skill files, which is a change to `write_region` and not to the bodies.

- **`ff` deferring to other bodies by name assumes those files are present.** True for skills-and-commands and skills-only tools. For a `SingleFile` tool every body is in the one region, so the reference resolves to a section of the same file. Both work; the phrasing has to be neutral about which, so it names the workflow rather than a path.

- **The command-name test's crude extraction will flag prose.** A sentence like "the archive command checks what you wrote" is fine, but "run htmlspec on the project" would be read as an invocation of a nonexistent subcommand. Accepted: rewording is cheap, and the alternative direction of failure is a shipped skill that sends an agent to an error.

## Migration

Existing projects re-run `htmlspec init`, which is already documented as safe to re-run and already refreshes instructions. Six new files appear and `archive` is rewritten identically. The generated-version frontmatter makes a project still carrying the one-workflow install identifiable.

No accepted specification is withdrawn. The bootstrap requirements are written in terms of "the instructions that tool needs to drive the HTMLSpec workflow", which is satisfied better by seven than by one; the delta strengthens that requirement to say the lifecycle is covered and that a named command must exist.
