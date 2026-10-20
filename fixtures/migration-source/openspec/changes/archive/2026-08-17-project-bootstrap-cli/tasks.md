## 1. The project root as a type

- [x] 1.1 Add `crates/htmlspec-cli/src/project.rs` with `ProjectRoot`, whose only constructors are the resolver and the bootstrap that just created one. The inner path stays private, so a `&Path` cannot be widened into a project by a caller who did not check
- [x] 1.2 Give `ProjectRoot` the accessors that derive the project's shape — `htmlspec()`, `specs()`, `changes()`, `archive()`, `assets()`. The store location must be derived from the root here and nowhere else, because deriving it from the document's directory is the defect that gives each change its own 6 MB copy
- [x] 1.3 Add `resolve(from: &Path) -> Result<ProjectRoot, NotAProject>` walking upward for a child `htmlspec/`, stopping at a filesystem root or a `.git` directory. Return a structured error naming the directory searched from, not a bare `Option`
- [x] 1.4 Test that resolution succeeds from a nested subdirectory, and separately that it stops at `.git`: construct a repository with no project nested inside a directory that has one, and assert the outer project is not resolved. That case is the reason the boundary exists
- [x] 1.5 Test that `assets()` returns the same path regardless of which subdirectory resolution started from, which is the property that makes one store per project true by construction

## 2. Point the existing commands at the project

- [x] 2.1 Change `commands::validate` and `commands::format` to resolve a project when given no path, keeping the single-file and single-directory cases as they are — a contributor naming one file means that file, and requiring a project for it would be a regression
- [x] 2.2 Change `commands::assets` to take a `ProjectRoot` and check the whole project. This is where the duplication defect closes: today it calls `Store::discover` on the directory named and reports `no asset store under .` while two 6 MB stores sit two directories down
- [x] 2.3 Add a test that establishes two changes with their own stores, runs the asset check from the project root, and asserts the duplication is reported. Written against the defect as measured, so it fails before task 3 and passes after
- [x] 2.4 Change `commands::archive_check` to resolve the project and accept a change by name as well as by path, since a change's location is now derived rather than typed
- [x] 2.5 Add the missing-project report: what was searched for, from where, and that `htmlspec init` creates one. Exit `OperationalFailure`

## 3. Rename init to new

- [x] 3.1 Restructure the clap definition: `Init` takes no positional argument, and a new `New` subcommand takes `new change <name>` and `new <kind> <path>`. Model the two forms as a nested subcommand rather than a flag, because `--change` conflicting with `--kind` is the current shape and it encodes a sum type as two flags
- [x] 3.2 Move the body of `commands::init` to `commands::new_document` and `commands::init_change` to `commands::new_change`, both taking a `ProjectRoot` and both referencing the project's store rather than establishing one per document
- [x] 3.3 Make `new change <name>` place the change under the project's `changes/` unconditionally and report the resolved path. Resolves the design's first open question in favor of the unconditional form: a path argument would let a change be created outside `changes/`, which makes the root pointless
- [x] 3.4 Add the withdrawn-spelling error: `init <path>` reports that document creation moved to `new` and names the equivalent invocation. clap's unknown-argument error is not sufficient, because this spelling was valid yesterday and the reader needs the replacement rather than a usage line
- [x] 3.5 Update `crates/htmlspec-cli/tests/cli.rs` and `tests/archive.rs`, which pin the old argument shape directly
- [x] 3.6 Assert in the help test that `init` takes no required argument, which is the property the proposal's first reported defect was about

## 4. The tool table

- [x] 4.1 Add `crates/htmlspec-cli/src/tools.rs` with `ToolId` as a closed enum, `Tool` as the table row, and `Delivery` as a sum type over `SkillsAndCommands`, `SkillsOnly`, and `SingleFile`. `Delivery` must be a sum type rather than a set of booleans: OpenSpec resolves the same question with four predicate functions and can represent a tool that receives nothing, which it handles by printing an apology at the end of `init`
- [x] 4.2 Populate the table with Claude Code, Codex, Cursor, Pi, and AGENTS.md, each with its directory, detection paths, and delivery
- [x] 4.3 Add `detect(root: &Path) -> Vec<ToolId>` checking each tool's detection paths. A detection path may be a file or a directory, so test existence rather than directory-ness — the check that recognizes `.github/copilot-instructions.md` and a bare `.claude/` with the same code
- [x] 4.4 Add a test asserting every table entry's detection paths include its own directory, so a tool cannot be added that installs into a directory it would never detect
- [x] 4.5 Add `--tools` parsing over `all`, `none`, and a comma-separated list, rejecting an unknown name with the supported list and rejecting `all` or `none` combined with a name. Reject before anything is written
- [x] 4.6 Test that a `Selection` naming no tool is valid and distinct from an absent flag, since `--tools none` is a legitimate answer and the absence of the flag is a question not yet asked

## 5. Bootstrap, with no terminal in it

