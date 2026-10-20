## Context

The program has five subcommands, all of which take paths and return an `Outcome`. None of them has a subject larger than a document, and none of them talks to a terminal. This change adds an operation whose subject is a project and whose input is partly a person pressing keys. Both are new, and each one is a way to make the rest of the program worse if it is added carelessly.

The constraints that actually bind:

- `unsafe_code = "forbid"` and `unwrap_used = "warn"` at the workspace level.
- Every dependency exact-pinned, because the reproducible-build requirement says two machines must agree.
- One exit point, in `main`. Every command returns `Outcome` rather than exiting, which is what makes them testable without spawning a process.
- Structured output must contain nothing but the payload on stdout.

The last two are the ones a prompt threatens. A library that reads a keypress and writes to the terminal wants to own both streams and to exit on interrupt, and both would break a contract the program already keeps.

## Goals / Non-Goals

**Goals**

- `htmlspec init` in an empty repository produces a working project and tells the user what to type next.
- One asset store per project, referenced relatively by every document in it.
- Commands work from any subdirectory.
- The interactive path is thin enough that everything below it is testable without a terminal.
- A sixth agent tool is a table entry.

**Non-Goals**

- Implementing `explore`, `propose`, `apply`, or `verify` as commands. `init` installs their instruction files; only `archive` has program-side behavior today.
- A project config file carrying rules or context. `spec.nix` holds a name, a version, and a binary pin — each read by a command. Authoring rules and project context of the kind `openspec/config.yaml` carries are out of scope, because no htmlspec command consumes them yet.
- Global user-level configuration, profiles, or delivery modes. OpenSpec has all three; they exist because it accumulated tools and workflows over time, and adopting them before that pressure exists would be adopting the scar tissue without the wound.
- Migration from an OpenSpec project layout.

## Decisions

### The project root is a type, not a path

`ProjectRoot` wraps a directory known to contain `htmlspec/`, constructed only by a resolver that walked upward and found one, or by `init` which just created one. Every command that operates on a project takes a `ProjectRoot` rather than a `&Path`.

This is the parse-at-the-boundary rule applied to the one ambiguity this change introduces. Once commands walk upward to find a root, "the path the user typed" and "the project this belongs to" are different things with the same representation, and the current signatures — `validate(path: Option<&Path>)` — cannot tell them apart. A resolver returning `Result<ProjectRoot, NotAProject>` makes the failure explicit at one place instead of letting five commands each decide what to do in a directory that is not a project.

The asset store's location follows from the root rather than being computed per document: `root.assets()` is `<root>/htmlspec/assets`. This is the whole fix for the 6 MB-per-change duplication. The current code derives the store from the *document's* directory, which is why two changes get two stores; deriving it from the project makes one store the only thing that can happen.

`validate` and `format` keep accepting a bare path for the single-file case. A contributor naming one file means that file, and requiring a project for that would be a regression.

### Resolution walks upward and stops at a boundary

The resolver ascends from the starting directory looking for a child `htmlspec/` directory. It stops at a filesystem root or a `.git` directory, whichever comes first.

Stopping at `.git` is the decision worth stating. Without it, a repository nested inside a directory that happens to contain an unrelated `htmlspec/` would resolve outward past its own boundary and write into a project it is not part of. Ascending forever is how a tool ends up writing to a sibling project because someone ran it in the wrong directory. `.git` is the marker for "this is a repository," and a repository is the outermost thing a command should reach.

Rejected: requiring `--root` explicitly. It is correct and nobody would pass it. Rejected: a marker file inside `htmlspec/`. The directory's existence is already the marker, and a second one is a second thing to get out of sync.

### Interaction is a decision, then a prompt that produces one

The layering:

```
Selection { tools: Vec<ToolId>, cleanup: bool, ... }   // plain data
    ↑                                    ↑
resolve_from_flags(&Args)          prompt(&mut dyn Interaction)   // inquire lives here
    ↓
bootstrap(&ProjectRoot, &Selection) -> Outcome         // pure, no terminal
```

`bootstrap` never sees a terminal. It takes a `Selection` and does the work, so every case that matters — first run, re-run, one tool, all tools, none, a tool whose directory exists — is a unit test constructing a `Selection` and asserting on files written. The prompt's only job is producing a `Selection`, and it is the only part with no test beyond "it compiles and a human ran it."

This is the answer to the impact the proposal named: without the split, the interactive path is the one part of the program with no test, and it is also the part a new user meets first.

