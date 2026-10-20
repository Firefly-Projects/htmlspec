## 1. Establish that the operation is absent before adding it

- [x] 1.1 Record as evidence that archival's message names a step with no command behind it: `crates/htmlspec-cli/src/commands.rs:1352` says "Apply the change's contract before archiving", and the subcommand list in `crates/htmlspec-cli/src/main.rs` holds `validate`, `format`, `assets`, `archive`, `init`, `list`, `status`, `store`, `instructions`, `new` and nothing that writes or reports on the accepted specifications
- [x] 1.2 Record that OpenSpec provides the operation and this repository already carries its workflow: `.pi/skills/openspec-sync-specs/SKILL.md` exists, and `.pi/skills/` holds seven `htmlspec-*` skills with no sync among them. This is what makes the naming a contract question rather than a preference
- [x] 1.3 Confirm against the accepted text that no requirement already covers this. `Archival assesses the merge and never performs it` excludes the operation from archival without assigning it anywhere; `The command surface names its operations as OpenSpec names them` fixes eight spellings and does not name `sync`. Quote both, so the change is shown to fill a hole rather than to reopen a settled decision
- [x] 1.4 Confirm the merge is reserved for an agent by reading `artifact-kinds` rather than by citing the archival comment that cites it. A design that derives its central constraint from a code comment has derived it from a summary

## 2. The contract states the operation and its exclusions

- [x] 2.1 Add a requirement to `openspec/specs/toolchain-packaging/spec.md` stating that the command surface provides an operation that reports whether a change's stated requirements have reached the accepted specifications
- [x] 2.2 State that the operation compares requirement identifiers and reads neither side for meaning, in the same terms `Archival assesses the merge and never performs it` already uses, so the two requirements cannot drift into different accounts of one comparison
- [x] 2.3 State the exclusion explicitly: the operation SHALL NOT modify an accepted specification and SHALL NOT propose the text of a merge. The name is inherited from OpenSpec and reads like an operation that writes, so the prohibition must be stated rather than left to be inferred
- [x] 2.4 State that the report distinguishes the requirements the change states, those the accepted specifications hold, and those absent from them, and that it is available as a structured payload
- [x] 2.5 State that an absent requirement is a finding and not an error, and that the operation succeeds while any remain. Give the reason: a change mid-implementation legitimately has absent requirements, and a withdrawn one is absent permanently
- [x] 2.6 State what it does refuse on — a change that cannot be resolved, an absent specification artifact, a specification that does not validate — with the reason that a comparison against an invalid contract has no meaning
- [x] 2.7 State that the operation and archival derive the comparison from one source, in the form `The command surface reports a change's completion` already uses for tasks. This is the requirement that prevents a contributor syncing clean and archival reporting unmerged requirements immediately after
- [x] 2.8 Modify `The command surface names its operations as OpenSpec names them` to carry `sync` in its fixed list of spellings, and leave the rest of the requirement as written
- [x] 2.9 State that the workflow instructions carry a stage for the operation, so the agent-driven half of the merge is written down where an agent reads it
- [x] 2.10 Run `openspec validate --all`

## 3. Extract the comparison so there is one of it

- [x] 3.1 Lift `unmerged_requirements` (`crates/htmlspec-cli/src/commands.rs:1290`) into a form both archival and sync call, returning the stated, accepted, and absent sets rather than only the absent one
- [x] 3.2 Leave archival's behavior unchanged. It reports absent identifiers as a finding and continues to; this change adds a caller, not a rule
- [x] 3.3 Add a test that the two callers agree, in the shape of the accepted scenario "Archival and the report agree" — a change with a known absent identifier, asserted through both paths

## 4. The subcommand

- [x] 4.1 Add the `sync` subcommand taking a change, with `--store` for the same reason every project-reading operation takes it
- [x] 4.2 Emit the human report: the three sets, each identifier named rather than counted, following `report_findings`' stated reason that a contributor can check three identifiers and cannot check "3 requirements"
- [x] 4.3 Emit the structured payload with the three sets as arrays
- [x] 4.4 Return the success exit code with a non-empty absent set, and the operational-failure code for an unresolvable change, an absent specification, or one that does not validate
- [x] 4.5 ~~Add the subcommand to `invocations.rs`, whose hand-written list exists so the command surface is stated twice and checked against itself~~ **No action: the premise was wrong.** `invocations.rs` resolves each invocation against clap's own command tree, and its module comment gives the reason — "A hand-written list states the command surface a second time, and it drifts in the direction that keeps this check passing while the instructions are wrong." There is no list to add to, and `sync` resolved the moment the subcommand existed. `no_shipped_body_names_an_absent_command` passes over `sync.md`, which is the property this task was reaching for
- [x] 4.6 Confirm `help_documents_every_subcommand_and_option` covers the new subcommand and its option

## 5. The workflow stage

- [x] 5.1 Add a `sync` workflow to `crates/htmlspec-assets/src/instructions.rs`, so `htmlspec instructions sync` and the installed copy come from one source
- [x] 5.2 Write the stage's instructions: run sync, read the change's specification and the accepted specifications on both sides, merge each absent requirement into the specification that should hold it, never copy a change's specification wholesale into an accepted one, re-run sync to confirm the absent set is empty
- [x] 5.3 State in the instructions that a requirement deliberately withdrawn is left absent and named in the report to the contributor, not merged to make the list go quiet
- [x] 5.4 Confirm `init` installs the new stage for every supported tool, and that re-running `init` on this repository adds it without disturbing the existing seven

## 6. Prove it against this repository

- [x] 6.1 Run the operation against an archived change replicated as an active one, and confirm the absent set is empty for a change whose merge is known to have happened
- [x] 6.2 Run it against a change with a requirement identifier that reaches no accepted specification, and confirm the identifier is named and the exit code is success
- [x] 6.3 Confirm no accepted specification changed on disk across every run above, which is the exclusion in 2.3 checked rather than asserted
- [x] 6.4 Add the changelog entry and advance the version