- [x] 5.1 Add `Selection` as plain data — the tools and the flags a prompt would collect — and `bootstrap(&ProjectRoot, &Selection, &mut Streams) -> Outcome`. `bootstrap` must not reference `inquire` or read a keypress; every case worth testing is a constructed `Selection`
- [x] 5.2 Create the directory shape and establish the one asset store at the project root. Reuse `establish_assets`, whose per-document call site is what task 3.2 removed
- [x] 5.3 Implement extend mode: `htmlspec/` already existing means refresh instructions, create missing directories, and leave `specs/`, `changes/`, and `archive/` alone. Nothing is deleted — the project has shipped nothing that could be stale, and a deletion path with no case to exercise it is a deletion path with no test
- [x] 5.4 Verify an existing store against the pinned lockfile and report a discrepancy rather than overwriting. `htmlspec_assets::verify_vendored_assets` already produces the violations
- [x] 5.5 Test first-run and re-run against the same directory, asserting a re-run rewrites skills, leaves a hand-edited document under `changes/` byte-identical, and does not grow the store

## 6. Instruction content, compiled in

- [x] 6.1 Move `.claude/skills/htmlspec-archive-change/SKILL.md` into the assets crate as the source of the archive skill and `include_str!` it. Reading skill files from disk at runtime is not available: the standalone-artifact requirement means the packaged binary runs with no checkout, which is the same reason `build.rs` uses `include_bytes!` for assets
- [x] 6.2 Add the skill table — one entry per workflow — carrying the workflow name, the installed directory name, and the content. Ship the `archive` skill's content in this change; the remaining workflows are named as non-goals
- [x] 6.3 Write skills with frontmatter recording the generating version, so an installation predating its instructions is identifiable
- [x] 6.4 Implement the three `Delivery` cases. `SkillsAndCommands` writes a skill directory and a command file per workflow; `SkillsOnly` writes skills alone; `SingleFile` writes a marked region
- [x] 6.5 Implement the marked region: absent means append, present means replace between the markers, opened-but-not-closed means report and write nothing to that file. Test all three — the third is the one that would otherwise delete a contributor's prose
- [x] 6.6 Test that a `SkillsOnly` tool receives no command file, and that every tool in the table receives at least one artifact. The second is the state the sum type is supposed to make unreachable, asserted rather than assumed

## 7. The prompt layer

- [x] 7.1 Add `inquire` pinned exactly in `Cargo.toml` workspace dependencies and to `htmlspec-cli`
- [x] 7.2 Add `prompt(...) -> Result<Selection, Cancelled>` as the only module referencing `inquire`. Its whole job is producing a `Selection`; it writes no files
- [x] 7.3 Implement the interactivity test: stdin is a terminal, stdout is a terminal, no `--tools`, and no `--json`. Any one missing means proceed with detected tools and no prompt. `--tools` suppressing the prompt is deliberate — a user who named their tools has already answered
- [x] 7.4 Make `--json` imply non-interactive, since a prompt writes to the stream structured output promises to occupy alone
- [x] 7.5 Add `Outcome::Cancelled` with a distinct code, and extend `each_outcome_has_a_distinct_code_and_only_success_is_zero` to cover it. Return it through the normal return path: nothing in the prompt layer calls `process::exit`, which is the shortcut OpenSpec takes in its cleanup prompt and the one this program's single exit point exists to avoid
- [x] 7.6 Restore the terminal on interrupt before returning `Cancelled`
- [x] 7.7 Test the non-interactive path end to end with no terminal attached, asserting it completes without blocking — the property that makes `init` safe in continuous integration

## 8. The project declaration

- [x] 8.1 Add `spec.nix` generation to `bootstrap`, carrying the project name, a starting version, and a pinned reference to the `htmlspec` binary. Write it only when absent — task 5.3's rule that an existing configuration survives applies to this file, and a contributor who edited the pin meant it
- [x] 8.2 Add a reader that evaluates `spec.nix` and returns a parsed declaration. Evaluate through the existing Nix toolchain rather than pattern-matching the file as text: a hand-rolled parser accepts files Nix rejects, and then the build and the program disagree about the project
- [x] 8.3 Make the reader's result a type whose construction proves the three fields are present, so a caller cannot read a version from a declaration that has none
- [x] 8.4 Derive the default project name from the project root's directory name, and write `0.0.0` as the starting version from a single named constant. `0.0.0` is what makes the version comparison total — a project always has a version on record, so there is no absent-previous-version case for the gate to special-case
- [x] 8.5 Test that a re-run leaves an edited `spec.nix` byte-identical, which is the requirement's second scenario and the one a careless `write` breaks

## 9. The version gate

