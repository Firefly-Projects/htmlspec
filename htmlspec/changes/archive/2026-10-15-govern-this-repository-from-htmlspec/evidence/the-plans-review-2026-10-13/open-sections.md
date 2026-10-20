# 2026-08-10-html-spec
- **The packet's note and `deviation_table` disagree on task 6.9.** The note says "done with no deviation", and the table says "A: another way". I followed the table, as the brief directs. The record supports it: `browser-verification.md` verified a server-less `file://` run rather than a machine with networking disabled, and it records lasting fixes beyond the task's words.
- **The `parse-errors.md` finding 3 question is open** (the short-doctype rule was never built). I carried it as written. It touches only the parse-errors evidence summary, which says a legacy doctype is invisible to the parse-error rule and claims no doctype rule.
- **`verify-no-request-leaves-the-machine` rests on a local server's access log.** The record calls that log the complete set of requests. A reviewer may judge that a local log cannot see requests to other origins. If so, delete the unit, and the warning stands.
- **`deterministic-formatting.html` overstates its CLI tests on line endings.** A property test, `every_formatted_document_is_normalized`, does assert that no CR survives, but it is not the suite the record cites, and a test in code is not a record.
- *Warnings can be made fatal* is in tension with *An upstream edit marks dependents…*. That is a contract question already listed in the plan's "Found while reading" for the accepted document-validation, and I added nothing beyond the class-A note.

# 2026-08-13-content-address-runtime-assets
- **Read first: deviation 1's disagreement with group-8 §8.4.** The record says the original defect "fails the suite" through `format --check`. At `86c33f4` the suite format-checked `fixtures/dogfood` only.
- **Packet note against brief (risks):** the packet says each design risk's mitigation "keeps a plain link to its task". The author's words never name a task, and the brief's "Add no link the source did not write" wins, so no link was added.
- **Packet wording (group-8):** the packet says "removing three copies including the change's own interim one". The store this change added was not removed. My note says the 18.1 MB counts three copies, one of them the new store, kept on both sides.
- **Init writes the runtime beside each new document (D).**
  - At `86c33f4`, a document initialized in another directory of the repository gets a second copy, unless `--assets` is given.
  - 2026-08-17-project-bootstrap-cli later measured two changes at 6.0 MB each, but called it "not a defect in the pieces".
  - I read the delta as defining no project root and wrote nothing. A reviewer may judge it a B against "A project can be initialized with a conforming document".

# 2026-08-14-separate-artifact-kinds
1. **Read first: the packet note against the brief on the "6 MB" link.** The packet's decided note for the design's "The archived change keeps all four documents" says to link "the 6 MB shared asset store the previous change just deduplicated" to the previous change's evidence. The brief (review 2026-10-13) forbids "a link from an uncited figure to another change's evidence", and the brief wins, so the phrase carries **no link**. The note names convention 3, but the phrase is not a source path.
2. **Design quotes a diagnostic the build does not print. I left it unnoted.** "no kind declared" appears three times (decision "`kind` is required", decision "Migration rewrites…", Risks). The `9c3a3c7` commit message records the built failure as "kind is a required property". The decision itself (required, undefaulted) was built, and the record is only a commit message, so I added no note. A reviewer may want one.
3. **A later change narrowed the boundary in "Archival synthesis is a skill; the program only checks". I left it unnoted.** `2026-08-30-finish-the-archival` moved the directory move into the program. Its design calls that "a narrowing … rather than a reversal" and says the artifact-kinds argument is untouched.
4. **The verification for *A tasks document carrying other units* rests on a test that runs the kind rules alone.** I recorded it as `passed`, because the scenario is about the completeness rule. A stricter reading would make it `partial`.

# 2026-08-17-project-bootstrap-cli
**Read first: the Verification section of `specification.html`.** It holds 23 units, and it is the largest body of migration judgment here. Two passes rest on a record's statement rather than on output it shows:
- *Installing for a tool without commands*: `first-run.md` says "Codex correctly received skills and no command file".
- *Creating a change*: "each declaring its kind" is taken from `validate` reporting 0 errors, which an undeclared kind would fail.

A stricter reading would make both partial.

**The brief overrode packet notes (brief lines 95–96):**
- The proposal question on Was/Becomes asks for a note linking `first-run.md` for the built spelling. Brief lines 645–647 say to write no note on a proposal whose plan the build departed from, so I wrote none. Task 3.1's deviation carries the built spelling.
- The Capabilities question says a note "may" name the omitted document-validation delta. I wrote none, for the same reason and to keep my own words to what the conventions call for.

**The design aside on "The version gate checks; it does not write".** The brief's two cases are a departure the author never corrected and a reversal by a later change. This is neither exactly: a later change fixed a defect in the build. I applied the departure rule, because the decision's first sentence describes a comparison that never happened. A reviewer may judge that the notes on task 9.1 and on the specification are enough without it.

**Decision-unit boundaries.** Two rationale paragraphs come close to stating decision:
- ¶2 of *Resolution walks upward…* opens "Stopping at `.git` is the decision worth stating", and the rest of the paragraph is its reasoning.
- ¶2 of *An ascending asset reference…* ends "so this change closes it".

I read both as reasons, and the "Rejected:" paragraphs as rationale.

**Packet facts:** none looked wrong.

# 2026-08-18-conform-generated-frontmatter-names
- **A reviewer should read this first: the class A note on *Every generated name is loadable*, with the design's decision-3 aside.** The reader left this to review. The alternative was class D, on the reading that a file with no frontmatter declares no name for the rule to constrain. I chose A so the reading matches the successor change's triage and draft.
- **Tasks 4.1 and 5.2 as deviations.** The packet decides 5.2's correction is its deviation in the author's words. `deviation_table` has no row for either task. 4.1 has the same shape, an in-text correction of the task's figure, so I treated it the same way. That required the mid-sentence split above. If a reviewer reads 4.1 as only correcting a claim (class C, like 1.2 and 1.3), it becomes heading = the full first sentence, plus a note paragraph.
- **Packet fact that looks slightly wrong.** For 5.2 the packet says "Its diff figure still has no record but the commit message". But commit `4972234`'s own diff shows `openspec/specs/project-bootstrap/spec.md` with 20 insertions and 0 deletions, which is the source at a named commit. I wrote no note on the figure either way.
- **Minor.** The reader's format note says groups 1 to 5 hold 20 tasks, but the source has 19.

# 2026-08-18-install-the-full-workflow-set
**Read first: deviation 1, the class E note on *Instructions naming an absent command*.** The reader asked the coordinator whether this is a difference at all. I kept the delta's words, because the design's "prefix" wording and task 1.4 pull in opposite directions. If a reviewer reads design decision 1 as deciding the narrower check, the class becomes A, and the note's wording would change but the delta's words would stay.

