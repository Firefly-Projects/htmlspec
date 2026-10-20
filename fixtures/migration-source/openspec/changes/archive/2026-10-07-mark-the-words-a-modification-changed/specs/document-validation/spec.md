## MODIFIED Requirements

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
