## 1. Pin the current behaviour before changing it

- [x] 1.1 Record as evidence that the version gate has never fired: `last_archived_version` (`crates/htmlspec-cli/src/commands.rs:1352`) scans archive directory names, `archived_version` (`:1371`) needs a part that `Version::parse` accepts, and `Version::parse` (`crates/htmlspec-cli/src/version.rs:56`) requires exactly three dot-separated numeric components. No directory in `openspec/changes/archive/` has one. Show the fallback to `0.0.0` by running the gate against this repository's archive rather than by reading the code
- [x] 1.2 Record the counterpart: all 11 archived changes carry a `VERSION` file, values `0.1.0` through `0.5.0`, and `toolchain-packaging`'s release-hygiene requirement is what asks for them. This is the convention the gate should have been reading
- [x] 1.3 Record the workflow's command usage as measured, not estimated: 170 invocations across the vendored OpenSpec prompts, `status` 52, `instructions` 48, `store` 22, `new` 14, `list` 14, `archive` 10, `schemas` 6, `context` 4. This is the evidence that the four commands are load-bearing rather than aspirational
- [x] 1.4 Record what OpenSpec's archival actually does, against its vendored workflow rather than its CLI help. The help advertises "update main specs"; `.pi/skills/openspec-archive-change/SKILL.md` routes the merge to `openspec-sync-specs` and calls it an agent-driven intelligent merge. The claim that this change does not diverge from OpenSpec on merging depends on this, so cite the file
- [x] 1.5 Confirm no accepted requirement other than `project-bootstrap`'s check-only sentence blocks archival from writing. Search every capability for obligations on what a command may write, and record what was searched — a withdrawal justified against one sentence is wrong if a second states the same thing elsewhere

## 2. The contract, before any code

- [x] 2.1 Modify `project-bootstrap`'s *Archival requires the project version to advance*: state that the last archived version is read from the recorded version and not from a directory name, and restate the reservation on the act rather than on the command's name
- [x] 2.2 Withdraw the check-only sentence inside that same requirement. It is the requirement's closing line rather than a requirement of its own, so it goes in the modification rather than in a removal, and the replacement states the reservation on the act. Carry every existing scenario forward — a modified requirement replaces its whole block, and dropping a scenario here would silently retire an obligation this change never argued against
- [x] 2.3 Modify `toolchain-packaging`'s *Archival produces the specification and updates the accepted contract*: archival completes the operation it is named for, and is atomic with respect to the project
- [x] 2.4 Add the requirement separating what refuses from what asks, with the contract on the refusing side and the work on the asking side
- [x] 2.5 Add the requirement that archival assesses the merge by comparing identifiers and never performs it. Derive it from `artifact-kinds` rather than asserting it fresh, because that capability already states the reason and a second statement of one obligation is a second thing to keep true
- [x] 2.6 Add the naming and collision requirement
- [x] 2.7 Add the requirement that archival is drivable without a person present
- [x] 2.8 Add the requirements for enumeration, completion reporting, and instruction emission
- [x] 2.9 Add the requirement for registered standalone projects
- [x] 2.10 Confirm `artifact-kinds` needs no delta. If stating that archival completes requires touching it, the split between assessment and merge is not holding and the design is wrong
- [x] 2.11 Run `openspec validate --all`
- [x] 2.13 Pin the subcommand and option spellings in the contract, and state the rule that produces them: an operation OpenSpec also provides carries OpenSpec's name. Rename the requirement that enumerated four subcommands, since it now names ten and its old title claimed a complete list it no longer held. Assert the spellings against `--help` — the installed instructions invoke these commands by name, so a rename nothing checks breaks every one of them silently
- [x] 2.12 Add the requirement for a mode that checks without performing, deriving it from the format subcommand's existing non-modifying mode rather than asserting it fresh. This came out of task 1.5's search: the search for obligations about writing surfaced the precedent, which is the kind of thing a search for one thing finds about another. State that the mode fails exactly where archival refuses, so the two cannot drift about what readiness means

## 3. The version gate, which is the smallest change and the one that already matters

- [x] 3.1 Rewrite `last_archived_version` to read each archived change's `VERSION` file and take the maximum. Keep the total-function property: absent archive, or archive with no recorded versions, yields the initial version
- [x] 3.2 Delete `archived_version` and its directory-name scan
- [x] 3.3 Assert the gate against this repository's real archive: the last archived version is `0.5.0`, so a declaration at `0.5.0` is refused as unmoved and one at `0.4.0` is refused as backwards. Both messages already exist and both are now reachable
- [x] 3.4 Assert an archived change with no `VERSION` file does not silently lower the maximum
- [x] 3.5 Land this before anything else in section 4. It is independently correct, it fixes a live defect, and it is the one part of this change that is not a design argument

