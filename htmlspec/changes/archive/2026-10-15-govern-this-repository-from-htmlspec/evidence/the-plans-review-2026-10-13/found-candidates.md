# Candidates for "Found while reading" and "What the destination cannot express"

These come from the Open sections in `review/open-sections.md`, checked against the `found` and `cannot` lists in `make_decisions.py`. Each one is a place where the product disagrees with an accepted or archived requirement, where the contract disagrees with itself or is silent on something it presupposes, or a distinction the destination cannot express. None is in either list.

"Confirmed at 139b206" gives what the drafter says. Where a packet or a triage note confirms a finding the drafter did not re-check, the entry says so. "Text" means the finding is about wording, so no code check or run applies.

31 candidates: 11 Product, 20 Contract, 0 Cannot express.

## Product (11)

**1. Product.** The shared-file region written into `AGENTS.md` has no frontmatter, so it declares no name, but "Every generated name is loadable" says every file the operation writes declares one.
- Where: `2026-08-18-conform-generated-frontmatter-names`. The same scenario is in accepted project-bootstrap ("Bootstrapping installs agent instructions for the tools a project uses").
- Raised by: 2026-08-18-conform-generated-frontmatter-names.
- Confirmed at 139b206: no. The drafter read code at `4972234`, where the test walks skills and commands but never regions. The drafter classed it A because the record treats the words as met.

**2. Product.** The absent-command check resolves only the subcommand path, so a withdrawn spelling such as `htmlspec init doc.html` is not reported. The scenario "Instructions naming an absent command" says it is reported. The design's "treat the prefix as the invocation" and task 1.4 disagree about which behaviour was intended.
- Where: `2026-08-18-install-the-full-workflow-set`, and accepted project-bootstrap.
- Raised by: 2026-08-18-install-the-full-workflow-set.
- Confirmed at 139b206: yes, according to the drafted note, which says it is still so there. The drafter read the code at `5f7e5cf`: a test and a doc comment in `invocations.rs`. Class E.

**3. Product.** Each workflow body written into the `AGENTS.md` region opens with `## What you were asked` and an unsubstituted `$ARGUMENTS`, seven placeholders in all. On one reading of "This obligation … SHALL NOT reach a kind that carries no invocation", that is a defect.
- Where: `2026-08-24-pass-the-description-through`, and accepted project-bootstrap.
- Raised by: 2026-08-24-pass-the-description-through.
- Confirmed at 139b206: yes. The drafter read `write_region` in the code at `4f2f3e7` and at `139b206`. They classed it C. A reviewer may judge it B.

**4. Product.** The release check fails when `Cargo.toml` is bumped before its changelog entry exists, which is a working-tree state. toolchain-packaging says "A rule whose violation is a legitimate state for work in progress SHALL therefore be stated as a property of a settled record".
- Where: `2026-08-21-check-release-hygiene`, and accepted toolchain-packaging ("Project checks are runnable as one command").
- Raised by: 2026-08-21-check-release-hygiene.
- Confirmed at 139b206: no. The drafter read `scripts/check-release` at `41310f8`. They classed it C. A reviewer may judge it E or B.

**5. Product.** The cycle walk covers active changes only. Two changes that each follow the other, one of them archived, are therefore not reported, though the contract says "Two or more changes that each follow the other SHALL be reported".
- Where: `2026-09-02-state-a-changes-place-in-a-sequence`, and accepted artifact-kinds ("A stated predecessor is resolved across the archive").
- Raised by: 2026-09-02-state-a-changes-place-in-a-sequence.
- Confirmed at 139b206: yes. The drafter read the code: the doc comment on `cycle_through` (`sequence.rs:200`) at `80558b5` is unchanged at `139b206`. Class E.

**6. Product.** `archive --check` first completes or rolls back an interrupted archival, so readiness-only archival can write. The requirement says "Readiness-only archival SHALL apply the same comparison and write nothing", and its scenario says it "leaves all files unchanged". This is a different finding from the listed "`archive --check` skips the collision test".
- Where: `2026-09-26-carry-a-runbook-to-another-machine`, and accepted project-bootstrap ("Archival checks the versions of runbooks it publishes"). It also bears on toolchain-packaging's "Archival supports a mode that checks without performing".
- Raised by: 2026-09-26-carry-a-runbook-to-another-machine.
- Confirmed at 139b206: yes, in code (`main.rs:445`, `commands.rs:1341`), as well as at `5ea25e7`. The drafter classed it E and notes it could be read as a contract question instead.

