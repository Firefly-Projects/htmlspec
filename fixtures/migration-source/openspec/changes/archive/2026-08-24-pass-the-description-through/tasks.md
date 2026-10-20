## 1. Establish the loss before changing anything

- [x] 1.1 Record that no generated file carries a placeholder: `grep -rc 'ARGUMENTS' .claude .codex .pi` returns 0 in every file, and no body under `crates/htmlspec-assets/instructions/` contains the token. Both halves matter — the frontmatter that declares an argument and the body that consumes it are separate omissions
- [x] 1.2 Quote the two writers by file and line: `write_skills` at `crates/htmlspec-cli/src/install.rs:88` and `write_commands` at `:124`, neither emitting `argument-hint`. Name the fields each does emit, so the omission is shown against a complete list rather than asserted
- [x] 1.3 Record the convention from an artifact rather than from memory: Claude Code's example plugin ships `argument-hint` with `$ARGUMENTS` in both its command and its skill form and documents the two as loaded identically, and a Codex plugin command branches on `$ARGUMENTS`. A convention this change follows must be shown to exist
- [x] 1.4 Confirm no existing test asserts anything about arguments, so a green suite is expected rather than evidence the surface works. Put this first because it is what makes the absence of a failing build unsurprising

## 2. The hint is a property of the workflow

- [x] 2.1 Add `argument_hint: &'static str` to `Workflow` in `crates/htmlspec-assets/src/instructions.rs`, documented with why it is per-workflow rather than one shared string
- [x] 2.2 Give each of the seven workflows a hint naming what it takes: a description of the work for `new` and `ff`, the idea for `explore`, the change to act on for `apply`, `verify`, `continue`, and `archive`
- [x] 2.3 Make every hint optional in form — `[description]`, not `<description>` — so a bare invocation is not rejected by a tool. A required hint would break the behavior the archived change deliberately kept
- [x] 2.4 Do not add a constant in `install.rs` instead. `project-bootstrap` requires the workflow set to be data, and a hint in the writer makes a new workflow's hint a change to a function

## 3. The writers emit it

- [x] 3.1 Emit `argument-hint` in `write_skills`, in the frontmatter beside `allowed-tools`
- [x] 3.2 Emit `argument-hint` in `write_commands`, in the same position, so the two kinds of file differ in their header only where they must
- [x] 3.3 Leave `write_region` untouched. A marked region in a shared file has no invocation, so there is no argument to declare — and `Installed.regions` being its own field is what keeps the tests below from reaching it
- [x] 3.4 Change no other field. `name`'s absence from command files is a settled decision with three recorded wrong attempts behind it, and this change must not disturb it

## 4. The bodies consume it

- [x] 4.1 Add an opening `## What you were asked` section carrying `$ARGUMENTS` to each of the seven bodies under `crates/htmlspec-assets/instructions/`
- [x] 4.2 Put it at the top, before the procedure. An agent that reads seven sections of method before learning its input has been given the document in the wrong order
- [x] 4.3 State in the section that an empty substitution means nothing was passed and the workflow's own guidance on an absent input applies. Without this an agent cannot tell a lost input from an absent one, and suspecting the former is what produced the recorded refusal
- [x] 4.4 Change nothing else in any body. The guidance is correct as of 0.3.0 and re-opening it would re-litigate an archived change
- [x] 4.5 Use `$ARGUMENTS` for every tool. A per-tool spelling would put a tool-specific string in a body the contract requires to be tool-independent

## 5. Check it against produced files

- [x] 5.1 Add a test to `install.rs` that installs for every tool in `TOOLS`, reads back every skill and command file, and asserts the frontmatter declares a hint and the body carries the placeholder
- [x] 5.2 Assert against the files rather than against `WORKFLOWS`, following the stated convention on `every_generated_name_is_loadable` and `generated_files_record_the_generating_version`: a value assembled by a format string at write time is invisible to a check on its inputs
- [x] 5.3 Confirm the test fails before the writers are changed, and record the failure. A test that has never failed is a test whose subject is unproven
- [x] 5.4 Confirm the region-delivered tool is not reached, by construction rather than by an exclusion in the test body
- [x] 5.5 Run `cargo test` and `./scripts/check`. The bodies are embedded in the binary, so the generated-file tests read the amended text

## 6. Check it against the case that failed

- [x] 6.1 Rebuild, reinstall into a scratch project, and confirm the installed files carry both the hint and the placeholder
- [x] 6.2 Run the recorded tally-mark request through `/htmlspec-new` and confirm the description reaches the agent and a change is created on the first exchange, with a derived name stated in the report
- [x] 6.3 Confirm the derived name comes from the contributor's description rather than from the worked example the body ships. This is what the archived change's dropped task could not observe, and the distinction is the whole point of the run
- [~] 6.4 **Dropped.** Run a bare `/htmlspec-new` with no description and confirm it still asks rather than inventing, and that the empty section did not read as a lost input. The empty path is asserted two ways in code — `every_workflow_declares_an_optional_argument` requires each hint to be optional in form so no tool rejects a bare invocation, and each body states that an empty substitution means nothing was passed — but no live run observed it. One attempt was made and proved nothing: the files it exercised were 0.2.0 copies with no placeholder at all, so the agent had no empty section to read. The remaining risk is narrow and asymmetric: a bare invocation that wrongly asks is the behavior this change preserves deliberately, so the failure mode a run would catch is a tool rejecting the invocation outright, which the optional-form assertion already prevents at the point the file is written. Worth stating at archival rather than leaving as an unticked box
- [x] 6.5 Record both runs as evidence. `evidence/pi-substitutes-2026-08-24.md` records the described-change run on Pi; the bare-invocation run is recorded there as not yet observed on a live tool. The check for an invocation surface is a transcript, because substitution happens in a tool this program does not control
- [x] 6.6 Note in the evidence whether Cursor and Pi substitute, if a run on either is available. The design accepts not knowing; recording an observation costs nothing and retires a stated risk

## 7. The contract

- [x] 7.1 Add to `openspec/specs/project-bootstrap/spec.md` the obligation that a generated invocation surface passes a contributor's arguments to the agent, and that a workflow declares what it takes
- [x] 7.2 Write the requirement so it does not reach a delivery kind with no command surface, rather than stating an exception for one
- [x] 7.3 Add a scenario for an invocation carrying a description, one for a bare invocation, and one for the placeholder surviving into the produced file
- [x] 7.4 Apply it as a partial update, leaving every requirement and scenario this change does not name intact
- [x] 7.5 Confirm no other accepted requirement is touched, and run `openspec validate --all`

## 8. Release

- [x] 8.1 Regenerate this repository's own 35 installed files, so the project can use the fix it ships. `0.3.0` was released to repair exactly the fault of dogfooding text the source had removed
- [x] 8.2 Advance the workspace version and update `Cargo.lock` offline. `--locked` fails the suite's stages otherwise
- [x] 8.3 Write the changelog entry, and state in it what the run established about the archived change's dropped task 5.2 — as a remark, since that task stays dropped and this change does not reopen it
- [ ] 8.4 Record the version in the archived change's `VERSION` file at archival, and confirm `scripts/check-release` passes with it
