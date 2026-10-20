## Purpose

Defines the structural and safety rules a specification document must satisfy to be accepted, along with the diagnostics a validator reports, so that a document is both reliably machine-editable and safe to open.

## Requirements

### Requirement: Structural conformance is enforced

Validation SHALL reject any document that violates the element contract. This includes missing or duplicate identifiers, identifier prefixes that do not match the unit type, requirements without scenarios, scenarios outside requirements, missing normative statements, malformed scenario bodies, and attribute values outside their permitted sets.

#### Scenario: Duplicate identifier rejected
- **WHEN** a document contains two units sharing one identifier
- **THEN** validation fails and reports both locations

#### Scenario: Requirement without a scenario rejected
- **WHEN** a requirement contains no scenario
- **THEN** validation fails and identifies the requirement

#### Scenario: Conforming document accepted
- **WHEN** a document satisfies every structural rule
- **THEN** validation succeeds and reports no structural errors

### Requirement: An upstream edit marks dependents for review rather than gating work

Validation SHALL report a dependent unit as needing review when the unit it references changes materially. Such a report SHALL NOT block continued work, SHALL NOT impose a phase gate, and SHALL NOT require regeneration of the dependent artifact. Exploration, proposal, design, implementation, and verification SHALL be able to loop.

#### Scenario: Requirement revised during implementation
- **GIVEN** tasks and verification that reference a requirement
- **WHEN** that requirement's obligation changes while implementation is underway
- **THEN** validation reports the referencing units as needing review, and work continues

#### Scenario: Reconciliation clears the report
- **WHEN** an agent reconciles the units reported as needing review
- **THEN** validation stops reporting them, and no artifact was regenerated wholesale

#### Scenario: Stale reference is reported, not fatal
- **WHEN** a dependent unit still resolves but describes behavior the referenced requirement no longer states
- **THEN** validation reports the inconsistency as needing review rather than as a structural error

### Requirement: Cross-references must resolve

Validation SHALL verify that every reference to a semantic unit resolves to a unit that exists. This includes fragment links within the document, and references from tasks, verification, evidence, and risks to the requirements and scenarios they address.

A reference resolves in the places that document-schema's "Cross-references address units by identifier" makes reachable. **Validation SHALL read the other documents a reference can reach for the identifiers they declare, and SHALL report nothing about them.** A diagnostic belongs to the document being validated. The documents it reaches are reported when they are validated themselves, so a defect in one of them is reported once, against the document that has it.

A reference that resolves nowhere SHALL be reported against the unit that makes it, and the report SHALL name the places that were searched. "No unit in the document declares that identifier" is true of a document that reaches only itself. For a task in a change it would hide that the change's specification and the accepted specifications were searched too.

**A reachable document that cannot be read, or whose markup needs the parser to recover, declares nothing a reference can rely on.** A reference that resolves in no other place SHALL be reported, and the report SHALL name that document as unread. It SHALL NOT claim that no unit declares the identifier. A document with more than one reading has no determinate content, so identifiers taken from it are a guess. And a report that no unit declares the identifier would send the author to add a unit that may already be there.

A change that has no specification yet is a change in progress. Its specification is then a place that declares nothing, and it SHALL NOT be reported as a document that could not be read.

#### Scenario: Dangling task reference
- **WHEN** a task references a requirement identifier that does not exist
- **THEN** validation fails and reports the task and the unresolved identifier

#### Scenario: Broken fragment link
- **WHEN** a document links to a fragment with no matching element
- **THEN** validation fails and reports the link

#### Scenario: Reference to a removed requirement
- **WHEN** a task or verification references a requirement marked removed
- **THEN** validation fails

#### Scenario: A reference resolved by the change's specification
- **GIVEN** a change whose specification declares `req-x`, and also declares a requirement with no scenario
- **WHEN** the change's tasks document, whose task names `req-x` in its `data-refs`, is validated
- **THEN** validation of the tasks document succeeds, and it reports nothing about the reference or about the specification

#### Scenario: A dangling reference in a change says where it looked
- **WHEN** a task in a change names in its `data-refs` an identifier that no reachable place declares
- **THEN** validation fails and reports the task, the identifier, and the places that were searched