**7. Product.** Archival skips an archive whose recorded `VERSION` will not parse. On the drafter's reading, that breaks "A malformed existing snapshot SHALL be reported, not treated as an absent predecessor". The doc comment on `runbook_release::history` says the code "fails closed", but the loop beneath it skips.
- Where: `2026-09-26-carry-a-runbook-to-another-machine`, and accepted project-bootstrap ("Archival checks the versions of runbooks it publishes").
- Raised by: 2026-09-26-carry-a-runbook-to-another-machine.
- Confirmed at 139b206: no. The drafter read the code at `5ea25e7` only (`runbook_release.rs:1096`). They classed it A with the words kept. On another reading it is C.

**8. Product.** Bootstrap writes `htmlspec = null;`, but the scenarios "Bootstrapping writes the declaration" and "The pinned program is a build input" still describe a pinned program.
- Where: accepted project-bootstrap.
- Raised by: spec-project-bootstrap.
- Confirmed at 139b206: the drafter cites the code (`render_declaration`, `crates/htmlspec-cli/src/commands.rs`) without naming a commit. The packet records that the triage confirmed it live at `139b206`.

**9. Product.** The formatter and the validator are not held to agreement in two cases: a document whose parse recovered from errors, and character references (the tree decodes `&amp;` and the scanner does not). "Formatting preserves meaning" says they "SHALL NOT disagree", with no limit.
- Where: `2026-10-09-fill-only-the-references-a-document-loads`, and accepted deterministic-formatting.
- Raised by: 2026-10-09-fill-only-the-references-a-document-loads.
- Confirmed at 139b206: no code check or run is stated. The finding comes from the change's own record: the design's Risks, and task 7.1's "Not taken". The drafter classed it A with the words kept.

**10. Product.** When clap refuses the arguments, the request-verb spelling prints a usage line that differs from the one for the operation's own name. For example, `get validity --bogus` prints a different line from `validate --bogus`. The contract says the spelling "SHALL produce the same output and exit status". The record's contributor read those words as covering only invocations that reach an operation.
- Where: `2026-10-12-name-each-command-by-its-request-verb`, and accepted toolchain-packaging ("The command surface names its operations as OpenSpec names them").
- Raised by: 2026-10-12-name-each-command-by-its-request-verb.
- Confirmed at 139b206: not by the drafter. The packet records it reproduced with 0.29.0.

**11. Product.** `init` writes the runtime into `assets/` beside each new document. So a document initialized in another directory of the project gets a second copy, unless `--assets` is given. A reviewer may judge this against "A project can be initialized with a conforming document".
- Where: `2026-08-13-content-address-runtime-assets`, and accepted document-template.
- Raised by: 2026-08-13-content-address-runtime-assets.
- Confirmed at 139b206: no, only at `86c33f4`. The drafter classed it D, because the delta defines no project root.

## Contract (20)

**12. Contract.** Under `--json`, no archival refusal emits a payload: the version gate, a missing or wrong-kind specification, a validation error, or a collision. The contract does not settle whether "Under structured output archival SHALL emit its reports and SHALL state whether the change was moved" covers a refusal. If it does, every structured refusal deviates.
- Where: `2026-08-30-finish-the-archival`, and accepted toolchain-packaging ("Archival is drivable without a person present").
- Raised by: 2026-08-30-finish-the-archival.
- Confirmed at 139b206: the drafter says the behaviour holds at `cf11132` and at `139b206`, without saying how they checked. They classed it D.

**13. Contract.** One requirement says two things that disagree for a change with no tasks document. Its scenario "The enumeration and the completion report agree" says the two report the same counts for any active change. Its statement says "A change whose tasks document is absent SHALL be enumerated with its completion absent", while the completion report gives 0 for such a change.
- Where: accepted toolchain-packaging ("The command surface reports what a project holds"), and `2026-09-02-carry-task-progress-into-the-listing`.
- Raised by: 2026-09-02-carry-task-progress-into-the-listing.
- Confirmed at 139b206: no. The finding comes from the change's record, which accepted the asymmetry.

**14. Contract.** The requirement says "The report SHALL distinguish three sets", then adds further sets: content not taken, removals not taken, and renames not taken.
- Where: accepted toolchain-packaging ("The command surface reports whether a change reached the accepted contract"), `2026-09-26-record-a-renamed-requirement`, and `2026-10-02-see-a-modification-the-merge-did-not-take`.
- Raised by: 2026-09-26-record-a-renamed-requirement and 2026-10-02-see-a-modification-the-merge-did-not-take.
- Confirmed at 139b206: text. I checked that `openspec/specs/toolchain-packaging/spec.md` at `139b206` still says "three sets".

**15. Contract.** The reach comparison says nothing about an accepted specification it cannot read. `merge_state` skipped such a specification and treated a failed discovery as holding nothing, so its requirements read as absent. The stated refusals cover only the change's own specification.
- Where: `2026-08-31-sync-a-change-into-the-accepted-contract`, and accepted toolchain-packaging, whose refusal paragraph is unchanged.
- Raised by: 2026-08-31-sync-a-change-into-the-accepted-contract.
- Confirmed at 139b206: no. The drafter read the code at `ee4301c` only.

