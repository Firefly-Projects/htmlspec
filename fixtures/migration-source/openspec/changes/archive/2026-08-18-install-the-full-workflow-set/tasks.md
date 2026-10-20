## 1. Replace the hand-held rule with a checked one

- [x] 1.1 Expose the CLI's command tree as data: a function in `htmlspec-cli` returning the subcommand names reachable from the root, derived from the clap definition rather than a hand-written list. A hand-written list is a second place the command surface is stated, and it would drift in the direction that makes the test pass while the body is wrong
- [x] 1.2 Add the extraction: given an instruction body, find every `htmlspec` followed by words and walk the words down the command tree, returning the invocations that do not resolve. Deliberately over-eager — prose that reads like an invocation is flagged, because the other failure direction ships a skill that sends an agent to a clap error
- [x] 1.3 Add the test in `htmlspec-cli` asserting no `WORKFLOWS` body names an unresolvable invocation. This is the test that makes tasks 3 through 8 safe to write, so it lands before any body does
- [x] 1.4 Test the extractor against a body known to be bad — `htmlspec propose`, `htmlspec new spec`, and `htmlspec init <path>` — and assert each is reported. A guard with no failing case is a guard nothing has exercised
- [x] 1.5 Remove `no_body_names_a_withdrawn_command` from `htmlspec-assets`, which task 1.3 subsumes, and update the module comment in `instructions.rs` to describe the checked property instead of the one-workflow rule it currently states as a non-goal

## 2. The workflow table grows

- [x] 2.1 Add the six entries to `WORKFLOWS` in lifecycle order — `explore`, `new`, `continue`, `ff`, `apply`, `verify`, `archive` — each with its directory, summary, and allowed tools. Bodies come in tasks 3 through 8; use a placeholder that the length assertion rejects, so an entry cannot ship without its body
- [x] 2.2 Add the entry-point flag to `Workflow` and set it on `new`. A flag rather than "the first entry", because order is for reading and the entry point is what a contributor types first; a later change adding a row above `new` must not silently repoint the closing message
- [x] 2.3 Test that exactly one workflow is flagged as the entry point. Two would make the closing message pick arbitrarily, and none would make it print nothing on the run where it matters most
- [x] 2.4 Test that the table's order is lifecycle order by pinning the first and last entries, which is the property the shared-file region depends on
- [x] 2.5 Extend `workflow_names_and_directories_are_distinct` coverage to the summaries: seven descriptions appear in one list in a contributor's tool, and two workflows with the same summary are indistinguishable at the point of choosing

## 3. explore

- [x] 3.1 Write `crates/htmlspec-assets/instructions/explore.md`. A stance, not a sequence: think, read, search, investigate, and do not implement. State plainly that an agent asked to explore must not write code, and that producing an artifact is capturing thinking rather than implementing
- [x] 3.2 State why the stance is worth an instruction file: work done before anyone decided what it was for produces a change with no proposal. This body names no `htmlspec` invocation at all, which is the strongest available demonstration that the hazard is naming an absent command rather than shipping a workflow without one

## 4. new

- [x] 4.1 Write `new.md` covering the three artifacts `htmlspec new change` writes, in order: the proposal, then the design, then the tasks. Name the command with its real spelling and its real arguments
- [x] 4.2 State what a proposal carries, in the terms the accepted `artifact-kinds` contract uses. Do **not** name a diagnostic for a proposal's content: `kind/*` has exactly two rules and neither constrains a proposal or a design, deliberately — a proposal arguing for a prospective requirement is doing a proposal's job whether it states it in prose or as a unit. The body says what a good proposal does and is explicit that validation does not enforce it, because an agent told a rule exists will write to satisfy a check that never runs
- [x] 4.3 State the same for the design — decisions, alternatives rejected, risks accepted — and that a design's reasoning is where an obligation is often stated most precisely, which is why archival reads it
- [x] 4.4 State the same for the tasks document, including that a dropped task states why it was dropped, and that the reason marks a boundary the specification will later state as out of scope. Name `kind/no-tasks` here, which does exist: a tasks document carrying no task unit is reported
- [x] 4.5 State that the specification is not written here. It is produced at archival from what was built, and a specification written at proposal time is a restatement of intent — which is the failure the archival body exists to avoid