- [x] 9.1 Record the archived version alongside each archived change, so archival has a previous value to compare against. Decide and document where it lives — the archive directory's name is one option and a field in the archived change is another
- [x] 9.2 Make the previous version total: with no archived change present, it is the initial `0.0.0` from task 8.4 rather than an absent value. This is the one place the totality is established, so it is the place a reader looks for it, and a `None` reaching the comparison is a defect rather than a case to handle
- [x] 9.3 Add a `Version` type parsed from three dot-separated numbers and ordered componentwise, with parsing at the boundary so an unorderable version is rejected once rather than compared as a string. A string comparison would order `0.10.0` before `0.9.0`
- [x] 9.4 Extend `commands::archive_check` to read the declaration and refuse unless the declared version is ordered after the last archived one. Report three outcomes distinctly: unchanged, ordered backwards, and unparseable. A difference test would accept a downgrade, which is why this is an ordering
- [x] 9.5 Assert that `archive_check` does not write. Add a test that snapshots `spec.nix` before and after both a refusal and a success, because the requirement says the program compares and never edits — and a test is the only thing that keeps a convenience bump from being added later
- [x] 9.6 Handle the absent declaration: report it and name `htmlspec init`, exiting `OperationalFailure` rather than passing the gate by default
- [x] 9.7 Assert the first archival is not a special case: bootstrap a project, archive its first change without advancing from `0.0.0`, and confirm the refusal is the same one a later archival gives. Assert separately that no branch reads an absent previous version, since such a branch would run once per project and be tested once
- [x] 9.8 Update the archive skill to state that the agent advances the version before running `htmlspec archive`, since the program checks and does not write

## 10. Close the orphaned-reference gap

- [x] 10.1 Add an ascent test to the asset rules: a reference containing `../` asserts a store above the document. Reading the reference string rather than the filesystem keeps the rule a property of the document
- [x] 10.2 Report an ascending reference when no store is found in any directory above the document. `store_for` already walks upward, so the walk's failure is the signal — what is missing is treating that failure as suspicious rather than as out of scope
- [x] 10.3 Keep the existing early return for non-ascending references, and assert all 41 conformance fixtures still pass. None of their 341 asset references ascends, which is the measured fact the rule's scope rests on
- [x] 10.4 Add a conformance fixture for the ascending-reference case, so the rule is pinned by the same mechanism as every other rule
- [x] 10.5 Add a regression test built from the measured defect: create a project, copy one change out of it, and assert strict validation now reports rather than accepting with zero diagnostics
- [x] 10.6 Assert that reference shape cannot exempt a document — a reference edited to stop ascending no longer resolves, so the escape hatch breaks the thing it would hide

## 11. The closing message

- [x] 11.1 Report what was installed, where, and whether each tool was newly configured or refreshed
- [x] 11.2 Name the literal next command per selected tool, one line per distinct spelling with the tools it applies to. A canonical form no selected tool answers to teaches a command that does not work, which is why this is printed after the selection is known rather than in a banner
- [x] 11.3 With no tools selected, report the project was established and name no command
- [x] 11.4 Add the banner as hand-rolled ANSI, suppressed when not a terminal, when `NO_COLOR` is set, and when the terminal is too narrow

## 12. Reconcile the project with itself

- [x] 12.1 Regenerate this repository's `.claude/skills/htmlspec-archive-change/` by running `htmlspec init --tools claude`, and confirm the result matches what was moved in 6.1. A skill that is only ever hand-copied is never verified to be what the tool installs
- [x] 12.2 Update `README.md`: the workflow example, the project layout, and every `init` invocation
- [x] 12.3 Update `AGENTS.md` and the archive skill for `new` and for the resolved change location
- [x] 12.4 Regenerate the flake's vendored dependencies for `inquire` and its four transitive crates, and confirm the reproducible-build check still passes. This is the vendoring obligation the dependency was accepted with
- [x] 12.5 Run the full check suite, and separately confirm the first-run experience by hand in an empty repository: `git init`, `htmlspec init`, then the command the output names. The defect this change exists to fix is a first impression, and no test asserts a first impression

## 13. Evidence

- [x] 13.1 Record the before-and-after asset footprint for a project with two changes — measured at 6.0 MB each before — in `evidence/asset-footprint.md`
- [x] 13.2 Record the first-run transcript from 12.5 in `evidence/first-run.md`, so the experience this change is judged on is written down rather than remembered

## 14. Close the empty-install path

Found by using the tool rather than by testing it: `init` in a fresh repository reported `the project is ready` having installed nothing, because nothing was detected, the list opened unchecked, and confirming it succeeded. Task 12.5 confirmed the first run by hand and did not catch this, because the person running it knew which tools they wanted and checked them.

- [x] 14.1 Preselect every tool where detection finds nothing. An absent detection means a project nobody has configured, which is where a default carries the most weight. Note in the code that this is contingent on the table staying small — OpenSpec preselects nothing here and at 34 tools that is correct
- [x] 14.2 Refuse an interactive selection of no tools and ask again, naming the argument that means no tool. Keep `--tools none` succeeding: an argument is an answer, and an empty confirmation is usually a question not read
- [x] 14.3 Add `scripts/check-prompt`, driving the real prompt through a pseudo-terminal, and add it to `scripts/check`. A script rather than a `cargo test` because `openpty` is an unsafe FFI call and this workspace forbids unsafe code
- [x] 14.4 Cover cancellation's exit code in that check. Three of the four exit codes had integration tests that read a real process status, and the code this change added was the fourth
- [x] 14.5 Confirm the check fails with either guard removed. A guard whose test passes without it is not testing the guard
- [x] 14.6 Record the divergences from `openspec init` in `evidence/openspec-divergence.md`, including the configured-against-detected distinction the tool table cannot express
