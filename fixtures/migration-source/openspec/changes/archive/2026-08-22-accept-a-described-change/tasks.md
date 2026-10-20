## 1. Reproduce the refusal before changing the guidance

- [x] 1.1 Record the transcript as evidence: the request, both refusals, and the agent's own reasoning that an absence of capability is a legitimate need. That reasoning is the finding — the instruction overrode a correct inference rather than failing to inform one
- [x] 1.2 Quote the two sentences that produced it, by file and line: `new.md:35` narrowing a need to a problem with three examples that all presuppose an existing system, and `new.md:14` stating the naming rule without saying who may satisfy it
- [x] 1.3 Confirm against the contract that the narrowing is the instruction's own. `openspec/specs/artifact-kinds/spec.md:36` gives a proposal's intended content as "Value, need, scope, alternatives" — need, not problem. Without this the change is a preference rather than a defect report
- [x] 1.4 Confirm no test asserts anything about this text, so the absence of a failing build is expected rather than evidence the guidance is fine

## 2. A need may be an absence

- [x] 2.1 Amend the proposal section of `crates/htmlspec-assets/instructions/new.md` to state that a need is either a problem with an existing system or a capability the project does not have
- [x] 2.2 Give the absence form three concrete questions: who or what wants the capability, what they cannot do without it, and what working looks like the first time you see it. A permission with no form is what produces "we need better data handling"
- [x] 2.3 Keep the problem form and its three examples — a measurement, an error message, a behavior somebody hit. They are correct for the case they were written for and this change does not displace them
- [x] 2.4 Keep the warning against stating a need as a category of concern. That warning is the correct part of the current sentence and it must survive, or the change trades a false refusal for accepted vagueness
- [x] 2.5 Note in the file that "what working looks like" is the input the tasks are broken from and what archival checks requirements against, so a reader knows why it is asked for rather than deleting it as ceremony

## 3. The agent proposes the name

- [x] 3.1 State in `new.md` that where a contributor describes the work without naming it, the agent derives a kebab-case name from the description, states it, and proceeds
- [x] 3.2 Leave the naming rule itself unchanged. It is a good rule and it applies identically to a derived name
- [x] 3.3 Require the derived name to appear in the report, so a contributor can correct it. A derived name they never see is a decision taken on their behalf
- [x] 3.4 Do not add a confirmation step before creating the change. A round trip to approve a name that follows from the description is the same refusal in a politer form

## 4. A description is enough to start

- [x] 4.1 State that the workflow asks only for what is genuinely missing, and infers what the description already carries. In the recorded transcript two of the four requested items were already supplied
- [x] 4.2 Amend `ff.md` so an absence of capability does not read as an unsettled idea routed to `explore`. `ff.md:13` sends unsettled work to `explore` and that routing is right; new work whose shape is described is not unsettled
- [x] 4.3 Confirm `ff.md:35` needs no repair — it already says "the value, the need, the scope", which is the contract's wording
- [x] 4.4 Confirm `explore.md` is untouched. Its stance is correct and this change does not reach it
- [x] 4.5 Confirm the guidance still asks, rather than invents, when a request carries no description at all. A bare `/htmlspec-new` is the case the current behavior gets right

## 5. Check it against the case that failed

- [x] 5.1 Rebuild and reinstall, then run the recorded tally-counter request through the `new` workflow in a scratch project
- [~] 5.2 **Dropped.** Confirm a change is created on the first exchange, with a derived name stated in the report and no request for a problem. The run was made and is recorded in `evidence/described-change-run-2026-08-22.md`. It establishes that the guidance no longer refuses an absence of capability — the agent named the second form of need, declined to invent one, and disclaimed the round trip to approve a name — but no change was created, because `/htmlspec-new <description>` discards the description. `write_commands` in `crates/htmlspec-cli/src/install.rs:124` emits no argument placeholder and no instruction body contains `$ARGUMENTS`, so the agent received the instructions alone and asking was the behavior task 4.5 preserved. This task's subject is unobservable through the command surface until the surface passes the description through, and the repair is codegen owned by `project-bootstrap` rather than prose owned by this change. Filed separately. The boundary is worth stating at archival: a prose repair to authoring guidance cannot be verified end-to-end while the invocation withholds the contributor's input
- [x] 5.3 Confirm the proposal states the need as an absence and answers the three questions, rather than asserting a problem that does not exist
- [x] 5.4 Confirm a bare `/htmlspec-new` with no description still asks rather than inventing, so the repair did not remove the behavior that was correct
- [x] 5.5 Record both transcripts as evidence. The check for a guidance change is a transcript, since no test asserts this text. `evidence/refusal-transcript-2026-08-21.md` records the failure, `evidence/amended-guidance-2026-08-21.md` what an agent now reads, and `evidence/described-change-run-2026-08-22.md` the run against the amended text
- [x] 5.6 Run `cargo test` and `./scripts/check` and confirm nothing regressed. The instruction bodies are embedded in the binary and installed by `init`, so the generated-file tests read the amended text

## 6. The contract

- [x] 6.1 Add to `openspec/specs/artifact-kinds/spec.md` the obligation that a need may be an absence of capability, and that authoring guidance must not narrow a need to a defect
- [x] 6.2 Add a scenario for a proposal whose need is an absence, and one for a workflow that receives a described change without a name
- [x] 6.3 Apply it as a partial update, leaving every requirement and scenario this change does not name intact
- [x] 6.4 Confirm no other accepted requirement is touched, and run `openspec validate --all`