- **Task 2.4's deviation is not in the `deviation_table`.** I followed the packet's mapped question, which says 2.4 states it and leaves the attribution to the drafter. The table is silent on 2.4, so there is no conflict and the checker raises nothing. If the reviewer prefers that the region test stay outside the change, as the reader's top-level question suggested, delete that paragraph.
- **Task 1.2: no deviation.** The packet's slice question says the `htmlspec --version` fix "can be" 1.2's deviation. Under the brief's threshold it is not a different result. So the fix now appears in no migrated document, only in `invocations.rs` and the `5f7e5cf` message, which also claims "both recorded in evidence" when the evidence records only one.
- **The brief overrode a packet note.** The decided proposal question says "the artifact-kinds change" should link to `2026-08-14-separate-artifact-kinds`. The brief's Links section forbids "a link from … a change's name in prose", and that phrase is a name, not a cited path, so I wrote no link. The `openspec/specs/artifact-kinds` path is linked, as both agree. The same phrase in design decision 3 and task 5.3 is also unlinked.
- **`verify-what-each-tool-received` is `passed`.** The record counts files "each tool received" but does not name their directories. I read per-tool attribution as showing "written where that tool reads them". If a reviewer disagrees, it should become `partial`, with that clause as `unchecked`.

# 2026-08-21-check-release-hygiene
- **The tension the reader raised.** The built check fails if `Cargo.toml` is bumped before the changelog entry is written. That is a working-tree fact, and toolchain-packaging's "A rule whose violation is a legitimate state for work in progress SHALL therefore be stated as a property of a settled record" is arguably at odds with it. I carried it as class C because the design treats a bump as an archiving act. A reviewer may judge it a contradiction, which would make it class E or B. The note in the specification does not decide this.
- **The packet's open contract questions.** Neither touched a document beyond carrying the source as written.
  - Whether toolchain-packaging's "distinguishable … satisfying one SHALL NOT be taken to satisfy the other" still holds. The proposal, the design, task 2.5 and the evidence keep the "`VERSION` is this repository's convention" premise as written.
  - The boundary task 2.5 asked to state, which the delta never states. Task 2.5's reason is verbatim, and nothing fills the silence.
- **Read first: the partial verification of *The baseline*.** It rests on the record's explanatory sentence, from a run designed to test the changelog rule. If a reviewer reads that as a side remark rather than a recorded outcome, it should be removed. The other four verifications are unaffected.

# 2026-08-21-distinguish-command-names-from-skill-names
- **Task 5.2: the packet note and the deviation table disagree.** The packet's decided note says tasks 5.1–5.5 state no deviation. The `deviation_table` says task 5.2 is "A: more" and owes one. The record supports the table: the delta, the `c864727` message ("replaced by two"), and task 5.4's twelve scenarios becoming thirteen. I followed the table, as the brief directs.
- **Task 5.3 states no deviation, although the record shows a result beyond its words.** The requirement gained paragraph 11 ("No two files the operation writes for one tool SHALL declare the same name…"). No task's words mention it, and 5.3 says only "keeps … and drops the sentence". The packet note says 5.3 states no deviation, and the table does not list it, so I followed the packet. I put no note there either, because convention 2 says not to carry a result difference as a note. A reviewer may want a deviation on 5.3.
- **The packet's open question on `loading-after-omission`, and where it touched these documents.** The closure links in tasks 4.2 and 4.3, the evidence unit, and the partial verification all point at the file as a whole. None points at its appended "The load, observed 2026-08-21" section, because the Markdown record carries no anchor ids of its own.
- **The packet's "why" for the removed scenario says "the source states no reason".** The proposal's What Changes and the design do state one. My note says the *delta* states none and points to those two.
- **The verification method says "a session load", as the record does.** The 2026-08-21 section does not name which harness ran that load.

# 2026-08-22-accept-a-described-change
1. **I departed from a packet note: the `spec.md:36` links.** In the proposal's Why and in task 1.3,
   `openspec/specs/artifact-kinds/spec.md:36` links to `../../../specs/artifact-kinds/spec.html` as a
   whole. The packet's slice decision says to link `#req-intended-content-is-guidance-and-enforced-structure-is-validity`
   with no `data-refs`.
   - The brief says "Link a requirement by its identifier only where the author's words name it …
     Never link a requirement you inferred", and the brief wins.
   - With the fragment, the checker reported task 1.3 as linking to a requirement it reaches with no
     `data-refs`, which the packet itself rules out.
   - Line 36 is that requirement's table row at `afc2bdc`. If a reviewer prefers the packet, they can
     restore the fragment and accept that finding.
2. **I departed from a packet note: task 5.3.** It has a note, not a deviation. The `deviation_table`
   (class B) wins over the packet note unless the record shows otherwise. `described-change-run`
   shows that no proposal existed, but it never says what was done for 5.3, so a deviation would
   state an inference.
3. **A sentence the delta does not carry.** Task 5.2 says "The boundary is worth stating at
   archival", and the delta does not state that boundary. It is carried only in task 5.2's reason.
   This is a contract question for a later change, and I added nothing to the specification for it.

# 2026-08-24-pass-the-description-through
**Read first: deviation 1, the shared file's region.** `write_region` writes every workflow body into `AGENTS.md`. Since this change, each body opens with `## What you were asked` and `$ARGUMENTS`, so the region carries seven unsubstituted placeholders. That is so at `4f2f3e7` and at `139b206`.
- I classed it **C**. On its words, "This obligation … SHALL NOT reach a kind that carries no invocation" scopes the obligation and does not forbid the token. The scenario's "no argument is declared" holds.
- A reviewer who reads that sentence as keeping the argument machinery out of the region would class it **B**. The note would then call it a defect still in the product.
- The design's own risk section counts a literal `$ARGUMENTS` in a body as "a legible artifact", but only for a tool that does not substitute.

**Two judgment calls worth checking:**
- **Task 7.3's reference.** "Add a scenario for an invocation carrying a description" uses the heading *An invocation carrying a description* word for word. I linked it and gave it `data-refs`. The task's other two scenarios are described in words that are not their headings, so they have no link.
- **The two declined verifications** in section 5. A reviewer who reads the-checks-failing's per-half reporting as reaching *A dropped argument is caught before release* would want a `partial` verification there instead of none.

No packet fact looked wrong. The brief and the packet notes did not disagree.

# 2026-08-24-report-in-the-contributors-terms
**Read first: task 5.6's deviation.** I found it myself, and nothing in the packet asks for it. The archive transcript is a kept record that no task asked for, and the record measures it against section 5's "one other workflow" (5.6). If a reviewer reads it as separate work, not beyond 5.6, the deviation should come out. The link to the archive report would then rest on the design and the specification alone.

- **Packet notes disagree on 4.1.** The decided note lists 4.1 among tasks that "stay `done` with no deviation". The mapped note gives 4.1 a deviation for the second test. I followed the mapped note and the brief, because a test kept in the suite beyond the task's words is a result. Brief lines 95–96 make the brief win.
- **Risk response.** The design's `### Risk: an agent that pads…` says "Accepted, and mitigated by one sentence in the contract". The closing list says "Mitigated above". I declared `accepted` with a mitigation paragraph. The subsection is the settlement the list points back to, and "accepted with a partial measure" is what the destination's value means.
- **The new transcript's first line**, "All three documents validate clean.", states a check result before the Cardinals sequence. The record calls it "a one-line preamble". I did not judge it, so the two affected verifications leave "what a check reported" unchecked.
- **The design's third requirement has no rationale.** *An agent settles what it can and reports the decision* has none in the design (packet note). It is carried as written, with nothing supplied.
- **Open packet questions:**
  - Archive-report placement: answered as described above.
  - The arithmetic boundary: it appears only in that record's evidence summary, as the record's argument. No requirement states it, and the contract is silent on it.