**16. Contract.** artifact-kinds' "A change states the change it follows" names no file. It places the declaration only "with the change's whole-directory facts". project-migration has a migrated change state its predecessor in `FOLLOWS` "as artifact-kinds' 'A change states the change it follows' provides".
- Where: accepted artifact-kinds and project-migration, and `2026-09-02-state-a-changes-place-in-a-sequence`.
- Raised by: 2026-09-02-state-a-changes-place-in-a-sequence, as the packet's open question.
- Confirmed at 139b206: text. The packet confirms both texts at `139b206`.

**17. Contract.** No requirement says what the merge does with an `unchanged` scenario whose text differs from the accepted one. As built, `sync` holds the whole requirement unmerged and `archive` stops. That may conflict with "The merge SHALL apply it as stated".
- Where: `2026-10-02-state-a-modified-requirement-whole`, and accepted artifact-kinds ("Every synthesis and every contract update is agent-driven").
- Raised by: 2026-10-02-state-a-modified-requirement-whole.
- Confirmed at 139b206: the drafter checked the accepted specifications' text at `139b206`. None states a hold.

**18. Contract.** The phrase "saying whether the accepted requirement still holds it" can mean two things. Design decision 6's table reads it by identifier. `sync` and the view as built read it by content: they show "no longer held" when another stated scenario keeps the text.
- Where: `2026-10-08-show-what-a-modification-changed`, and accepted toolchain-packaging ("The command surface shows a modified requirement beside the accepted one").
- Raised by: 2026-10-08-show-what-a-modification-changed.
- Confirmed at 139b206: no. The finding comes from the record: task 9.1, where the PR #47 review was "Not taken".

**19. Contract.** toolchain-packaging says "This obligation is on this repository. It SHALL be distinguishable from any obligation the program places on a project it bootstraps". That may no longer hold, because the shipped archival gate reads the `VERSION` files this change treats as the repository's own convention.
- Where: accepted toolchain-packaging ("The project records what changed in each version it ships"), and `2026-08-21-check-release-hygiene`.
- Raised by: 2026-08-21-check-release-hygiene.
- Confirmed at 139b206: not by the drafter. The packet records the text, and `recorded_version` in the code, confirmed at `139b206`.

**20. Contract.** Task 2.5 asks for the boundary "an obligation on this repository must not be written into prose the product ships" to be stated at archival. No requirement states it.
- Where: `2026-08-21-check-release-hygiene`.
- Raised by: 2026-08-21-check-release-hygiene.
- Confirmed at 139b206: text.

**21. Contract.** Task 5.2 says a boundary "is worth stating at archival": a prose repair to authoring guidance cannot be verified end to end while the invocation withholds the contributor's input. The delta does not state it.
- Where: `2026-08-22-accept-a-described-change`.
- Raised by: 2026-08-22-accept-a-described-change.
- Confirmed at 139b206: text.

**22. Contract.** The evidence argues "No obligation in workflow-reporting addresses arithmetic, and none should". That is a boundary no requirement states. The record treats the silence as deliberate.
- Where: workflow-reporting, and `2026-08-24-report-in-the-contributors-terms`.
- Raised by: 2026-08-24-report-in-the-contributors-terms and spec-workflow-reporting.
- Confirmed at 139b206: text.

**23. Contract.** The proposal's "Not in scope" and the design's Non-Goals say that judging whether a summary is a good one is reading, which "`artifact-kinds` reserves … for an agent". No requirement says this.
- Where: `2026-09-15-catch-a-templates-placeholder-prose`.
- Raised by: 2026-09-15-catch-a-templates-placeholder-prose.
- Confirmed at 139b206: text.

**24. Contract.** In "The canonical procedure carries no completion state", a merge left the run-record paragraph between the canonical-documents paragraph and "The consequence is what makes it worth stating". So the consequence reads as following from the wrong paragraph.
- Where: accepted runbook-authority.
- Raised by: spec-runbook-authority.
- Confirmed at 139b206: text. The packet confirms it at `139b206`.

**25. Contract.** The clause "the project's places are the four a contributor looks for and one of them cannot exist before its runbook does" reads as one of the four places. It means a runbook's place for its records. The packet marked this "Found while reading, carried as stated", but the `found` list omits it. The `found` list's runbook-execution entry covers only the packet's other two items.
- Where: accepted runbook-execution ("A run leaves a record where the runbook keeps them").
- Raised by: spec-runbook-execution.
- Confirmed at 139b206: text. The triage confirmed it at source line 26.

