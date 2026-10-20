## Why

`htmlspec archive` is named for an operation it does not perform. It checks a change's
specification, reports that archival *may* proceed, and exits. The change is still sitting in
`changes/`. The accepted specifications still say what they said before. Four steps remain —
merge the contract, move the directory, name it, record the version — and the program has no
opinion about whether any of them happened. It reported success, because it succeeded at the
only thing it does.

The workflow instructions carry those four steps as prose. Prose is not a gate. An agent that
runs `htmlspec archive`, sees a zero exit, and stops has done exactly what the command told it,
and the project is left with a change that is archived in the report and active on disk.

**The gate that does exist has never fired.** `last_archived_version`
(`crates/htmlspec-cli/src/commands.rs:1352`) finds the previous version by scanning archive
*directory names* for a part that parses as a version. This project records the version
somewhere else: a `VERSION` file inside each archived change, which is what
`toolchain-packaging`'s release-hygiene requirement asks for and what all 11 archived changes
carry. No directory name in `changes/archive/` holds a version. So the scan finds nothing, falls
back to `0.0.0`, and every declared version above that passes. The one mechanical check archival
enforces has been inert since it was written, and the reason is that the program reads a
convention that nothing writes — because nothing in the program writes the archive at all.

**Who wants this and what they cannot do.** Every plausible adopter arrives from OpenSpec; the
README says HTMLSpec is a spinoff in its second paragraph. There, `archive` finishes the job. A
contributor who learns the command here finds a checker wearing the name of an operation, and
discovers the difference by finding an unmoved directory some days later.

**The commands the workflow already assumes.** The OpenSpec workflow instructions vendored in
this repository make 170 command invocations across 8 commands. Ranked: `status` 52,
`instructions` 48, `store` 22, `new` 14, `list` 14, `archive` 10, `schemas` 6, `context` 4. Four
of the top five have no HTMLSpec counterpart. `status` and `instructions` alone account for 100
of the 170. The HTMLSpec prompts do not paper over the gap — they invoke `validate`, `archive`,
`new`, and `format`, and carry no change-selection step at all, because there is no command that
lists changes.

**What working looks like the first time you see it.** A contributor finishes a change and types
`htmlspec archive`. With no argument it lists the active changes and asks which. It checks the
version, checks the specification, and reports that two tasks are still open and that three
requirements in the change's specification do not yet appear in the accepted specifications. It
asks whether to proceed. On yes, it moves the change to
`changes/archive/2026-08-30-<name>/`, writes the `VERSION` file recording `0.6.0`, and reports what moved and
what the project now guarantees. On no, nothing has moved and nothing is half done.

## What Changes

- **`archive` performs the archival.** It moves the change directory into the archive, names it,
  writes its `VERSION` file, and reports. The operation either completes or leaves the project
  exactly as it found it.

- **An accepted obligation is withdrawn.** The closing sentence of `project-bootstrap`'s version
  requirement states that *a command whose stated job is checking SHALL NOT write to the project it
  checks*. That sentence is why archival is half an operation. What it was protecting — that the
  program never decides what a version increment means, and never writes the declaration — is
  restated as the narrower obligation it always was, on the act rather than on the command's name.

- **The version gate reads what the project actually records.** `last_archived_version` stops
  scanning directory names and reads the `VERSION` file inside each archived change, which is the
  form `toolchain-packaging` already requires and which all 11 archived changes already carry.
  `archive` writes that file as it moves the change, so the convention the gate reads and the
  convention the project records become one convention with one author. The gate starts working
  against the existing archive on the first run, with nothing renamed.

- **The contract still gates absolutely, and the work only informs.** A missing specification, a
  wrong declared kind, a validation error, or a version that does not advance refuses with no
  prompt, exactly as today. Incomplete tasks and unmerged requirements warn and ask. That is
  OpenSpec's policy, stated in its own workflow as *don't block archive on warnings — just inform
  and confirm*, and it is right: the contract is the thing archival makes permanent, and the rest
  is a contributor's business.

- **The merge stays agent-driven, and archival gains a way to tell whether it happened.** Deciding
  which accepted requirement a new scenario belongs under is judgment, and `artifact-kinds` says
  so. Observing that a requirement identifier present in the change's specification is absent from
  every accepted specification is not judgment. Archival assesses, reports, and asks. It never
  merges.

- **Four commands are added:** `list`, `status`, `instructions`, and `store`. `archive` needs the
  first two to select a change and to report completion, and the workflow instructions have been
  written against all four since they were vendored.

## Non-Goals

- **A merge performed by the program.** `artifact-kinds` requires every contract update to be
  agent-driven and gives the reason: a proposal and a design have no imposed structure, so no
  rule locates an obligation inside them. That requirement is untouched by this change. OpenSpec
  does not merge programmatically either — its own workflow delegates the merge to an
  agent-driven sync — so this is not even a gap against the tool being matched.

- **An extractor that produces a specification from a proposal.** Same requirement, same reason.
  `archive` continues to refuse a change with no `specification.html` and to say why.

- **`view`, `workset`, `feedback`, `doctor`, `completion`, `schema`, `schemas`, `templates`, and
  `config`.** Nine of OpenSpec's remaining commands. Some serve its runtime-schema model, which
  HTMLSpec rejects in favour of a closed set of four artifact kinds. Others are conveniences no
  workflow here invokes. Adding a command because another tool has one is how a command surface
  stops being a surface.

- **Advancing the version.** `archive` still compares and never edits the declaration. Deciding
  whether a change earned a patch, a minor, or a major increment is judgment.

- **Migrating this repository.** This change makes archival complete. Whether `html-spec` itself
  moves off OpenSpec is `migrate-an-openspec-project`'s subject and is decided separately.

- **Retrofitting the 11 existing archived changes.** Their directory names carry no version and
  this change does not rename them. The gate reads the `VERSION` file, which they already have.
