## Context

Two functions write every file with a command surface. `write_skills`
(`crates/htmlspec-cli/src/install.rs:88`) and `write_commands` (`:124`) each build
frontmatter with a `format!` and append `workflow.body` unchanged. `write_region` (`:174`)
writes the `AGENTS.md` region and has no invocation at all.

The frontmatter each currently writes:

| Writer | Fields |
| --- | --- |
| `write_skills` | `name`, `description`, `allowed-tools`, `license`, `metadata.generatedBy` |
| `write_commands` | `description`, `allowed-tools`, `generatedBy` |

Neither writes `argument-hint`. Neither body contains a placeholder. The loss is therefore
in two places at once — the frontmatter that declares an argument exists, and the body that
consumes it — and fixing one without the other produces a file that either declares an
argument it ignores or consumes one no tool told it to expect.

## Goals

- The text after an invocation reaches the agent, for every workflow and every tool with a
  command surface.
- An absent argument reaches the agent as absent, so the "ask when nothing was described"
  behavior survives.
- The obligation is checked against produced files rather than against the table it derives
  from.

## Non-Goals

- Positional arguments, flag parsing, or a second placeholder. See the proposal.
- Any change to a body's existing guidance.

## Decisions

### The hint belongs in `WORKFLOWS`, not in the writers

`Workflow` gains one field:

```rust
/// What a contributor may type after the invocation, as a tool's `argument-hint`.
///
/// A property of the workflow because what it takes differs per workflow: `new` takes a
/// description of the work, `apply` takes the change to work on. One string shared by all
/// seven would describe none of them.
pub argument_hint: &'static str,
```

The alternative was a single constant in `install.rs`, and it is wrong for the reason the
module comment on `tools.rs` already gives about `Delivery`: a property with one value per
row belongs in the row. `project-bootstrap` also requires the workflow set to be data, so
that covering another stage is an entry in the table. A hint hard-coded in the writer would
make a new workflow's hint a change to a function.

Every hint is optional in the tools' sense — `[description]` rather than `<description>` —
because no workflow should fail on a bare invocation. `new` asks what the work is;
`explore` opens with the idea unspecified; `apply` and `verify` operate on the change they
find. Marking one required would make a tool reject an invocation the guidance handles
correctly.

### The placeholder goes at the top of the body, under a heading of its own

Each instruction body gains an opening section:

```markdown
## What you were asked

$ARGUMENTS

Where that is empty, nothing was passed and this workflow's own guidance on an absent
input applies.
```

**Why the top.** The body is a document an agent reads in order, and the input it is acting
on is the first thing it needs. Placed at the bottom, an agent reads seven sections of
procedure before learning what it was asked to do.

**Why a heading rather than an inline substitution.** The alternative was weaving the token
into a sentence — "Start a change for $ARGUMENTS" — which reads well when an argument is
present and badly when it is not: "Start a change for ." A heading with the substitution on
its own line degrades to an empty section, which is legible either way.

**Why the sentence about emptiness.** Task 4.5 of the archived change and this change's
third Non-Goal both require a bare invocation to keep asking. An agent meeting an empty
section without that sentence has to infer whether the input was lost or never given, and
the recorded refusal is what an agent does when it suspects the former. Naming it removes
the ambiguity.

### The check asserts against files, following the two checks already there

`install.rs` has a stated convention for this, in the comment on
`every_generated_name_is_loadable`: assert against the files an install produces rather
than against `WORKFLOWS`, "because the value is assembled by a format string at write
time." `generated_files_record_the_generating_version` does the same for `generatedBy`.

An argument placeholder is assembled by the same format strings, so it gets the same
treatment: install for every tool in `TOOLS`, read back every skill and command file, and
assert both that the frontmatter declares a hint and that the body carries the placeholder.
A test over `WORKFLOWS` would pass while a writer that forgot to emit the field shipped.

The `AGENTS.md` region is excluded, and by construction rather than by a special case:
`Installed.regions` is a separate field from `skills` and `commands`, so a test that
iterates the latter two never reaches it.

### Risk: a tool that does not substitute

**Accepted, with the reasoning stated because it is the one thing this change cannot
verify from inside the program.**

Substitution happens in the tool, not here. Claude Code and Codex are confirmed — Claude
Code's own example plugin uses `$ARGUMENTS` in both a command and a skill and documents the
two as "loaded identically", and a Codex plugin command branches on its contents. Cursor
and Pi are not confirmed by an artifact on this machine.

The failure mode if one does not substitute is that a literal `$ARGUMENTS` appears in the
body an agent reads. That is visible rather than silent — an agent seeing the token
unexpanded knows something was meant to be there — which is strictly better than the
current behavior, where the text vanishes and the agent cannot tell it was ever sent. So
the risk is worth taking on an unconfirmed tool: the bad outcome is a legible artifact
rather than a lost input.

`$ARGUMENTS` is also the only spelling worth trying. It is what both confirmed tools use,
and a per-tool placeholder would put a tool-specific string into a body the contract
requires to be tool-independent — the `instructions.rs` module comment states that a skill
and a prompt "differ in their header and not in what they say to do."

### Why this repository's own installed files are regenerated in the same change

`0.2.0` and `0.3.0` both regenerated all 35 files, and `0.3.0` did it to fix exactly the
fault of this repository dogfooding text its own source had removed. The same applies here:
this repository's `.claude/commands/htmlspec/new.md` is what an agent working on html-spec
itself reads, so leaving it without the placeholder would leave the project unable to use
the fix it shipped.

## Risks

- **A tool that does not substitute `$ARGUMENTS`.** Accepted above; the failure is visible
  rather than silent.
- **An argument long enough to crowd the body.** Not mitigated. A description is prose a
  contributor typed on one line, and the bodies are already thousands of words, so a
  proportional risk does not exist.