- **Packet facts:** none looked wrong. The reader's task count (36) is wrong; the packet does not carry it.

# 2026-08-30-finish-the-archival
**Read first: task 4.8. Two packet facts disagree, and I followed the deviation table.**
- The `deviation_table` measures 4.8 "A: another way" and owes a deviation.
- The note on the packet's question about `cf11132`'s commit message and the CHANGELOG says "4.8 still compares the tree before and after", so no deviation is owed.
- The code at `cf11132` shows the declined-offer direction is never tree-compared: only `a_declined_offer_stops` covers it, as a pure function.
- The deviation names the source at `cf11132`, not the CHANGELOG, so the note's "take in nothing from … the CHANGELOG" holds.

**The brief overrode a packet note (brief lines 95–96).** The packet's decided question on the proposal asks for a note on the proposal's "What working looks like". Brief lines 626–629 say to write no note on a proposal whose plan the build departed from. I wrote none.

**Questions that touched my documents**, all now `decided` in the revised packet:
- decision status: I wrote `accepted` units only where the shape fits;
- apostrophe identifiers: I used the packet's `…-a-change-s-…`, `scenario-a-project-s-first-archival` and the like;
- the produces/never-performs tension: carried as written.

**A contract question (deviation 6).** Does "Under structured output archival SHALL emit its reports and SHALL state whether the change was moved" cover a refusal? If it does, every structured refusal deviates, from `cf11132` to `139b206`.

**References a reviewer should check.**
- I gave `data-refs` to 2.1, 2.3, 2.5, 2.7 and 2.12, whose words give or plainly are the heading. In 2.5 the heading's words carry an inserted "by comparing identifiers". In 2.12 the words are the heading without "Archival supports".
- I gave none to 2.13. The packet's question on tasks 2.1–2.13 mentions 2.13's renamed requirement, but its words ("the requirement that enumerated four subcommands") describe the requirement, not its heading, and the task renames it (a unit the work touched).

**Packet facts:** none looked wrong.

# 2026-08-31-sync-a-change-into-the-accepted-contract
- **Read first: the 5.4 deviation's link to another change's carried evidence** (`../2026-09-06-distinguish-a-requirement-from-an-example/evidence/…`). The brief's link table lists other changes' documents, not their evidence. I used "the record a migration note names, including a later change where the note says that change fixed a defect". The file exists under S.
- **Contract question: a removed requirement counts as stated.** At `ee4301c`, `merge_state` takes every requirement in the change's specification, including a `removed` one. A removal already applied is reported absent, and one not applied is reported reached. The delta's "the requirements its specification states" does not settle this. At `139b206`, `merge_state` counts an absent withdrawal as reached, so a later change addressed it.
- **Contract silence: an unreadable accepted specification.** At `ee4301c`, `merge_state` skips an accepted specification it cannot read and treats a failed discovery as holding nothing, so those requirements read as absent. The delta's refusals cover only the change's own specification.
- **Finding 6** (the human report gives stated as a count) is D on my reading. It is the one place a reviewer could reasonably want a deviation on task 4.2.
- No packet fact looked wrong.

# 2026-09-02-carry-task-progress-into-the-listing
- **For a reviewer to read first:** the class-A note on "The enumeration and the completion report agree", together with its `passed` verification. The tension is inside the requirement. "A change whose tasks document is absent SHALL be enumerated with its completion absent", while the scenario says "GIVEN any active change … the two report the same counts", and the completion report gives 0 for such a change. The change's record accepted this. Whether the contract should say so is a contract question, which I did not settle in the document.
- **Task 6.6** ("Confirm against a real project…") is `done` with no deviation. The record's run used a change named `unsynced-change` holding `task-example` and `req-example`, which looks like a scratch project. The task asks for "a real project", not this repository, so I found no departure from its words.
- No packet fact looked wrong. Every packet question was `decided` or `mapped`, none `open`, and I followed each note.

# 2026-09-02-state-a-changes-place-in-a-sequence
**Read first: the packet's open question on naming `FOLLOWS`.** I carried the source: neither the specification's summary nor any requirement names the file. It is named:
- in the design note;
- in task 3.1's deviation;
- in the evidence summaries of `where-the-declaration-goes` and `prove-sequence.py`, which describe what those records hold.

A reviewer deciding the question should check that last placement.

Other points a reviewer may want to confirm:
- **Deviation 6 (class E).** The delta says "Two or more changes that each follow the other SHALL be reported", and the built walk ignores a cycle through an archived change. A reviewer may read it as D.
- **`partial` on *A change archived before its predecessor*.** This rests on reading `--yes` as an answer given in advance, not "an affirmative answer" to a question that was put.
- **Packet facts.** None looked wrong. The brief and the packet notes did not disagree.

# 2026-09-06-distinguish-a-requirement-from-an-example
- **Read first: task 3.6, and the one unclearable checker finding.**
  - The `deviation_table` measures 3.6 "C: corrects a claim", which owes no deviation.
  - The source text carries "**Amended during implementation.**". The brief says such text "is the deviation whatever its kind", and `check_task_vocabulary` demands a deviation for it.
  - The two checks cannot both pass. I followed the record, as the brief allows ("unless the record itself … shows otherwise"): the amendment is 3.6's deviation.
  - The table's class C fits the first attempt's note, which corrected the "iterating nothing" claim. I carried that correction too, as a `Migration note:`.
- **Packet fact overridden by the brief.** The design-risk question (mapped) says the "Mitigated" risk becomes a unit. It cannot without splitting its only paragraph, so it stays prose (section 4).
- **Task 6.4's scenario reference.** It rests on the author's phrase matching the scenario heading verbatim, inside "the failure requirement 2 of this change exists to prevent". A reviewer may judge that a description rather than a naming.
- **Verifications of agent behaviour.** Four of the ten verifications rest on readings of one or two agent runs. The records themselves state "one run of one task" and that the control run cannot separate contract from guidance. The first verification's method repeats the first limit, and the `the-guidance-changed` evidence unit states both.

# 2026-09-11-migrate-an-openspec-project
1. **`task-5-9` deviation against the deviation table** (C: later review, owes none). The task's own note says its instruction "is not satisfiable" and that the obligation moved into the promotion gate. The brief says to follow the record over the table when the record shows otherwise, so I kept the deviation. The checker reports it.
2. **Contract tension.** The first paragraph of "A source distinction…" says SHALL NOT map onto the nearest value. The design, the audit, and the run used `modified` with prose for renames.
3. **The design's direction words are wrong in four places.** For example, the design says "above" for a section that is below. I carried them as written with no note, because no convention asks for one.
4. **Task 1.1's note repeats the false `.openspec.yaml` "disagrees" claim.** I added no note there. The decided question names only the design, 8.2, and the audit.
5. **Deviations beyond the packet's list.** I added 5.4, 5.8, and 6.3 because their own records say the work was done another way. I added none for extra tests beyond a task's words (4.4, 4.8, 5.1).
6. **References.** Tasks 2.13 and 2.21 give `data-refs` because their words are verbatim headings. Tasks 2.10, 2.12, 2.15, 2.17, 2.20 and the "one obligation renamed…" phrase in 4.8 are close to a heading but are not the heading, so they give none.