#### Scenario: A reachable document that cannot be read
- **GIVEN** a change whose specification's markup needs the parser to recover
- **WHEN** a task in that change names in its `data-refs` a requirement that no other reachable place declares
- **THEN** validation of the tasks document fails, and the report names the specification as the document that could not be read

#### Scenario: A change with no specification yet
- **GIVEN** a change that has no specification, and an accepted specification declaring `req-y`
- **WHEN** a task in that change names `req-y` in its `data-refs`
- **THEN** the reference resolves, and nothing is reported about the missing specification

### Requirement: Markup must be deterministically parseable

Validation SHALL reject a document whose markup cannot be parsed into a single unambiguous structure. Constructs that different parsers would interpret differently SHALL be rejected rather than accepted with a guess.

A conforming document SHALL produce zero recovered parse errors by default. Any recovered parse error SHALL fail validation unless the project permits that specific recovery.

#### Scenario: Improperly nested elements
- **WHEN** a document contains overlapping or improperly nested elements
- **THEN** validation fails and reports the location

#### Scenario: Ambiguous construct
- **WHEN** a document contains markup whose interpretation depends on parser error recovery
- **THEN** validation fails rather than silently accepting one interpretation

### Requirement: Permitted parser recoveries are enumerated and pinned

The project SHALL pin an exact parser version. Every permitted recovery SHALL be enumerated individually, SHALL carry a stable internal diagnostic code owned by this project, and SHALL be justified by a fixture that demonstrates the recovery is unambiguous across consumers.

Permitted recoveries SHALL NOT be derived by collecting whatever a sample of documents happens to produce. A parser version change SHALL require deliberate review of the enumeration before the new version is adopted.

#### Scenario: Diagnostic codes are owned by this project
- **GIVEN** a parser that reports a recovery as free text rather than as a typed code
- **WHEN** validation reports that recovery
- **THEN** it reports this project's own stable diagnostic code, so a reworded parser message does not change the reported code

#### Scenario: Parser rewords a message
- **WHEN** a new parser version changes the wording of a recovery message
- **THEN** the project's checks fail rather than silently widening or narrowing what documents are accepted

#### Scenario: Ambiguous recovery is not permitted
- **WHEN** a recovery is proposed for the permitted enumeration and no fixture shows it is unambiguous across consumers
- **THEN** the recovery is not permitted

#### Scenario: Coarse parser message covering two faults
- **GIVEN** a parser that reports two distinct faults with one identical message
- **WHEN** the enumeration is built
- **THEN** it distinguishes the two faults by parser state, or it permits neither

### Requirement: Remote and hidden network dependencies are rejected

Validation SHALL reject any document that depends on a resource outside the project. This includes remote scripts, stylesheets, fonts, images, and iframes, and any construct capable of initiating a network request when the document is opened.

#### Scenario: Remote script rejected
- **WHEN** a document references a script by absolute URL
- **THEN** validation fails and reports the reference

#### Scenario: Remote font rejected
- **WHEN** a document's styling loads a font from a remote origin
- **THEN** validation fails

#### Scenario: Concealed request rejected
- **WHEN** a document contains a construct that would issue a network request without a visible remote reference, such as a prefetch directive or a beacon
- **THEN** validation fails

#### Scenario: Local asset accepted
- **WHEN** a document references the project's shared runtime by relative path
- **THEN** validation succeeds

### Requirement: Code examples cannot execute

Validation SHALL ensure that example code shown in a document is inert. Code presented as content SHALL NOT be capable of executing as part of the page, and SHALL NOT be able to terminate its container and inject markup into the document.

The security property is the absence of element descendants. A code element's children SHALL contain text and comment nodes only. Validation SHALL NOT require exactly one text node, because a parser splits text around a comment and an empty example has no children at all.

#### Scenario: Script shown as an example
- **WHEN** a document displays a JavaScript snippet as an example
- **THEN** validation succeeds and the snippet does not execute when the document is opened

#### Scenario: Empty example accepted
- **WHEN** a code element has no children
- **THEN** validation succeeds, because an empty example carries no element descendant

#### Scenario: Text split by a comment accepted
- **WHEN** a parser represents a code element's content as two text nodes separated by a comment
- **THEN** validation succeeds, because no child is an element

