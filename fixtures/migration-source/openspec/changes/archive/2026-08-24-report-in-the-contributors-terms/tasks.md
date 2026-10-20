## 1. Establish the defect across all seven bodies before changing any

- [x] 1.1 Record the received report as evidence, with the contributor's own reaction. The complaint is the finding: a described app answered with a task order, a warning count, and two questions about scope nobody asked for
- [x] 1.2 Quote `new.md`'s five report bullets in full and name the subject of each — a directory, an argument, decisions, an order, open questions. Shown against the complete list rather than asserted, so the report is demonstrably obedient rather than careless
- [x] 1.3 Count artifact-subjected lines in all seven `## What to report` sections and record the table. `apply.md` at six is worse than `new.md` at five, which is what makes this a class rather than one body's fault
- [x] 1.4 Confirm no body asks for a gloss: `grep -ril "gloss\|plain words"` across `crates/htmlspec-assets/instructions/` returns nothing
- [x] 1.5 Apply the deletion test to the received report's task list and record that it collapses. Also record the one part that was already correct — the warning kind was named — so the rule is shown not to fire on prose that satisfies it
- [x] 1.6 Confirm no accepted requirement governs a report at all, so the absence of a failing check is expected. Put this first because it explains why seven independently-written sections were never reconciled

## 2. The contract states both obligations, once

- [x] 2.1 Add a `workflow-reporting` capability at `openspec/specs/workflow-reporting/spec.md`, holding both obligations for every workflow
- [x] 2.2 State the register obligation: a report's first claim is stated in the vocabulary of what the contributor asked for, rather than the vocabulary of the artifacts that hold it
- [x] 2.3 State it as register rather than as a sequence of slots. A two-slot shape breaks on `explore`, which produces no artifact and changes no file, and would force "nothing changed, nothing was written"
- [x] 2.4 State the gloss obligation with the deletion test that bounds it. Imported without the test it fires on `htmlspec validate` and `--json`, which name their own referents
- [x] 2.5 State that a report surfaces what the artifacts already say in the contributor's terms rather than summarizing them. An agent asked to summarize invents; an agent asked to surface quotes
- [x] 2.6 State that a report's first claim is a **fact**, so the register obligation cannot be read as license for reassurance. Without this, "Great news, your tally app is coming along nicely" satisfies the letter
- [x] 2.7 State that a decision an agent can settle is settled and reported, not delegated. Scope the contributor did not ask for is offered for removal rather than presented as a question
- [x] 2.8 Add scenarios: a described app reported in artifact terms, a report whose pointers collapse under the deletion test, a self-describing pointer that owes no gloss, a workflow that produces no artifact, and a decision the agent settled itself
- [x] 2.9 Do not extend `artifact-kinds`. It governs documents that are written, validated, and archived; a report is none of those, and merging the two makes one capability about two things
- [x] 2.10 Run `openspec validate --all`

## 3. Each body conforms to the one statement

- [x] 3.1 Amend `new.md`'s report section so the first item is what the contributor is getting, drawn from the proposal's "what working looks like", with the change directory and the next command demoted to one closing line
- [x] 3.2 Amend `ff.md` the same way. It carries the same subject as `new` and its report currently opens on a judgment about workflow fit
- [x] 3.3 Amend `apply.md`, which is the worst of the seven. Its first item becomes what now works and what a contributor can try, with task states and check output following
- [x] 3.4 Amend `verify.md` so it opens with whether the change is sound. Its per-task and per-decision accounting is a pointer list by construction, so the gloss obligation does most of the work here
- [x] 3.5 Amend `continue.md` so it opens with what the change can now do that it could not before, rather than with which artifact was missing
- [x] 3.6 Amend `archive.md` so it opens with what the project now guarantees, rather than with the specification file it produced
- [x] 3.7 Amend `explore.md` least. Its report is already nearest the contributor's terms — the question and the answer — and it needs the gloss obligation more than the register one
- [x] 3.8 Keep every artifact fact that a contributor sometimes needs. This change moves them from subject to annotation and from opening to close; it deletes none
- [x] 3.9 Change nothing outside each body's `## What to report` section. The guidance on writing a proposal, a design, or a specification is settled and re-opening it would re-litigate two archived changes

## 4. What can be checked, and what cannot

- [x] 4.1 Add a test asserting every body has a `## What to report` section, so a workflow added later cannot ship without one
- [x] 4.2 Confirm the test fails when the section is removed from a body, so its subject is proven rather than assumed
- [x] 4.3 State plainly in the tasks and the changelog that the register obligation is **not** mechanically checkable. Whether a first claim is in the contributor's vocabulary is a judgment, and the check is a transcript — the same limit the 0.3.0 and 0.4.0 guidance repairs recorded
- [x] 4.4 Do not write a test that greps for banned words in a report. A report is not a file this program writes, so there is nothing to assert against, and a proxy check would pass while the register stayed wrong
- [x] 4.5 Run `cargo test` and `./scripts/check`

## 5. Check it against the case that failed

- [x] 5.1 Rebuild, reinstall, and confirm the amended report sections are what an agent reads. The bodies are compiled into the binary
- [x] 5.2 Re-run the tally-mark request in a scratch project and confirm the report opens with what the contributor is getting rather than a task order
- [x] 5.3 Confirm every pointer in that report is glossed on first use, and that the ten task identifiers either name what they are or do not appear
- [x] 5.4 Confirm the report still names the boundary — documents exist, the app does not yet, `apply` builds it — because a report that reads warmly and hides that is a worse failure than the one under repair
- [x] 5.5 Confirm no artifact fact a contributor needs was lost in the reordering
- [x] 5.6 Run one other workflow's report — `apply` on a real change — and confirm the register holds where the subject is different. One workflow conforming proves the wording, not the obligation
- [x] 5.7 Record both reports as evidence, before and after, since the check for this defect class is a transcript

## 6. Release

- [x] 6.1 Regenerate this repository's 35 installed files, so the project reads its own amended guidance
- [x] 6.2 Advance the workspace version and update `Cargo.lock` offline
- [x] 6.3 Write the changelog entry, naming the contributor's complaint as the source and stating that the register obligation has no mechanical check
- [x] 6.4 Record the version in the archived change's `VERSION` file at archival, and confirm `scripts/check-release` passes