# 2026-09-15-catch-a-templates-placeholder-prose
**Read first: the archived instance and its repair.** Two packet questions are open (`the-gap-measured` and slice):
- `0ce5ad5` replaced the masthead `<p>` of `2026-09-11-migrate-an-openspec-project/evidence/the-plan-for-this-repository-2026-09-09.html`, so that slice carries the repaired bytes.
- My notes on tasks 1.2 and 6.5 state this as a fact about the copy, confirmed by `git log` (the last change to that file is `0ce5ad5`). Tasks 1.2 and 6.5 also link that copy.
- Whether the plan or the 2026-09-11 slice should say so is not mine to decide.

Other items:
- **Open question, carried as written**: "`artifact-kinds` reserves it for an agent" appears in the proposal's "Not in scope" and the design's Non-Goals. No requirement says this. No link, no note.
- **Packet note vs brief**: the mapped question on "`document-template`'s requirement that creation produce a valid document" suggests a link to `req-a-project-can-be-initialized-with-a-conforming-document` under convention 3. I added none. The words paraphrase rather than give the heading, and the brief says "Never link a requirement you inferred". Convention 3 covers source paths. The section is prose, so no `data-refs` was possible in any case. The brief wins.
- **Contract question**: *Generated example content…* says an absent summary "SHALL be reported by whatever rule governs an absent summary", and no rule does. The only `summary.is_empty()` check is this rule's own redundant guard. Carried as stated, per the plan's "Found while reading".
- **Judgment calls a reviewer may weigh**: the task deviations on 3.6 and 7.1, which the first attempt's table does not cover, and the `partial` verification of the unchanged *A newly created document*.

# 2026-09-18-establish-a-runbook
- **Tension inside the contract (carried as stated).** runbook-authority says "A runbook SHALL be a directory holding a `specification` document and a `tasks` document". Its own scenario "A runbook whose procedure is absent" treats a directory with no tasks document as a runbook, and the build lists such directories. The packet marks this as found while reading, not to be fixed here.
- **Data-refs beyond the packet's examples.** I gave task 4.2 `data-refs` for `req-the-canonical-procedure-carries-no-completion-state`. Its words are plainly that heading, and the packet names only 4.3 and 4.5 as examples. I gave task 4.4 `data-refs` for `scenario-an-enumeration-nobody-maintains`, because its note names the scenario by its exact heading. The packet says 4.4 "carries none", but that was about its words naming a requirement. The brief's rule for a named scenario decides it.
- **Reviewer, please look first at the two bootstrap verifications.** Their check runs `ProjectRoot::establish`, which `bootstrap` calls to create the directories, rather than the full `init` command. I read that as the same trigger. A stricter reading would drop both.
- **Carried as written, with no note.** The proposal's "10 external links" against five everywhere else: no record says which is a correction. The leftover "Two" before "One thing falls out at no cost".
- **A link on the misnamed requirement.** I linked the proposal's "A project declares its name, version, and expected toolchain" to its accepted counterpart, and the migration note beside it explains the misnaming.

# 2026-09-26-carry-a-runbook-to-another-machine
- **Read first: deviation f, `archive --check` recovery, which I classed E.**
  - A reviewer may judge it a contract question instead. "Readiness-only archival SHALL … write nothing" has no exception, and the build writes when a journal exists.
  - It bears equally on toolchain-packaging's *Archival supports a mode that checks without performing*, which belongs to another change. I wrote nothing there.
- **Deviation a turns on a reading.** I read an archive whose `VERSION` will not parse as a case of "a malformed existing snapshot". Read otherwise, it is class C and the note should go.
- At `5ea25e7`, the doc comment on `runbook_release::history` says "Fails closed. An archive whose version cannot be read … [is a] refusal rather than [an] omission". The loop beneath it skips. This contradiction is in the code only, and is not written into any document.
- No packet question is `open`, and no packet fact looked wrong.

# 2026-09-26-log-a-runbook-run
- **Read first: the five verification units.** They rest on the mutation record plus tests read at `16df91e`. That is the squash commit: the record was written on 2026-09-18, before the review round edited the tests, and the branch commits are not in history.
  - In `verify-a-run-under-way`, the scenario's "under the strictest setting available" is met by `run` having no strictness option at all.
  - `verify-a-record-offered-where-a-change-is-expected` is partial.
- **The brief over a packet note.** The packet's design note asks the specification to record "the whole procedure copied" as a departure, with a note. That build matches the delta's words, so the brief's class D wins and nothing is written. Only the other two departures carry notes.
- **Task 4.1** ("The fifth — that a record is history rather than contract") is not linked to *A run's record is history rather than contract*. I read it as a paraphrase, not the heading.

# 2026-09-26-record-a-renamed-requirement
1. **Read first: the five verifications built on the CLI test.** I relied on the 2026-10-13 rule for a test given by name and passing. The passing comes from the-runbook-ran's `scripts/check` run, a release record I gave no evidence unit. Please confirm that this combination supports `passed`, and confirm the one `partial`.
2. **Task 8.1's deviation** is my own finding. The `deviation_table` does not list it.
3. **The third-risk aside**, against the packet's "crosses as written". The words are unchanged, and the aside adds the disagreement.
4. **Packet note for 9.2.** "Nothing records the workspace test run" is not quite true: `scripts/check`'s `test` stage is that command. The note says so.
5. **Reader counts.** "26 tasks" is 27 in the source. "Seven fixes within the design" is eight bullets in what-the-implementation-revealed.
6. **Contract wording.** "The command surface reports…" still says "three sets" before it introduces a fourth. Its unchanged scenario "The comparison is of identifiers" says nothing of headings. I carried both as stated; the reader judged them awkward, not contradictory.
7. No packet fact looked wrong.

# 2026-09-28-confine-the-asset-check-to-the-project
1. **Task 2.1's references to all four scenarios.**
   - Only "two stores inside the project" spells its heading.
   - The other three are short forms, but the author introduces them as "the four scenarios in `specs/document-validation/spec.md`", in the spec's order, so I read each as plainly naming its scenario.
   - A stricter reading would keep only `scenario-two-stores-inside-the-project`.
2. **Sub-checks inside done tasks that nothing records:** 2.1's "first two fail on HEAD", 3.3's `--test promote`, and 3.4's "existing payload keys keep their values". I gave them no notes. The `deviation_table` lists only 5.2.
3. **Packet facts:** none looked wrong. The packet's 5.2 note says "nothing records the cargo test half". My note is more exact: `scripts/check`'s `test` stage runs it, so only a standalone run is unrecorded.

# 2026-09-29-keep-evidence-out-of-document-discovery
- **Verification judgment calls:**
  - "Carried evidence … does not gate promotion" and "An archived change keeps its evidence" rest on a staged tree that held no other document, so "whose documents all conform" holds only vacuously. The second also rests on promotion's checks rather than a bare `validate`.
  - "Formatting leaves evidence untouched": the record lists `--check` before `format`. My note argues the order makes no difference and cites the test that runs them in the scenario's order.
  - "Formatting a named record is refused": the test asserts that `format --check` exits with a document violation, not that its report names the file.