#### Scenario: Presentational element inside an example rejected
- **WHEN** a code element contains a child element, including a purely presentational one
- **THEN** validation fails, because the example is no longer inert text

#### Scenario: Container escape rejected
- **WHEN** an example's content would close its containing element and introduce live markup
- **THEN** validation fails and reports the example

#### Scenario: Executable script accepted only from the pinned runtime
- **WHEN** a document contains an executable script that is not part of the pinned runtime
- **THEN** validation fails

### Requirement: Visualization specifications are validated

Validation SHALL check every embedded diagram or chart specification against its own format. An invalid specification SHALL be reported at authoring time rather than failing silently in the browser.

#### Scenario: Invalid chart specification
- **WHEN** a document embeds a chart specification that its format rejects
- **THEN** validation fails and reports the location and the underlying error

#### Scenario: Invalid diagram source
- **WHEN** a document embeds diagram source that cannot be parsed by the diagram library
- **THEN** validation fails

#### Scenario: Chart data is local
- **WHEN** a chart specification loads its data from a remote URL
- **THEN** validation fails, because the document would not render offline

### Requirement: No-JavaScript readability is enforced

Validation SHALL reject a document whose semantic content is not present in its markup. Content that only appears after scripts run SHALL be treated as absent.

#### Scenario: Script-injected requirement rejected
- **WHEN** a requirement's text is supplied by a script rather than present in the markup
- **THEN** validation fails

#### Scenario: Diagram without a fallback
- **WHEN** a document contains a diagram with no source or textual description present in the markup
- **THEN** validation fails

### Requirement: Diagnostics are actionable and machine-readable

Every validation failure SHALL report the rule violated, the location within the document, and the offending identifier or content where one applies. Diagnostics SHALL be available in a machine-readable form so that agents and continuous integration can consume them without parsing prose.

#### Scenario: Human-readable output
- **WHEN** a contributor validates a failing document
- **THEN** the output names the rule, the location, and what to fix

#### Scenario: Machine-readable output
- **WHEN** an agent validates a failing document and requests structured output
- **THEN** it receives each diagnostic as structured data with a stable rule identifier

#### Scenario: Stable rule identifiers
- **WHEN** the same violation is reported across versions of the validator
- **THEN** the rule identifier is unchanged, so suppressions and tests remain valid

### Requirement: All violations are reported in one pass

Validation SHALL report every violation it can detect in a single run rather than stopping at the first. Where one failure prevents further analysis, validation SHALL report what it could determine and state that analysis was curtailed.

#### Scenario: Multiple independent violations
- **WHEN** a document has a duplicate identifier, a dangling reference, and a remote script
- **THEN** all three are reported in one run

#### Scenario: Unparseable document
- **WHEN** a document cannot be parsed at all
- **THEN** validation reports the parse failure and states that no further rules were evaluated

### Requirement: Violations are classified by severity

Every rule SHALL be classified as an error or a warning. An error SHALL cause validation to fail. A warning SHALL be reported without failing validation. Rules concerning structural conformance, unresolved references, executable content, and network dependencies SHALL be errors.

#### Scenario: Warning does not fail validation
- **WHEN** a document triggers only warnings
- **THEN** validation succeeds and the warnings are reported

#### Scenario: Warnings can be made fatal
- **WHEN** a contributor requests strict validation
- **THEN** warnings are treated as errors, so continuous integration can enforce them

#### Scenario: Safety rules cannot be downgraded
- **WHEN** a document contains a remote script or executable content
- **THEN** the violation is reported as an error and cannot be reduced to a warning

### Requirement: An asset reference must match the bytes it names

Validation SHALL reject a document whose asset reference does not resolve to stored bytes whose hash equals the hash named by that reference. A reference naming bytes the project does not store SHALL be reported as unresolved. A reference resolving to bytes whose hash differs from the name SHALL be reported as a mismatch, and the two SHALL be distinguishable diagnostics because they have different causes.

This rule SHALL apply only to a document validated within a project that has an asset store. A document validated outside such a project SHALL NOT have its asset references resolved, and SHALL NOT be rejected for naming bytes that no store holds. The rule states a relationship between a document and the bytes beside it, and where there are no bytes there is no relationship to violate.