**26. Contract.** contract-authority's Purpose covers only the first three requirements. It calls the repair "a stated obligation rather than a stricter check", while requirement 4 is a stricter check.
- Where: accepted contract-authority.
- Raised by: spec-contract-authority.
- Confirmed at 139b206: text.

**27. Contract.** document-validation's Purpose says "a specification document". That is narrower than its requirements, which reach every kind and, for the asset-duplication rules, a project.
- Where: accepted document-validation.
- Raised by: spec-document-validation.
- Confirmed at 139b206: text.

**28. Contract.** project-migration's Purpose describes only the capability as it was introduced on 2026-09-11. It covers none of the six mapping requirements added since.
- Where: accepted project-migration.
- Raised by: spec-project-migration.
- Confirmed at 139b206: text.

**29. Contract.** runbook-authority's Purpose says "and what a runbook deliberately does not carry". That phrase referred to the no-version rule, which "A runbook carries an independently checked release version" has since reversed.
- Where: accepted runbook-authority.
- Raised by: spec-runbook-authority.
- Confirmed at 139b206: text.

**30. Contract.** toolchain-packaging's Purpose is out of date. It understates what the capability now governs.
- Where: accepted toolchain-packaging.
- Raised by: spec-toolchain-packaging.
- Confirmed at 139b206: text.

**31. Contract.** "A source distinction the destination cannot express…" says migration SHALL NOT map a distinction onto the nearest available value. The same change's design, audit, and run carried renames as `modified` with prose.
- Where: `2026-09-11-migrate-an-openspec-project`. The requirement is also in accepted project-migration.
- Raised by: 2026-09-11-migrate-an-openspec-project.
- Confirmed at 139b206: text.

## Cannot express (0)

No Open section names a distinction the destination cannot express beyond the three in the `cannot` list. Three reports touch that list's first two rows:
- 2026-10-12-name-each-command-by-its-request-verb says a decision built otherwise has no field.
- 2026-08-14-separate-artifact-kinds raises a decision that a later change narrowed. That change calls it a narrowing rather than a reversal. The drafter raises it only as a question of whether to add a note.

## Listed already, with detail the list lacks

- **Reason paragraphs with no SHALL or MUST** (`found`, document-schema "and others"). This was raised by ten accepted-specification drafters. spec-artifact-kinds extends it to an author's table: twelve body cells each project a statement, among them bare "proposal" and "None". spec-runbook-execution extends it to list items whose modal sits in the lead-in ("The report SHALL NOT be produced for:").
- **Strict mode never promotes `review/*`** (`found`, document-validation). 2026-08-10-html-spec describes it as two requirements in tension: "Warnings can be made fatal" against "An upstream edit marks dependents for review rather than gating work". The build follows the second. The listed entry names only the product side.
- **An absent summary deferred to "whatever rule governs an absent summary"** (`found`, contract-authority). 2026-09-15-catch-a-templates-placeholder-prose adds that the only `summary.is_empty()` check is this rule's own redundant guard.

## Considered and left out

- **The short-doctype rule** (2026-08-10-html-spec, `parse-errors.md` finding 3). An evidence record recommended it and it was never built. No requirement states it.
- **A design or decision the build departed from**: separate-artifact-kinds' "no kind declared" diagnostic, project-bootstrap-cli's "The version gate checks; it does not write", reject-change-markup-inside-a-unit's decision 3, and record-how-a-risk-was-settled's decisions 5 and 8. These are designs, not requirements, and the `cannot` list's first row covers them as a class.
- **A removed requirement counted as stated** (2026-08-31-sync-a-change-into-the-accepted-contract). Accepted toolchain-packaging now settles this: a removal reaches the contract when no accepted specification holds its identifier. Likewise, record-a-renamed-requirement's point that "The comparison is of identifiers" says nothing of headings is answered: the accepted scenario now names "the headings of a stated rename".
- **`review/unverified-requirement` firing although scenarios are verified** (2026-09-29-keep-evidence-out-of-document-discovery). This matches document-validation's rule, which asks whether a verification names the requirement.
- **Scenario wording raised only while judging a verification**: log-a-runbook-run's "under the strictest setting available", where `run` has no strictness option, and record-a-partial-verification's "nothing is reported about that requirement".
- **contract-authority's "already measured that divergence" claim, which rests on no evidence file.** Review settled it as a question about links.
- **Wrong claims in records and commit messages**: content-address-runtime-assets' deviation 1, the `5f7e5cf` message in install-the-full-workflow-set, and task 1.1's note in migrate-an-openspec-project.
- **Spaced literals collapsing in inline `<code>`** (2026-10-08-record-a-task-done-differently). This affects rendering only: the bytes and the model keep the spaces, and no requirement is cited.