## 5. continue

- [x] 5.1 Write `continue.md`: determine which artifacts the change has, and produce the next one it is missing. Name how the change's state is established, using commands that exist
- [x] 5.2 State the stopping condition. When a change has its proposal, design, and tasks, the next step is implementation rather than another artifact, and the body says so rather than inventing a fourth document
- [x] 5.3 Defer to `new` for what each artifact carries rather than restating it. Three descriptions of a proposal is the drift the artifact-kinds change exists to prevent, and the workflow is named rather than pathed because a shared-file tool has every body in one region

## 6. ff

- [x] 6.1 Write `ff.md` as a driver: the sequence, the checkpoint after each artifact, and what makes stopping early the right outcome. Defer to `new` and `continue` by name for artifact content
- [x] 6.2 State the checkpoint explicitly — validate after each artifact, and do not carry an invalid artifact into the next one. Fast-forward means fewer pauses for a contributor, not fewer checks
- [x] 6.3 State when `ff` is the wrong choice: a change whose shape is not yet settled wants `explore` first, because carrying an unsettled idea through three artifacts produces three artifacts to rewrite

## 7. apply

- [x] 7.1 Write `apply.md`: work the tasks in the tasks document, settling each task's state as the work completes. Name the states the tasks template actually uses, including `dropped`
- [x] 7.2 State that a dropped task must record its reason before it is left dropped, and why: without the reason it records that something was abandoned and not what the decision was
- [x] 7.3 State that implementation may reveal the design was wrong, and what to do about it — say so and settle it, rather than implementing something the design does not describe. Archival reads the implementation as authoritative, so an unrecorded divergence becomes a contract nobody chose

## 8. verify

- [x] 8.1 Write `verify.md`: check the implementation against the change's own artifacts before archival. Name what is compared — tasks against what was built, design decisions against what the code does
- [x] 8.2 State what a verification failure means and what it does not. A divergence is not automatically a defect: the implementation may be right and the design stale. The body says to name which, because that judgment is the whole value of the stage
- [x] 8.3 State the boundary against archival. `verify` reports; it does not produce the specification, advance the version, or merge. Archival does those, and a workflow that half-performed the next one would leave a change in a state neither workflow describes

## 9. The closing message

- [x] 9.1 Change `report_next_command` to name the entry-point workflow from the table instead of the literal `"archive"`, keeping the grouping by distinct spelling
- [x] 9.2 Move archival into a following sentence rather than the headline, since a contributor who has just bootstrapped has nothing to archive
- [x] 9.3 Update the test that pins the closing message's content, and assert it names the entry point rather than asserting a literal string — a test pinning `/htmlspec:archive` would have to be rewritten by every change that touches the table

## 10. Verify against a real project

- [x] 10.1 Run `htmlspec init --tools all` in a scratch directory and confirm seven skills and seven commands land for each tool that takes them, and that the `AGENTS.md` region carries seven sections in lifecycle order
- [x] 10.2 Measure the `AGENTS.md` region's size with seven bodies inlined and record it as evidence. The design names an unreasonable result as a risk with a stated fallback, and the number is what decides whether the fallback is needed
- [x] 10.3 Confirm every generated file records the generating version, and that a second `init` run rewrites rather than duplicates — the existing properties, re-checked at seven rows because the loop is what changed
- [x] 10.4 Drive one small change through the installed workflows end to end in the scratch project, and record where an instruction was ambiguous or wrong. A body is prose and prose has no compiler; using it is the only check that exists
- [x] 10.5 Regenerate this repository's own `.claude/`, `.codex/`, `.cursor/`, and `.pi/` instructions through `htmlspec init`, so the repository carries what the tool installs rather than a hand-copied subset