Where a document's asset reference ascends out of the document's own directory, and no asset store is found in any directory above it, validation SHALL report that reference. Such a reference asserts that a store sits above the document, and the absence of one contradicts the reference rather than placing the document outside the rule's scope.

A reference that does not ascend SHALL remain unreported when no store is found. This is what keeps a document whose references are symbolic — a conformance fixture, or a document written to be read rather than rendered — outside the rule, and the distinction is exact: a fixture's references name a store beside the document that has never existed, while an ascending reference names a store the document was generated against.

A document SHALL NOT be able to exempt itself. The scope SHALL be determined by whether a store is present and by the shape of the reference, never by a declaration inside the document, because a rule a document can switch off protects nothing. Reference shape is not such a declaration: it is written by whatever generated the document, and changing it to silence the rule breaks the reference it describes.

This rule SHALL be an error rather than a warning, and SHALL NOT be reducible to a warning. A document that renders against bytes other than the ones it names is misreporting its own provenance.

#### Scenario: Reference matches stored bytes
- **WHEN** every asset reference in a document names the hash of the bytes stored at its location
- **THEN** validation succeeds

#### Scenario: Document validated with no store present
- **GIVEN** a document that is not part of a project with an asset store, such as a conformance fixture whose references are symbolic
- **WHEN** it is validated
- **THEN** its asset references are not resolved, and their targets being absent is not a violation

#### Scenario: An ascending reference with no store above it
- **GIVEN** a document whose asset references ascend out of its own directory, such as a change directory copied away from the project that holds its store
- **WHEN** it is validated
- **THEN** validation reports the ascending references, because each one names a store that is not there

#### Scenario: Strict validation does not accept dead ascending references
- **GIVEN** a document carrying only ascending asset references and no store in any directory above it
- **WHEN** it is validated in strict mode
- **THEN** validation does not report the document as accepted

#### Scenario: A document cannot exempt itself
- **WHEN** a document declares that its asset references are not to be resolved, by any marker it carries
- **THEN** the declaration has no effect, and a document inside a project with a store is checked regardless

#### Scenario: Reference to bytes the project does not store
- **WHEN** a document references an asset hash that no stored asset matches
- **THEN** validation fails and reports the reference as unresolved

#### Scenario: Stored bytes do not match the name
- **GIVEN** an asset location whose contents were replaced with different bytes
- **WHEN** a document referencing that location is validated
- **THEN** validation fails and reports a mismatch rather than an absent asset

#### Scenario: Stale reference cannot pass
- **GIVEN** a document authored against one revision of a stylesheet
- **WHEN** it is stored beside a different revision of that stylesheet
- **THEN** validation fails, because the reference and the bytes disagree

#### Scenario: Mismatch cannot be downgraded
- **WHEN** a contributor requests that asset mismatches be treated as warnings
- **THEN** the violation is still reported as an error

### Requirement: A project must not store one asset's bytes more than once

Validation SHALL reject a project that stores identical asset bytes at more than one location. The report SHALL name every location holding the duplicated bytes, so a contributor can tell which copy to remove.

Two assets whose bytes differ SHALL NOT be reported as duplicates, however similar their names, versions, or purposes.

This rule takes a project as its subject rather than a document, so it SHALL be evaluated where a project is in view. A directory holding no asset store has nothing to duplicate, and SHALL NOT be reported.

**A project's stores SHALL be the asset stores within the project's own directory, `htmlspec/`, and no others.** The comparison across stores SHALL be made among those stores alone. A repository holding a project can hold other stores that are not the project's — a conformance fixture that is a project of its own, a toolchain's source copy of the runtime, a vendored copy another tool reads — and bytes those stores share with the project's are not the project storing its bytes twice. The subject of this rule is one project, and a comparison that reaches past the project's directory has changed its subject to the repository.

A store outside the project's directory SHALL still be checked on its own, so a store that holds identical bytes at two of its own locations is reported wherever it lives. It SHALL NOT be compared with the project's stores or with any other store.

The report SHALL state which stores it treated as the project's and which it checked on their own, so a contributor can tell a duplication that is not the project's from a check that did not run.