- **Attribution in the proposal.** The bullet "Promotion's 'Checks gate promotion' scenario then refuses…" is credited to the 2026-09-09 record ("measured what happens next"), but that record shows no promotion run. This change's own task 1.2 record does, and it calls that run "the refusal the proposal predicts". I linked this change's record from the claim and added no note. A reviewer may want one.
- **Validator warnings.** `review/unverified-requirement` still fires for all three requirements although 11 of their 13 scenarios have verifications, because the validator wants a verification that names the requirement itself. Left as is.

# 2026-10-02-reject-change-markup-inside-a-unit
- **Read first:** the five test-based verifications.
  - They rest on the record's statement that both suites passed after section 5.2's last restore. That run was against the rule before the review's fixes (`8032003`, equal to `ab54590`). I read the tests at the archive commit and they are unchanged, but the record's explicit "passed" predates the review's code.
  - A reviewer may judge that this needs the post-review run, which the record implies ("all ten are still caught") without stating it. If so, drop them.
  - The same question decides "A change element that is itself a unit", which I left without a verification.
- **Packet:** the reader's mapped partial for "A document declaring an earlier contract version" (the 0.1.0 fixture) is superseded here by a passed verification from the unit test. That is a stronger reading of the same record, not a contradiction of the packet.
- **Design decision 3 aside:** this is my own finding of a build departure, from `what-the-implementation-revealed`'s "not the projection's normalized text in one case". It is not in the triage. Drop it if "normalized text" in the decision is read as whitespace normalization only.

# 2026-10-02-see-a-modification-the-merge-did-not-take
1. **Check first: 7 verifications rest on unit tests, not on a run of `sync`.** They call `merge_state`, the one comparison that `sync` (commands.rs line 2436 at `913a87d`) and archival's `findings_for` (line 3375) both report from. They do not run the command. Each method says so. If "the operation is run against that change" requires the command itself, these become none under "a check whose … trigger differs".
2. **The brief overrides two packet notes on the proposal.** The packet's two proposal questions are decided "with a note linking task 7.1" and "with the disagreement noted". The brief says "Where a proposal described a plan the build then departed from, write no note on the proposal", and the brief wins. So the proposal carries no note. The "three sets" disagreement is noted on the specification's requirement, as the spec-delta question decided. The unchanged-requirement exception added in review needs no note, because the specification already states it as built.
3. **Decision 6's note links task 2.1, not task 7.1** (section 2).

# 2026-10-02-state-a-modified-requirement-whole
1. **Read first: the class C note on "Every synthesis and every contract update is agent-driven"** and decision 6's aside. I read the hold as beyond the delta rather than against it. A reviewer could instead read holding a mismatched `unchanged` scenario as contrary to "The merge SHALL apply it as stated", which would make it class A.
2. **A possible partial verification.** `the-rule-measured-2026-10-02.md` shows the served `sync` and `archive` bodies naming the requirement and no longer saying "partial update". That might be a partial check of "The project SHALL supply the rules and guides that direct these merges, as instructions rather than as code". I was unsure, so I wrote none, as the brief directs. The reader left it to the coordinator.
3. **Where the brief overrode the reader:**
   - The reader's deviations on 2.1, 2.2, 4.2, and 4.3 were declined under the 2026-10-13 threshold. 4.2's was moved to 6.1 by the packet's decided question.
   - The reader's link from "design decision 4" was declined under "Add no link the source did not write".
4. **Packet facts:** none looked wrong.
5. **The proposal's "Why now"** records that the first attempt staged `separate-artifact-kinds`' two kept scenarios only as a note. That is a fact about that time. I added no note, and the packet for that change states them `unchanged`.

# 2026-10-04-resolve-a-reference-across-its-change
- **Read first: the 10 verifications resting on tests read at `e61e3b1`.** They follow the second review's rule for name-only records, and this slice adds more of them than most. Three may need a reviewer's judgment:
  - "A reachable document that cannot be read" combines a CLI test and a conformance test to reach `passed`.
  - "A reference resolved by the change's specification" treats "reports nothing about the reference" as no `refs/*` diagnostic plus success.
  - "A reference to another change" takes its passing from mutation-testing's clean run, while the name comes from `the-rule-measured`.
  
  If any of these should be `partial` or dropped, it is a one-unit edit.
- **Packet facts:** none looked wrong.

# 2026-10-07-mark-the-words-a-modification-changed
1. **Read first: the class A sentence** in "A deleted word is recorded apart from what a unit states". It puts words the build implies into an `added` requirement, so this archived statement now differs from the accepted one. Please confirm A rather than D.
2. **Verification (a)**, "A word and a paragraph deleted…": I wrote `partial`. The packet leaves this open: is a check that has one of the scenario's two marks a partial check, or a check of a different condition?
3. **Verification (b)**, "Markup left in the accepted contract": I wrote none, because the trigger differs. The reader had mapped it as passed. The packet leaves this open.
4. **Test names as references.** The packet lists this as open. The brief's review of 2026-10-13 settles it, so task 2.4 carries no `data-refs` for the test names it mentions.
5. **The `runbooks.json` process gap.** It stays in the carried evidence, and nothing in the documents mentions it.
6. **Packet facts:** none looked wrong. The reader's counts ("nine bullets", "seventeen scenarios") are off; the source has ten and sixteen.

# 2026-10-08-migrate-into-the-extended-vocabulary
- **Read first:** the 15 verification units. Many rest on the first attempt's working-folder tools
  (`check_draft.py` and its self-test, `stage_carry.py`) run on scratch re-drafts, not on the
  product. I judged those to be records of checks of migration behaviour. A reviewer may judge some
  to be checks of something nearby.
- **A packet fact that looks inconsistent:** `scenario-a-mitigated-risk`,
  `scenario-an-accepted-risk-that-names-a-partial-measure`, and
  `scenario-a-check-that-reached-part-of-its-scenario` are unqualified. The same unqualified
  identifiers also head scenarios in the accepted `document-schema`, even though decision 2
  qualifies a scenario whose heading is shared with another in any document. I followed the packet.
  The staged accepted specifications do the same.
- **The packet's two `open` slice questions** concern other slices' findings: the 6.5 worklist, and
  item (d) of the proof's findings. They reach my documents only as figures that the evidence
  summaries and task records carry as written.

# 2026-10-08-record-a-partial-verification
1. **Read first: the verifications not written for document-validation's "A requirement whose check failed", "A requirement verified only in part", and "A fully verified specification".** These are the packet's open caveat 1.
   - The reader's records for them are scratch variants that draw `review/stale-verification`, because the requirement is `added`.
   - The mutation record has a cleaner check: its row 7, `a_verification_of_any_outcome_clears_the_unverified_warning`. I read it at `c3a9716`. Its doc comment names the first two scenarios. It uses an `unchanged` requirement, so no staleness warning fires. It asserts only that `review/unverified-requirement` is not reported, for every enumerated outcome, `failed` and `partial` included.
   - If "nothing is reported about that requirement" means that report, this test supports `passed` for both scenarios, and plausibly for "A fully verified specification" through the `passed` iteration.
   - If it means nothing at all, the test supports `partial`, with "nothing else is reported about that requirement" unchecked.
   - I wrote none of the three, and leave the choice to the review.
