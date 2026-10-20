## ADDED Requirements

### Requirement: Change markup in or around a semantic unit is rejected

Validation SHALL reject a document in which an `ins` or a `del` element sits inside a semantic unit, encloses one, or is one because its own class names the unit. These two elements are HTML's change markup: `del` marks text an edit removed, and `ins` marks text an edit added. An element of the same name inside SVG or MathML belongs to that language and is not change markup. Each such element SHALL be reported as an error under a rule identifier of its own. The diagnostic SHALL name the element and the unit it sits in or encloses, and SHALL quote the element's text, so that an author can find the edit in a long statement.

The reason is that the projection gives change markup no meaning. An element that carries no reserved class is presentational, and its text contributes to whichever field encloses it. So a deleted phrase in a requirement's statement becomes part of the normative statement, run together with the words inserted beside it. A deleted paragraph becomes a second statement. A deleted link still counts as a reference. A unit wrapped whole in `del` projects as a unit the document still states. In each case the author reached for the right HTML and wrote a wrong contract, and nothing reported it.

**It SHALL be an error and not a warning.** A warning reports a document that is incomplete or needs review, and it never gates work. Here the model is wrong: it states, as current, words the author withdrew. A document whose model says what its author did not mean does not conform, and a report that let it through would be read as saying it does.

**It SHALL cover `ins` as well as `del`.** An inserted word does no harm to the model, because its text is current text and the projection keeps it. It is rejected anyway, because a unit that marks one side of an edit and not the other states half of a diff. A reader sees words marked as new and cannot see what they replaced.

The rule SHALL apply to every unit in every kind, and whatever contract version the document declares. A document SHALL NOT exempt itself from the rule by declaring an earlier version, because a rule a document can switch off protects nothing.

Change markup outside every unit SHALL NOT be reported by this rule. A unit is where a document states its contract, and prose outside every unit projects into no unit's field. A document MAY mark a wording change with `del` and `ins` in that prose, as a proposal showing a before and an after would.

An element that only draws a line through text, such as `s` or the obsolete `strike`, SHALL NOT be reported by this rule. HTML defines `s` as content that is no longer accurate or relevant, not as an edit to the document, and it stays presentational.

#### Scenario: A deleted phrase and an inserted word in one statement
- **GIVEN** a requirement whose statement is `The spellings are fixed: <code>list</code>, <code>status</code>, <code>sync</code>, <code>instructions</code>, and <del>nothing else</del><ins><code>store</code></ins> SHALL name them.`
- **WHEN** the document is validated
- **THEN** validation fails with one error for the `del` and one for the `ins`, each naming the requirement

#### Scenario: A deleted paragraph in a requirement
- **WHEN** a requirement holds a paragraph wrapped in `del`
- **THEN** validation fails and names the requirement, because the paragraph would otherwise be one of its statements

#### Scenario: An inserted word with nothing marked as removed
- **WHEN** a unit's text contains an `ins` element and no `del` element
- **THEN** validation fails, because the unit marks one side of an edit

#### Scenario: Change markup in a nested unit
- **WHEN** a scenario's entry contains a `del` element
- **THEN** validation fails and names the scenario

#### Scenario: A unit wrapped in change markup
- **WHEN** a `del` element encloses a whole requirement
- **THEN** validation fails and names the requirement, because the requirement would still project as one the document states

#### Scenario: A change element that is itself a unit
- **WHEN** a `del` element's own class names a unit, such as `<del class="decision">`
- **THEN** validation fails and names that unit, because it would still project as one the document states

#### Scenario: Any unit of any kind
- **WHEN** a decision, task, evidence, risk, or verification unit carries change markup, in a document of any kind
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