Where no project is in view, every store SHALL be checked on its own, as before.

#### Scenario: Duplicate copies of one library
- **GIVEN** a project storing the same library bytes in two locations
- **WHEN** the project is validated
- **THEN** validation fails and names both locations

#### Scenario: A copy per document is rejected
- **GIVEN** a project in which each document carries its own copy of the runtime
- **WHEN** the project is validated
- **THEN** validation fails, because the shared-runtime obligation is violated

#### Scenario: Different revisions are not duplicates
- **GIVEN** a project storing two revisions of one stylesheet whose bytes differ
- **WHEN** the project is validated
- **THEN** validation succeeds, because distinct bytes are not a duplicate

#### Scenario: One store serving many documents
- **GIVEN** many documents referencing one stored copy of each library
- **WHEN** the project is validated
- **THEN** validation succeeds

#### Scenario: A project beside a fixture that carries its own store
- **GIVEN** a repository holding a project, and a conformance fixture elsewhere in the repository whose store holds bytes identical to the project's
- **WHEN** the project's asset topology is checked
- **THEN** no duplication is reported, and the report names the fixture's store as checked on its own

#### Scenario: A project beside a toolchain's source copy of the runtime
- **GIVEN** a repository whose root holds a store of the runtime outside the project's directory, and a project whose store holds the same bytes
- **WHEN** the project's asset topology is checked
- **THEN** no duplication is reported

#### Scenario: Two stores inside the project
- **GIVEN** a project whose own directory holds two stores with identical bytes, such as one beside a change's documents
- **WHEN** the project's asset topology is checked
- **THEN** the duplication is reported and names both locations

#### Scenario: A store outside the project that duplicates within itself
- **GIVEN** a store outside the project's directory holding identical bytes at two of its own locations
- **WHEN** the project's asset topology is checked
- **THEN** that duplication is reported, and the store is not compared with the project's

### Requirement: Asset rules are checked as part of the project's check suite

The asset reference and duplication rules SHALL be evaluated by the project's single check command, so a contributor and continuous integration verify the same asset topology. A project whose documents validate individually but whose asset topology violates these rules SHALL fail the check suite.

#### Scenario: Topology regression fails the suite
- **WHEN** a change introduces a second copy of a stored asset
- **THEN** the check command fails and identifies the duplicated locations

#### Scenario: Per-document validation is not sufficient
- **GIVEN** a project whose every document passes validation on its own
- **WHEN** two documents together hold duplicate asset bytes
- **THEN** the check command still fails, because the violation is a property of the project rather than of one document

### Requirement: A specification must not carry task units

Validation SHALL reject a `specification` document carrying a task unit. The report SHALL name the task and say that a specification records the obligation a task satisfied rather than the work that satisfied it.

This SHALL be the **only** placement rule in the vocabulary. No other combination of unit and kind SHALL be prohibited, and validation SHALL report nothing about any other. In particular, a requirement unit in a `proposal`, a task unit in a `design`, and a decision unit in a `tasks` document SHALL all be accepted without diagnostic.

The rule is an error because a specification is read by tools and by later changes as the statement of the contract, and a task appearing there would offer work as though it were an obligation.

#### Scenario: A task in a specification
- **WHEN** a specification carries a task unit
- **THEN** validation fails and names the unit

#### Scenario: A requirement in a proposal
- **WHEN** a proposal carries a requirement unit
- **THEN** validation reports nothing about it, and the document is accepted

#### Scenario: A task in a design
- **WHEN** a design carries a task unit
- **THEN** validation reports nothing about it, and the document is accepted

#### Scenario: Prose in any kind
- **WHEN** a document of any kind carries headings, paragraphs, and figures with no unit markup
- **THEN** no placement diagnostic is reported, because prose belongs everywhere

### Requirement: A tasks document must carry at least one task

Validation SHALL reject a `tasks` document carrying no task unit. The stage exists to enumerate work, and a tasks document that enumerates none records nothing.

This is a **completeness** rule and not a placement rule. It constrains what the document must contain rather than which units are permitted in it, so a tasks document carrying other units alongside its tasks SHALL NOT be reported for them.