2. **"A check that reached part of its scenario"** (open caveat 2, and the open question on the presentation record). Variant c passed whole except for its Given: the verification names a requirement, not a scenario.
3. **No FOLLOWS.** `record-a-task-done-differently` is named as the change below this one in the stack only in prose (design, tasks 1.0, 1.3, 3.3, 9.1, and the evidence). Per the triage, the configuration names none, so the order is kept in prose only.
4. No packet fact looked wrong.

# 2026-10-08-record-a-task-done-differently
- **The packet's open question, the verification of "The enumeration and the completion report agree".** It is stated of "any active change". The record shows agreement for one change: for the measured document, `list --json` and `status --json` both report `doneDifferently` 1. For the note and dropped variants it shows only `status --json` at 0. I wrote no verification, because the triage leaves passed-or-partial open. If review settles it as `passed`, or as `partial` with "any active change" left unchecked, the unit is one paragraph from the same record.
- **Task deviation threshold.** I declined the reader's mappings for 2.1, 2.3, and 2.5, and did not map 1.3 or 7.1 (section 2). This change has no `deviation_table`, so the checker could not compare them. A reviewer who reads checks run against a stub as a difference in result would add deviations to 2.1 and 2.5.
- **Spaced literals in inline `<code>`** render with their spaces collapsed, though the bytes and the model keep them (section 4).

# 2026-10-08-record-how-a-risk-was-settled
- **Two verifications I did not write. Both questions are `open` in the packet, and the brief says write none when unsure.**
  - Variant E for "A response on another unit". The decision also carries `data-status`, `data-state`, and `data-outcome`, and it is compared with HEAD's model rather than with the same decision without the attribute.
  - The template-risk sentence of section 2 for "A mitigated risk". That record never validates the template.

  If review accepts either, it is one `verification` unit: E as passed, the template risk as partial, with "the document is valid" unchecked.
- **The design-decision notes on 5 and 8** rest on the record's own heading "Built beside the design". Decision 8's words ("rewrites the base's risk back to its committed form") are still true of what was built, so a reviewer may judge that note unneeded.
- **The 83 notes** (proposal and decision 1): the figure was true when written and grew because the repository changed, not because it was wrong. I applied convention 2 because task 1.2 calls the design's 83 stale and corrects it.

# 2026-10-08-show-what-a-modification-changed
1. **Read first: the two `partial` and one `passed` verifications the packet leaves open.**
   - "The author's marks account for the difference" is `partial`. The check had an `ins` and no `del`. I chose `partial` to match the mark-the-words pilot's identical case.
   - "The view writes nothing" is `partial`.
   - "Showing a change without a view" is `passed`. "Shows nothing" is read as "prints no view", so the stream the human-form message went to does not matter.
   - "A paragraph rewritten throughout" has none (§5).
2. **The class A note under "Scenarios SHALL be shown scenario by scenario".** Please confirm this is A with the words kept, not D. It depends on whether "still holds it" means by identifier, which is how decision 6's table reads it, or by content, which is how `sync` and the build read it.
3. **Four task deviations beyond the reader's list:** 3.1, 5.3, 5.4, and 6.1. Each is a judgment under the review's rule. The line I drew is in §2.
4. **Not annotated:** the design's first Risk says task 1.1 "corrects decisions 3, 5, and 6". The record shows 3, 5, and 7. This is a plan that went otherwise, not a figure, so no convention asks for a note. It is carried as written.
5. **Packet facts:** none looked wrong.
   - The plan's text for toolchain-packaging (line 595) still gives the unqualified `scenario-an-invalid-specification` for this change. The packet's qualified identifier follows the brief's decision 2 as the second review settled it, and I used the packet's.

# 2026-10-09-fill-only-the-references-a-document-loads
- **The brief overrode a packet note.** The proposal question's note says "Why now" "links
  the-change-measured section 2, with no other annotation". Convention 2 says that a claim later
  corrected gets a migration note and that "A bare link does not tell a reader it points at a
  correction". I followed the brief and wrote an aside with the link. The claim "is still in two staged
  documents" was true. What the evidence corrected is that only one of the two needed the workaround,
  so a reviewer may judge whether this counts as a correction at all.
- **A contract question, for a change of its own.** "Formatting preserves meaning" states "SHALL NOT
  disagree" with no bound, while the change accepted disagreement on documents parsed with errors and
  on character references. The requirement could state that bound. I recorded it as a note only.
- **2.3's deviation is in my words** while the author's two bullets hold the account. The brief's
  one-paragraph rule assumes one bullet states the difference. Here two do.
- The proposal links "prose quoting an asset reference" to the scenario headed "Prose that quotes an
  asset reference". I read it as plainly that heading, as the reader did. It carries no `data-refs`, so
  nothing in the model depends on it.
- No packet fact looked wrong.

**Read first:** the `Migration note:` on "Formatting preserves meaning" in `specification.html`. It is
the one place where the draft says the build and the stated contract differ.

# 2026-10-12-name-each-command-by-its-request-verb
- **Read first:** the task-deviation calls. Seven tasks are involved: the reader mapped deviations
  on 2.2, 2.3, 5.2 and 5.3, and I declined them; I kept 2.5, 3.2 and 3.3. This change has no
  `deviation_table`, so only review checks these calls against the new threshold.
- Packet `open` items:
  - Decisions 4 and 5 built otherwise, and decision status: handled as the brief's section on a
    decision's status says. The destination still has no "built otherwise" field for a decision.
  - Evidence section 1's unsettled choice to hide the verbs: carried as written. The decision 5
    aside states the cost and does not say it was settled.
- Contract question (decided by triage, flagged for the reviewer): the literal "SHALL produce the
  same output" against the usage line.
- No packet fact looked wrong.

# spec-artifact-kinds
- **Read first: the `<p>` in the author's table's cells.** The reader advised bare text, so the
  model would hold "The canonical mapping SHALL be:" with nothing after it. With bare cells,
  `check_draft.py` reported six "statement sentence MISSING" findings. Convention 5 as amended
  forbids a `<p>` only in a table the migration *adds*, and its principle is that a normative list
  keeps projecting. So each of the 12 body cells holds a `<p>`, and the requirement now projects 16
  statements, among them bare "proposal" and "None". The header cells hold none: the brief treats a
  header that names its column as structure, and the first paragraph already defines both terms.
  **The 2026-08-14-separate-artifact-kinds drafter must make the same choice for this
  requirement**, body cells only. Otherwise its statements differ from the accepted ones as an
  ordered list of paragraphs, which is how toolchain-packaging's "The command surface reports
  whether a change reached the accepted contract" compares them. The brief does not address an
  author's table inside a requirement.
- **A contract question, not resolved here.** Every source paragraph is one unmarked `<p>`, so reason
  paragraphs, and now the table cells, project as statements that carry no SHALL or MUST.
  document-schema's "A requirement contains a title and normative statement" says the normative
  statement SHALL use SHALL or MUST. The validator raised nothing. The contract does not say
  whether a reason paragraph or a table cell is "the normative statement".

# spec-contract-authority
- **Packet questions, followed as settled.** "'this project has already measured that divergence
  in its report guidance' rests on no evidence file" (`decided`, review question 6): carried as
  written, no link. "The Purpose … covers the first three requirements" (`decided`, decision 5):
  carried verbatim, although it does not mention requirements 4 and 5 and calls the repair "a
  stated obligation rather than a stricter check" while requirement 4 is a stricter check. The
  third entry (`note`) binds the plan, not this document.