`inquire` is called from `prompt` alone. It does not appear in `bootstrap`'s signature or in the module that writes files. If it is ever replaced, one function changes.

### Deciding whether to prompt at all

Three conditions, all required: standard input is a terminal, standard output is a terminal, and no `--tools` flag was passed. Any one missing means non-interactive, and non-interactive means use detected tools without asking.

`--tools` suppressing the prompt is the part that reads as surprising and is not. A user who named their tools has answered the question the prompt asks; showing it anyway would ask them to answer twice. This matches OpenSpec's `canPromptInteractively`, which returns false when `toolsArg !== undefined` for the same reason.

Checking stdout as well as stdin is because a prompt writes; if its writes are going into a pipe, the user cannot see what they are answering.

An interactive run also honors `--json` by refusing: structured output promises stdout contains nothing but the payload, and a prompt writing to stdout breaks that promise outright. `--json` implies non-interactive.

### The tool table is data, and its shape is a sum type

```rust
pub struct Tool {
    pub id: ToolId,               // enum: closed set, so a typo is a compile error
    pub name: &'static str,       // "Claude Code"
    pub directory: &'static str,  // ".claude"
    pub detection: &'static [&'static str],
    pub delivery: Delivery,
}

pub enum Delivery {
    SkillsAndCommands { commands: &'static str },  // ".claude/commands/htmlspec"
    SkillsOnly,
    SingleFile { path: &'static str },             // "AGENTS.md"
}
```

`Delivery` is a sum type because the three cases are genuinely different operations — write a directory of skills plus a directory of commands, write skills only, or append a section to one file — and OpenSpec's equivalent is a set of booleans (`shouldGenerateSkillsForTool`, `shouldGenerateCommandsForTool`, `shouldReconcileCommandFilesForTool`) resolved against a delivery mode and a per-tool capability. That arrangement has four functions deciding what a tool gets, and combinations that should be impossible are representable — `delivery=commands` on a skills-only tool yields a tool that gets nothing, which OpenSpec handles by printing a paragraph of apology at the end of `init`. A sum type makes that state unconstructible.

Five entries: Claude Code, Codex, Cursor, Pi, AGENTS.md.

`detection` is a list of paths because a tool can be recognized by more than one thing and because a rename must not lose recognition of the old directory — the case OpenSpec handles with `detectionPaths: ['.devin', '.windsurf']`.

### AGENTS.md is a marked region, and that is why it is a distinct delivery

`SingleFile` appends a region delimited by HTML comment markers and rewrites only between them on re-run. A file the user also edits by hand cannot be overwritten wholesale, and it cannot be appended to blindly either or every `init` grows it.

If the markers are absent the region is appended. If they are present the content between them is replaced. If the opening marker is present without the closing one, `init` reports it and writes nothing to that file: a truncated region means someone edited across the boundary, and guessing where it ended could delete their prose.

### Skills are compiled in, and the archive skill's content moves

Skill and command content is `include_str!`'d into the binary, the way templates already are. The standalone-artifact requirement means the packaged program runs with no source checkout, so reading skill files from disk at runtime is not available — the same reasoning `htmlspec-assets/build.rs` records for `include_bytes!`.

The existing `.claude/skills/htmlspec-archive-change/SKILL.md` becomes the source file for the archive skill, moved under the assets crate. This repository's copy then comes from `htmlspec init`, which is the only way the instructions get tested: a skill that is only ever hand-copied is never verified to be what the tool installs.

Skills carry the version they were generated by in their frontmatter, so a stale skill is identifiable. OpenSpec does this with `generatedBy` and it is the mechanism by which a re-run's value is visible.

### Re-run refreshes instructions and touches nothing else

Extend mode is `htmlspec/` already existing. In it:

- Skills and commands for selected tools are rewritten unconditionally. They are program-owned output; the user's edits to them are not preserved because the file says which version generated it and refreshing is the point.
- `specs/`, `changes/`, and `changes/archive/` are created if absent and otherwise untouched.
- The asset store is established if absent, and verified if present. A store whose bytes do not match the pinned lockfile is reported, not silently rewritten.

Nothing is deleted. OpenSpec's `init` removes legacy artifacts and stale command files, which it needs because it has renamed things across versions. htmlspec has shipped nothing to be stale. A deletion path added before there is anything to delete is a deletion path with no test case.

### `spec.nix` is written, because it has a consumer

`init` writes `spec.nix` at the project root carrying three fields: the project's name, its version, and a pinned reference to the `htmlspec` binary the project expects.