#### Scenario: An empty tasks document
- **WHEN** a tasks document carries no task unit
- **THEN** validation fails

#### Scenario: A tasks document carrying other units
- **WHEN** a tasks document carries its tasks and also a decision unit
- **THEN** validation reports nothing about the decision, because the rule constrains what must be present rather than what is permitted

### Requirement: A specification requirement without verification is reported as incomplete

Validation SHALL report a requirement in a `specification` document that no verification names. A specification is an as-built record, and a requirement with nothing attesting to it records an intention rather than a result.

This SHALL be **advisory** — a warning, never an error. Verification arrives as work completes, so a specification in progress legitimately has requirements not yet verified, and a rule that failed the document would gate the work that produces the verification.

A requirement that a verification names SHALL NOT be reported, whatever outcome that verification records, `partial` included. The report asks whether a verification names the requirement, not what the verification found. A requirement whose check failed, was blocked, or covered only part of it is named by a record that says so, and reporting it as unverified would treat an unwelcome result as a missing one. That would teach an author to record a pass, which is the reason runbook-execution's "A run's conformance to its runbook's contract is reported and never gated" gives for a run. This rule SHALL apply only to the `specification` kind: a requirement unit appearing in any other kind SHALL NOT be checked for verification, because it is not part of the contract there.

#### Scenario: A requirement with no verification
- **WHEN** a specification states a requirement that no verification names
- **THEN** validation reports a warning, and the document is still accepted

#### Scenario: A fully verified specification
- **WHEN** every requirement in a specification is named by a verification
- **THEN** nothing is reported

#### Scenario: A requirement whose check failed
- **WHEN** a specification states a requirement whose only verification records the outcome `failed`
- **THEN** nothing is reported about that requirement, because a verification names it and its outcome states the result

#### Scenario: A requirement verified only in part
- **WHEN** a specification states a requirement whose only verification records the outcome `partial`
- **THEN** nothing is reported about that requirement, because a verification names it, and that verification states what it did not check

#### Scenario: Verification is not a gate
- **WHEN** a contributor validates a specification whose verification is not yet recorded
- **THEN** validation succeeds, because recording verification is the work in progress rather than a violation

#### Scenario: An unverified requirement in a proposal
- **WHEN** a proposal carries a requirement unit that no verification names
- **THEN** nothing is reported, because a proposal's units are not part of the contract

### Requirement: The contract rules apply to every kind

The rules that make a document readable, parseable, and offline SHALL apply to all four kinds without relaxation. Freedom of structure at the proposal and design kinds SHALL NOT extend to markup that cannot be parsed unambiguously, a reference that resolves to nothing, a remote or executable dependency, or an asset reference that disagrees with the bytes beside it.

A kind decides how much *shape* validation requires. It SHALL NOT decide whether a document opens.

A kind SHALL NOT decide what a reference can reach, either. That is decided by where a document sits, so a change's proposal, design, and tasks reach the same documents.

#### Scenario: A proposal with a remote script
- **WHEN** a proposal loads a script from outside the project
- **THEN** validation fails, exactly as it does for any other kind

#### Scenario: A design with a dangling reference
- **WHEN** a design references an identifier that no unit declares, in the design or in any document it can reach
- **THEN** validation fails, because a pointer to nothing is a defect at every stage

#### Scenario: A design references its change's specification
- **WHEN** a decision in a change's design names in its `data-refs` a requirement that the change's specification declares
- **THEN** the reference resolves, exactly as it would from a task in the same change

#### Scenario: A proposal that needs error recovery
- **WHEN** a proposal's markup requires the parser to recover
- **THEN** validation fails, because a document with more than one reading has no determinate content whatever its kind

### Requirement: Change markup in or around a semantic unit is rejected

Validation SHALL reject a document in which an `ins` or a `del` element sits inside a semantic unit, encloses one, or is one because its own class names the unit, except in the one place this requirement permits it. These two elements are HTML's change markup: `del` marks text an edit removed, and `ins` marks text an edit added. An element of the same name inside SVG or MathML belongs to that language and is not change markup. Each such element SHALL be reported as an error under a rule identifier of its own. The diagnostic SHALL name the element and the unit it sits in or encloses, and SHALL quote the element's text, so that an author can find the edit in a long statement. It SHALL also say why the markup is not permitted where it sits.