- **A contract tension, not resolved here** (as the document-schema drafter also found). Every
  source paragraph is one unmarked `<p>`, so reason paragraphs project as statements, and several
  state no SHALL or MUST (for example "Stated as a property of the operation rather than as a list
  of commands…" and "Stated without a count, deliberately…"). document-schema's 'A requirement
  contains a title and normative statement' says the normative statement SHALL use `SHALL` or
  `MUST`. The validator raised nothing. Whether a reason paragraph is a statement, the contract
  does not say.
- Read first: the declined comparison table (section 4). The plan carries it as a reader format
  note, so a reviewer may expect it.

# spec-deterministic-formatting
- Nothing from the packet is open. Every question is `decided`, `fixed`, or `mapped`. The hand
  conversion at `2026-08-10-html-spec/documents/deterministic-formatting.html` (now under that
  change's `evidence/documents/`) uses different identifiers for the same seven obligations
  (`req-idempotent`, `req-canonical`, and so on). As decided, I drew nothing from it, and the two
  identifier sets stay unreconciled.
- Read first: the scenario "Prose that quotes an asset reference". Its `href="assets/htmlspec.css"`
  is written as plain `<code>` text with no workaround, and `format` and `format --check` left it as
  written. That is the 0.28.1 fix, and the requirement it states, holding on this very document.
- A procedural slip, now corrected: my one scratch copy (a byte-comparison of the file before
  `format`) went straight into `/private/tmp/` rather than a directory named for this assignment. I
  deleted it. Nothing else was created outside the destination and this report.

# spec-document-schema
- **The packet and the brief disagree on one question.** The packet lists "citations by heading
  across accepted specifications" as `open`. The brief's "An accepted specification" section and its
  Links section (review 2026-10-13) settle it: a quoted heading stays text. I followed the brief. So
  four quoted headings carry no link and no `data-refs`:
  - "Every semantic unit has a stable unique identifier" and the scenario "Reference to a removed
    requirement", both inside this document;
  - "The element contract is published as a machine-readable schema" and "A task declares
    completion state", also inside this document;
  - document-validation's "Change markup in or around a semantic unit is rejected", in another
    accepted specification.

  The packet's triage may want updating.
- **Evidence links from accepted specifications** (`open` in the packet). This touches 'A deleted
  word is recorded apart from what a unit states', paragraph 3. It is carried as written, with no
  link.
- **A contract tension, not resolved here.** Every source paragraph is one unmarked `<p>`, so the
  reason paragraphs project as statements. Several state no SHALL or MUST: for example "The reason
  is that…" in the rename, task, deleted-word, partial-verification, and risk requirements. 'A
  requirement contains a title and normative statement' says "The normative statement SHALL use
  `SHALL` or `MUST`". The validator raised nothing, so this is not a check failure. Is a reason
  paragraph a statement? The contract does not say.
- Read first: the two reader tables I declined (section 4). The plan carries both as format notes,
  so a reviewer may expect them.

# spec-document-template
- **The accepted text states a behaviour the product does not have.** The scenario 'A document
  declaring a lifecycle status' says validation reports a declared lifecycle status. Triage
  (`triage-a.json`, and the 2026-10-13 review's note in `zz-review-2026-10-13.json`) records that
  htmlspec 0.29.0 reports nothing for `htmlspec-status`. I did not re-run that check. Under decision
  5, the scenario is carried as written with no note. The class B note belongs to
  2026-08-14-separate-artifact-kinds's specification, and the fix needs a change of its own.
- **Both of the packet's `decided` questions were followed.** The history paragraph in 'A document
  declares its identity and provenance' ("The declared kind replaces … nothing in validation ever
  read it") is carried verbatim as an unmarked `<p>`. So is the Purpose. That history paragraph has
  no SHALL, yet it projects as a statement. The document-schema drafter raised the same tension
  against 'A requirement contains a title and normative statement'. The validator reported nothing.
- Read first: the lifecycle-status item above. It is the one place where this document's words
  and the built program disagree.

# spec-document-validation
- **Two product questions, carried as stated** (packet: `decided`). The scenario 'Warnings can be
  made fatal' says strict validation treats warnings as errors, with no exception. The code never
  promotes `review/*` (`Namespace::is_promotable`, `crates/htmlspec-validate/src/diagnostic.rs`).
  'Diagnostics are actionable and machine-readable' and the scenario 'Improperly nested elements'
  require a location, but `Location` holds no line or offset. Both texts are carried verbatim, with
  no note. A later change to document-validation would settle them.
- **Purpose scope** (packet: `decided`, carried verbatim). The Purpose says "a specification
  document". It predates the four kinds and reads narrower than the requirements, which reach every
  kind and, for the asset-duplication rules, a project.
- **A contract tension, the same one the document-schema drafter raised.** Every source paragraph is
  one unmarked `<p>`, so reason paragraphs project as statements. Several state no SHALL or MUST,
  for example "The reason for the rule is that change markup states a history…" and "The rule is an
  error because…". document-schema's 'A requirement contains a title and normative statement' says
  "The normative statement SHALL use `SHALL` or `MUST`". The validator raised nothing. The contract
  does not say whether a reason paragraph is a statement.
- Read first: the two reader tables I declined (section 4). The plan carries both as format notes,
  so a reviewer may expect them.

# spec-project-bootstrap
- **Read first: the summary.** Decision 3 asks for a summary that says nothing the requirements do
  not. I chose to name every requirement by its heading so that each claim is traceable. That makes
  the summary long (three paragraphs). A reviewer should check it clause by clause against the
  requirements, and decide whether the parenthetical headings are the form wanted.
- **Built behaviour against the accepted text** (packet question 2, decided under decision 5).
  Bootstrap writes `htmlspec = null;` (`render_declaration`, `crates/htmlspec-cli/src/commands.rs`).
  The scenarios 'Bootstrapping writes the declaration' and 'The pinned program is a build input'
  still state a pinned program. They are carried as written, with no note.
- **Reason paragraphs project as statements.** This is the same tension the document-schema drafter
  reported. For example, "That division is the one archival already uses: …" and "The runbooks are a
  fourth place …" hold no SHALL or MUST, yet each one is an unmarked `<p>` inside a requirement. The
  validator raised nothing.
- Carried exactly as written: "A deleted runbook SHALL not cause deletion of its snapshots" (with a
  lowercase "not").

# spec-project-migration
- **Read first: the figure.** It is the only content in the document the migration wrote. Section 4
  explains why I dropped the reader's two gate nodes under convention 12. A reviewer who reads that
  convention as allowing a stage flow to show other requirements' gates may want them back. The
  plan's format note still describes them.
- **The plan's format notes ask for links that this draft does not have.** The reader asked for
  same-document and cross-capability links around every quoted heading, and for `<q>`. The brief
  overrides the links (review 2026-10-13), so a reviewer comparing the draft with the plan will see
  the difference. The plan's text could be updated.
- **Carried as stated, as the packet decided:**
  - The tension between 'The stages that capture the work…' (stages 1 to 4 produce no
    destination-format document) and 'The plan accounts for every surveyed entity' (the plan is a
    tasks document).
  - The Purpose, which describes only the 2026-09-11 capability and none of the six mapping
    requirements added since.

  Both are candidates for a change after promotion.
- **A contract question, as the document-schema draft also found.** Several statement paragraphs
  give reasons and contain no SHALL or MUST: "The reason is that…" in the conformance, conversion,
  layout, task, reference and risk requirements, and similar paragraphs elsewhere. Each source
  paragraph is one unmarked `<p>`, so each of these projects as a statement. document-schema's "A
  requirement contains a title and normative statement" says the normative statement SHALL use
  SHALL or MUST. The validator raised nothing, and the contract does not say whether a reason
  paragraph counts as a statement.

# spec-runbook-authority
- **The packet's decided questions, and where each one touched the document.**
  - The merge-misplaced paragraph in 'The canonical procedure carries no completion state' is
    carried as merged. 'A unit in the record of a run SHALL declare that it is unchanged…' (from
    `2026-09-26-log-a-runbook-run`) still sits between the canonical-documents paragraph and 'The
    consequence is what makes it worth stating.' So 'The consequence' still reads as following from
    the run-record paragraph. That is left for a later change.
  - The Purpose's 'and what a runbook deliberately does not carry' is carried verbatim, although
    'A runbook carries an independently checked release version' now reverses the no-version rule
    the phrase first referred to.
  - The two 2026-09-26 requirements have no evidence (their change has no `evidence/`), and here
    they draw `review/unverified-requirement` like the rest.
  - The removed requirement 'A runbook carries no version and no changelog of its own' is
    correctly absent here.
- **Citations by capability name stay as text.** `project-bootstrap` (in 'A runbook carries an
  independently checked release version') and `runbook-portability` (in 'A machine requirement is
  still an obligation of the run') are `<code>` text, with no link and no `data-refs`. The same goes
  for the paraphrased citations ("the prohibition on deriving a version from a directory name",
  "the only placement rule in the artifact-kind vocabulary"). The brief forbids a link from a
  capability's name and a link the source did not write. The reader names the requirements each one
  resolves to, if a later change wants to make them explicit.
- **The same contract tension the document-schema drafter reported.** Every source paragraph is an
  unmarked `<p>`, so each one projects as a statement. 'The honest cost of that…' in 'A runbook
  records what its history taught' states no SHALL or MUST. document-schema's 'A requirement contains a title and normative statement' says the normative
  statement SHALL use `SHALL` or `MUST`. The validator raised nothing. Whether a reason paragraph is
  a statement, the contract does not say.
- Read first: the declined table (section 4). The reader's slice notes call it "One table to add",
  so a reviewer may expect it.

# spec-runbook-execution
- **Settled reader questions, carried as stated.** The packet marks three `decided` ("Found while
  reading, carried as stated"). Each is carried verbatim, with no note:
  - In 'A run is one record from start to finish', paragraph 4 says the reason is "the same
    objection that keeps a runbook from declaring a version of its own". That reason is stale:
    runbook-authority's 'A runbook carries an independently checked release version' now requires a
    runbook to declare a version.
  - In 'A run leaves a record where the runbook keeps them', paragraph 3 has the clause "the
    project's places are the four a contributor looks for and one of them cannot exist before its
    runbook does".
  - In 'A run's conformance to its runbook's contract is reported and never gated', the third
    non-report case has no scenario. I added none.

  The fourth question, on runbook-portability's preflight, is `withdrawn`. It touches nothing here.
- **A quoted heading stays text.** 'A record carries…', paragraph 2, quotes document-schema's
  "Cross-references address units by identifier". It has no link and no `data-refs`, as the brief's
  "An accepted specification" section and Links section (review 2026-10-13) require.
- **Modal vocabulary of the list items.** Convention 5 makes each of the three non-report cases a
  `<p>`, so each projects as a statement. None of the three contains SHALL or MUST: the modal is in
  the lead-in, "The report SHALL NOT be produced for:". The validator raised nothing. The
  document-schema drafter reported the same tension for reason paragraphs.
- No packet fact looked wrong. Read first: nothing in this document is contested. If a reviewer
  reads one thing, it should be section 4's declined diagram, because the plan once proposed it.

# spec-runbook-portability
- **The `log-a-runbook-run` link.** The packet's note says the citation "may also link to the
  migrated archive of that change". The brief's Links section (review 2026-10-13) allows "no link
  from a capability's name or a change's name in prose", and convention 3 covers cited source paths,
  which this is not. I followed the brief and added no link. "May" leaves that choice open, but the
  packet note could be aligned with the brief.
- **Read first: the declined supported-needs table (section 4).** The plan carries it as a format
  note, so a reviewer may expect it.

# spec-toolchain-packaging
Read first: **the one added table, and the six reader tables I declined** (section 4). The plan
carries all seven as format notes, so a reviewer may expect them. One risk comes with the added
table. Neither `sync` nor `show --diff` compares it ('The command surface reports whether a change
reached the accepted contract' compares statements only). So a later change that adds a spelling
to paragraph 5 could leave the table stale, and no check would notice. Drop it if that risk
outweighs the scan.

Where the packet's settled questions touched the document, I carried the source as written:

- **Rename** (`mapped`): the naming requirement keeps `req-a-single-command-exposes-validate-format-and-init`
  under its current heading, with no `renamed-from` paragraph (artifact-kinds' "A rename reaches the
  accepted contract as a new heading on the same identifier").
- **The tension between 'Archival produces the specification and updates the accepted contract'
  and 'Archival assesses the merge and never performs it'** (`decided`): both are carried verbatim.
  The naming requirement's "this program's `sync` and `archive` never do" sharpens it.
- **"The obligations on the *program* live in the requirement above"** (`decided`): carried
  verbatim, with no link. The requirement directly above is now 'The workflow instructions are
  available from the command surface', not the reporting operation it meant.
- **Quoted headings** (`decided`, review question 6): all of them stay text, with no link and no
  `data-refs`. That covers four in-document headings, cited six times, and the cross-capability
  citations of artifact-kinds, document-schema, and document-validation.
- **The incomplete enumeration of spellings** in paragraph 3 (`decided`): carried, not filled in.
- **The Purpose** (`decided`): verbatim, although it is stale.

Minor points:

- **A contract question, the same one spec-document-schema raised.** Nine statement paragraphs
  carry no SHALL or MUST. They are reason or context paragraphs, such as "The asymmetry is the
  requirement.", "The reason is the reviewer.", and "The enumeration of accepted specifications is
  unaffected by the above, which governs changes only." Each is one unmarked `<p>`, as convention 5
  requires, so each is a statement. 'A requirement contains a title and normative statement' asks
  for SHALL or MUST, and the validator raised nothing. The contract does not say whether a reason
  paragraph is a statement.
- **A reader figure.** The reader gives "126 statements (124 paragraphs plus the three list
  items)", but the total is 123 paragraphs plus 3 items. The total, 126, is right.
- **The packet itself**: no fact looked wrong.

# spec-workflow-reporting
- **Read this first: the declined table (section 4).** The plan carries it as a format note, so a
  reviewer may expect to see it. I declined it under the brief's rule on tables, because the brief
  wins over a packet or plan note.
- The triage records a silence: the-archive-report says "No obligation in workflow-reporting
  addresses arithmetic, and none should". That belongs to the change's evidence, and nothing was
  written into this specification for it.