I deferred this file in the first draft on the grounds that nothing read it, and that a file nothing reads drifts from the truth silently because nothing fails when it is wrong. That objection is answered by giving it a consumer rather than by arguing about it. Archival reads the version and refuses when it has not moved, so the drift *is* the failure and the file cannot rot unnoticed. This is the difference in kind from OpenSpec's `config.yaml`, which holds project context an agent reads and no command enforces.

Nix rather than YAML or TOML, and the binary reference is why. A pinned tool reference is a flake input with a hash — the project already builds with a flake, and expressing the pin anywhere else means inventing a syntax for something Nix states directly and then writing a resolver for it. The other two fields are along for the ride, and a second configuration language for them is not worth it.

The binary pin closes a gap that is open today. `--version` reports `htmlspec 0.1.0 (9c3a3c79b1fe-dirty)`, so the binary knows what it is, but no project records which binary it expects. Two machines agree today because both enter the same development shell, which is convention rather than a checked fact. The reproducible-build requirement asks for more than convention.

### The version gate checks; it does not write

Archival compares the version in `spec.nix` against the last archived version and refuses unless the declared version is ordered after it. The contributor or the agent performs the bump. `htmlspec archive` never edits `spec.nix`.

`init` writes `0.0.0`, which is what makes the comparison total. A project starts with a version already on record, so there is never a previous value to be absent, and the first archival is the same operation as the thousandth: advance past what is recorded, or be refused. The alternative — treating an absent previous version as a pass — is a branch that executes once per project, which means it is a branch nobody exercises twice.

`0.0.0` also settles a question I had left implicit. I first wrote the gate as "refuses while they are equal", which is a test for difference, and a difference test accepts a downgrade: archiving at `0.1.0` after `0.2.0` passes. Once the initial value is `0.0.0` the versions are obviously meant to be ordered rather than merely distinct, so the comparison is an ordering. A version ordered *before* the last archived one is reported separately from one that has not moved, because the causes differ — work not yet done in the second case, and a version that went backwards in the first.

That makes the version a value the program parses into an ordering rather than a string it compares. Three dot-separated numbers, ordered componentwise. A version it cannot parse is reported rather than accepted, because a gate that cannot compare its inputs is not a gate. This stays within the checking side of the division: parsing `0.1.0` and asking whether it follows `0.0.0` is exact, while deciding whether the change deserved a minor or a major bump is judgment and stays with the agent.

This follows the division the project already chose, and the choice is not a new one. An agent produces the change's specification and the program validates it, because the judgment is judgment and the checking is exact. A version an agent sets and the program checks is that same split. The alternative puts a write inside the one command whose stated job is checking, and `archive_check` is documented as reading one file and validating it.

The gate then reduces to a question the program answers exactly: is this version ordered after the last archived version? That is a comparison rather than a judgment, which is the test for what belongs in the program instead of in an instruction.

### An ascending asset reference makes an absent store suspicious

Every asset rule returns early when no store is present. That guard is correct — the 41 conformance fixtures carry symbolic references and never sit beside a store, so a rule firing without one fails all of them. Its consequence is that a fixture and a change directory copied out of its project are indistinguishable, and I measured that: `htmlspec validate --strict` accepts an orphaned change whose seven asset references are all dead, reporting zero diagnostics.

One store per project makes that case reachable in a way it currently is not, so this change closes it. The signal is the shape of the reference itself. A reference that ascends out of its own directory — one containing `../` — asserts that a store sits above the document. A fixture's reference never ascends: I checked all 341 asset references in the conformance fixtures and **none** contains `../`, while every reference `init` generates for a change three levels down does.

So the rule is: an ascending asset reference with no store found anywhere above it is reported. A non-ascending reference with no store stays silent, which keeps every fixture passing. The document still cannot exempt itself, because ascent is a property of the reference the generator wrote rather than a marker an author can set.

Rejected: requiring a store unconditionally, which fails 41 fixtures. Rejected: keying the rule on `spec.nix` being present. That reads better than it works — a change directory carries no `spec.nix`, so the orphan would look storeless and unmarked exactly as it does now.

### Cancellation is a fourth outcome

`Outcome` gains `Cancelled` with a distinct code. The three existing outcomes were designed for documents: success, the document is wrong, the operation could not run. A user who pressed Escape at a menu is none of those, and mapping it onto `OperationalFailure` would make a continuous-integration job unable to distinguish "the tool broke" from "a human declined," which is the same distinction the exit-code requirement already draws between document violations and operational failures.