## 4. Archival performs the archival

- [x] 4.1 Split the existing `archive_check` into the gate it already is and the operation that follows, so the refusing conditions keep their current messages and exit codes unchanged
- [x] 4.2 Add the task count, reading the change's tasks document. Report the count and offer to proceed
- [x] 4.3 Add the merge assessment: collect requirement identifiers from the change's `specification.html`, collect them from the accepted specifications, report identifiers present in the first and absent from the second. Name them individually — a count is not checkable in under a minute and a list of three identifiers is
- [x] 4.4 Add the move: target `<YYYY-MM-DD>-<name>`, no second date where the name already carries one, refuse on collision naming the existing archive, and rename rather than copy
- [x] 4.5 Write the `VERSION` file recording the declared version. Where the write fails after the rename, report the archive location and the failed write together — the change is archived and one file is missing, and reporting that as a failed archival sends a contributor to redo something that already happened
- [x] 4.6 Add `--yes` and the interactive prompt, with no prompt where the stream is not interactive
- [x] 4.7 Extend the `--json` payload with the reports and a field stating whether the change moved
- [x] 4.8 Assert atomicity in both directions: a refused archival and a declined offer each leave the tree byte-identical, compared before and after
- [x] 4.9 Assert `the_archive_check_does_not_write_the_declaration` still passes unchanged. Archival now writes to the project and still must not touch `spec.nix`, which is exactly the obligation the withdrawal was careful to preserve
- [x] 4.11 Add `--check`, applying every condition archival applies and performing none of them. It shares the gate with archival rather than reimplementing it — two code paths deciding readiness is exactly the drift the requirement exists to prevent
- [x] 4.12 Assert `--check` three ways: ready exits zero and reports ready, refused exits nonzero with archival's own message, and would-ask exits zero while saying archival would ask. Assert in all three that the tree is byte-identical before and after
- [x] 4.10 Assert the collision refusal against an archive that already holds the target, and confirm nothing was moved

## 5. `list` and `status`, which archival now depends on

- [x] 5.1 Add `htmlspec list`, enumerating active changes, with `--specs` for accepted specifications and `--json` for both
- [x] 5.2 Exclude the archive from the active enumeration, and assert it against a project holding both
- [x] 5.3 Add `htmlspec status --change <name>`, reporting artifact presence and task completion, with `--json`
- [x] 5.4 Have archival's incomplete-work report call the same code, and assert the two agree for one change. Two counts of one thing drift, and archival disagreeing with the command a contributor just ran is the failure this prevents
- [x] 5.5 Add change selection to `archive` with no argument: one active change is selected, several are listed, none reports that there is nothing to archive

## 6. `instructions`

- [x] 6.1 Add `htmlspec instructions <stage>`, emitting the workflow text for that stage from the compiled workflow table `init` already installs from
- [x] 6.2 Assert the emitted text equals the installed text for every stage. This is the requirement that keeps one source, and a test comparing them is the only thing that keeps it true
- [x] 6.3 Report an unknown stage by naming the stages that exist

## 7. `store`, which is severable

- [x] 7.1 Settle where registrations live, and confirm the location is machine-level and outside any project
- [x] 7.2 Add registration, listing, and removal
- [x] 7.3 Add the selection flag to the operations that read and write specifications and changes
- [x] 7.4 Report an unresolvable registration as unresolvable, and assert that no operation falls back to the working directory's project. This is the failure mode worth a test: falling back means acting on a different project than the one named, and it would look like success
- [x] 7.5 If this section is cut, confirm sections 1 through 6 stand alone and that nothing in them references a registered project

## 8. The workflow instructions catch up with the command

- [x] 8.1 Rewrite `.pi/skills/htmlspec-archive-change/SKILL.md`: steps 4 through 6 are now one command, and the agent's remaining work is the specification, the version, and the merge
- [x] 8.2 Add the change-selection step the HTMLSpec prompts have never had, now that `list` exists
- [x] 8.3 State that the agent merges before running `archive`, so the assessment has something to find. An agent that archives first will be told three requirements are missing and will have to answer a prompt it caused
- [x] 8.4 Regenerate the installed instructions and confirm `instructions` emits the same text

## 9. Prove it against this repository

- [x] 9.1 Take the two active changes in `openspec/changes/` and dry-run selection, completion reporting, and the merge assessment against them
- [x] 9.2 Confirm the version gate now refuses a declaration at `0.5.0` and accepts `0.6.0`
- [x] 9.3 Update `CHANGELOG.md` and record this change's version
- [x] 9.4 Run the conformance suite and `openspec validate --all`