**Change markup SHALL be permitted inside a requirement or a scenario marked `modified`, in a change's specification.** There, `del` marks words the change deleted from the accepted text and `ins` marks words it added, and document-schema's "A deleted word is recorded apart from what a unit states" keeps the deleted words out of every statement and entry. A change's specification is the one document whose modified units restate accepted text as the change edited it, for the merge to apply. So it is the one place where "these words are new" and "these words were deleted" are claims about something a reader can check: the accepted text before the merge. The markup is permitted there, not required. A modified unit that marks no words is valid, because whether its marks are complete can only be checked against the accepted text, which one document does not hold.

The permission SHALL be decided by the unit the markup's text belongs to, which is the nearest unit that encloses it. So a scenario marked `modified` inside a requirement marked `modified` may carry markup. A scenario marked `unchanged` inside that requirement may not, because it states that its text is the accepted text.

The permission SHALL NOT extend to any of the following, in a change's specification or anywhere else:

- a unit marked `added`, `removed`, or `unchanged`. Every word of a new unit is new, so there is no accepted text to mark against. A removed unit is withdrawn whole, and its change state says so. An unchanged unit states that none of its text changed;
- a decision, evidence, risk, or verification unit, whatever its change state. An accepted specification holds requirements and scenarios, so no other unit has accepted text that a change's marks could be read against;
- a task, in any document. A task gives up its words by being dropped with a reason, and its state already shows that;
- markup in a unit's heading, or in the paragraph stating the heading a requirement replaced. A heading is a name and not a statement. A requirement's new name is stated with a `renamed-from` paragraph, which the merge and its comparison already read, and a scenario keeps its identifier when its heading changes;
- markup that encloses a unit, or is one because its own class names the unit, because a unit's change state already says whether it is new or withdrawn.

A change's specification SHALL be recognized by where the document sits: the specification directly within a change's directory, whether the change is active or archived. A change's proposal, design, and tasks, an accepted specification, a runbook's documents, a file beneath a change's evidence, and a document outside any project SHALL NOT carry the permission. A document SHALL NOT be able to claim the permission by what it declares, because a change's specification and an accepted specification declare the same kind. A document outside any project sits in no change, so the rule applies to it without the permission.

The reason for the rule is that change markup states a history, and a unit has a history to state only where it restates accepted text as a change edited it. Anywhere else the markup says something false or empty. An accepted specification that kept a mark would go on claiming that a word is new. An unchanged scenario with a mark would claim both that its text is the accepted text and that part of it is not. A new unit would mark as new some of the words that are all new. Before document-schema's "A deleted word is recorded apart from what a unit states" gave `del` its meaning, the markup also corrupted the model. A deleted phrase ran into the statement beside the words that replaced it, a deleted paragraph became a second statement, and a deleted link still counted as a reference.

**It SHALL be an error and not a warning.** A warning reports a document that is incomplete or needs review, and it never gates work. Markup outside its one place is neither. The document states something false about the contract, and a document whose markup contradicts its own units does not conform. This rule is also the check that a merge left no markup in an accepted specification. As a warning, it would let every check a project can run accept an accepted contract that still shows a deleted phrase on its page.

**It SHALL cover `ins` as well as `del`, and permit them in the same place.** They are one vocabulary for one edit. An inserted word does no harm to the model, because its text is current text and the projection keeps it. It is rejected outside the permitted place anyway, because a word marked as new states an edit, and no edit of accepted text is being stated there. Inside the permitted place, an insertion with nothing marked as removed is a whole edit: a change that only adds a word to a list deletes nothing.

The rule SHALL apply to every unit in every kind, and whatever contract version the document declares. A document SHALL NOT exempt itself from the rule by declaring an earlier version, because a rule a document can switch off protects nothing.

Change markup outside every unit SHALL NOT be reported by this rule. A unit is where a document states its contract, and prose outside every unit projects into no unit's field. A document MAY mark a wording change with `del` and `ins` in that prose, as a proposal showing a before and an after would.