Interrupt during a prompt restores the terminal and returns `Cancelled` through the normal return path. The one-exit-point rule holds: nothing in the prompt layer calls `process::exit`. OpenSpec calls `process.exit(0)` inside its legacy-cleanup prompt, which is exactly the shortcut this program's structure exists to avoid.

### The closing message names the literal command per tool

After a successful `init`, the output names what to type — `/htmlspec:archive` for a tool that got slash commands, the skill invocation for a tool that got skills only — and prints one line per distinct spelling when the selection disagrees.

This is copied from OpenSpec deliberately, and the reason is in its own source comment: printing a canonical form that no selected tool actually answers to teaches the user a command that does not work. The spelling cannot be known until the tools are chosen, which is why this is the last thing printed rather than part of a banner.

## Risks / Trade-offs

- **The rename breaks every documented invocation.** `README.md`, `AGENTS.md`, `crates/htmlspec-cli/tests/cli.rs`, and the archive skill all spell `init <path>`. Mitigation: the removed spellings produce an error naming the replacement — `init` with a path says to use `new`, and clap's own unknown-argument error is not enough here because `init <path>` was valid yesterday.

- **A pinned interactive dependency is a vendoring obligation.** `inquire` brings four transitive crates the flake must fetch, and the reproducible-build check must still pass. Mitigation: the pin is exact, the lockfile is committed, and the flake's vendored set is regenerated as a task rather than assumed. The alternative — hand-rolled raw mode — trades a vendoring task for four classes of terminal bug, and one of them (failing to restore the terminal on interrupt) leaves the user's shell broken.

- **`ProjectRoot` touches every command's signature.** Five commands change shape in one change, alongside a rename and a new interactive surface. Mitigation: the tasks sequence the type before the interaction, so the pure layer is complete and tested before a prompt exists to call it.

- **One asset store is a different failure mode, not none.** Today a change carries its own runtime and is self-contained: the directory can be copied anywhere and still opens. With one store per project, moving a change directory out of its project breaks its asset references. This is the intended trade — the duplicate-detection requirement exists because the project decided sharing is correct — but it is a real property being given up, and `assets` reporting project-wide is what makes the remaining risk visible.

- **Two skill sets coexist in this repository.** `openspec-*` and `htmlspec-*` will both be in `.claude/skills/`. Mitigation: distinct prefixes, and `init` writes only the `htmlspec-*` names. It never scans for or removes anything it did not write.

## Migration Plan

1. Add `ProjectRoot` and the resolver. Nothing calls them; the type and its tests land first.
2. Repoint the existing commands at `ProjectRoot`, keeping the single-file path for `validate` and `format`. `assets` becomes project-wide here, which is where the duplication defect actually closes.
3. Rename `init` to `new`, reshaped as `new change <name>` and `new <kind> <path>`. The old spellings start erroring with a message naming the replacement.
4. Add the tool table, `Selection`, and `bootstrap` — the whole non-interactive path, driven by `--tools`. `init` is fully usable at this step with no prompt.
5. Add the prompt layer over it.
6. Move the archive skill's content into the crate and regenerate this repository's copy through `htmlspec init`.
7. Update `README.md`, `AGENTS.md`, and the flake's vendored dependencies.

Steps 1–4 are independently valuable and independently testable. If step 5 is deferred, `init --tools claude` still bootstraps a project, which is why the flag path is built before the menu rather than after it.

## Resolved Questions

Both questions the first draft left open are now settled, and they are recorded here rather than deleted because the reasoning is what a later reader needs.

- **Where a new change goes.** `new change <name>` places it at `<root>/htmlspec/changes/<name>` unconditionally and prints the resolved path. A path argument lets a change be created outside `changes/`, and a root that does not determine where changes live is not doing its job.
- **Whether `init` writes a configuration file.** It writes `spec.nix`. The deferral rested on the file having no consumer, and the version gate gives it one. See the two decisions above for the format and for why the gate checks rather than writes.
- **What the first archival compares against.** `init` writes `0.0.0`, and the gate compares against that rather than against nothing. The first archival is then an ordinary comparison: `0.0.0` is the version already on record, so archiving requires advancing past it exactly as every later archival requires advancing past its predecessor. There is no first-archival case, which is why this is settled rather than deferred — an absent-previous-version branch would be code that runs once per project and is therefore tested once, if at all.