An element that only draws a line through text, such as `s` or the obsolete `strike`, SHALL NOT be reported by this rule. HTML defines `s` as content that is no longer accurate or relevant, not as an edit to the document, and it stays presentational.

#### Scenario: A deleted phrase and an inserted word in one statement
- **GIVEN** a requirement in an accepted specification whose statement is `The spellings are fixed: <code>list</code>, <code>status</code>, <code>sync</code>, <code>instructions</code>, and <del>nothing else</del><ins><code>store</code></ins> SHALL name them.`
- **WHEN** the document is validated
- **THEN** validation fails with one error for the `del` and one for the `ins`, each naming the requirement

#### Scenario: A deleted paragraph in a requirement
- **WHEN** a requirement marked `added` holds a paragraph wrapped in `del`
- **THEN** validation fails and names the requirement, because a requirement the change introduces has no accepted paragraph to delete

#### Scenario: An inserted word with nothing marked as removed
- **WHEN** a unit marked `unchanged` contains an `ins` element and no `del` element
- **THEN** validation fails, because the unit states that none of its text changed

#### Scenario: Change markup in a nested unit
- **GIVEN** a change's specification holding a requirement marked `modified`
- **WHEN** a scenario inside it that is marked `unchanged` has a `del` element in one of its entries
- **THEN** validation fails and names the scenario, because the scenario states that its text is the accepted text

#### Scenario: A unit wrapped in change markup
- **WHEN** a `del` element encloses a whole requirement, in any document, a change's specification included
- **THEN** validation fails and names the requirement, because a requirement's change state is how a document says it is withdrawn

#### Scenario: A change element that is itself a unit
- **WHEN** a `del` element's own class names a unit, such as `<del class="decision">`
- **THEN** validation fails and names that unit, because it would still project as one the document states

#### Scenario: Any unit of any kind
- **WHEN** a decision, task, evidence, risk, or verification unit carries change markup, in a document of any kind, wherever the document sits and whatever the unit's change state
- **THEN** validation fails

#### Scenario: A document declaring an earlier contract version
- **WHEN** a document declaring an earlier accepted contract version carries change markup inside a unit
- **THEN** validation fails, exactly as it does for a document declaring the current version

#### Scenario: Change markup outside every unit
- **GIVEN** a proposal whose prose outside every unit marks a wording change with `del` and `ins`
- **WHEN** it is validated
- **THEN** this rule reports nothing

#### Scenario: A struck-through phrase is not change markup
- **WHEN** a requirement's statement contains an `s` element
- **THEN** this rule reports nothing

#### Scenario: A word inserted into a modified requirement
- **GIVEN** a change's specification whose requirement marked `modified` states `… <code>list</code>, <code>status</code>, <ins><code>sync</code>, </ins><code>instructions</code>, and <code>store</code> name the operations OpenSpec also provides.`
- **WHEN** the document is validated
- **THEN** this rule reports nothing

#### Scenario: A word and a paragraph deleted from a modified requirement
- **GIVEN** a change's specification whose requirement marked `modified` has `<del>and </del>` inside one statement and a paragraph wrapped in `del`
- **WHEN** the document is validated
- **THEN** this rule reports nothing

#### Scenario: A modified scenario in a modified requirement
- **WHEN** a change's specification holds a scenario marked `modified` inside a requirement marked `modified`, and the scenario carries change markup in one of its entries
- **THEN** this rule reports nothing

#### Scenario: An archived change keeps its marks
- **WHEN** an archived change's specification carries change markup in a requirement marked `modified`
- **THEN** this rule reports nothing, because an archived change is still a change, and its marks are its record of the edit

#### Scenario: The same markup outside a change's specification
- **GIVEN** a requirement marked `modified` whose statement carries change markup
- **WHEN** it sits in an accepted specification, in a change's proposal, design, or tasks, in a runbook's document, beneath a change's evidence, or in a document outside any project
- **THEN** validation fails, because only a change's specification restates accepted text for the merge to apply

#### Scenario: Change markup in a unit's heading
- **WHEN** a change's specification holds a requirement marked `modified` with a `del` element in its heading
- **THEN** validation fails, and the diagnostic says that a requirement's new name is stated with a `renamed-from` paragraph
