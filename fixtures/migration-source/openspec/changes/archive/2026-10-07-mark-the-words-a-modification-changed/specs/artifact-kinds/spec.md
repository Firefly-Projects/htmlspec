## ADDED Requirements

### Requirement: A marked modification reaches the accepted contract as its current text

Where a change's specification marks words in a requirement or a scenario it modifies, applying that specification's contract to the project's accepted specifications SHALL give the accepted unit its current text: the content of each `ins` element without the element, and nothing of each `del` element.

**The accepted specification SHALL NOT carry change markup.** The accepted contract states what the project requires now. Which words a change added and which it deleted is history, and it belongs to the archived change that made the edit, whose specification keeps its marks. This is the arrangement "A rename reaches the accepted contract as a new heading on the same identifier" makes for a heading, where the accepted specification does not carry the `renamed-from` paragraph.

A mark carried into the accepted contract would go on claiming that a word is new for as long as the text stands. This project's history holds the case. `2026-09-18-establish-a-runbook` bolded the words it added to two project-bootstrap requirements, the merge copied the bold into the accepted specification, and it is still there, where it now reads as emphasis. The word the same edit deleted was never marked at all. A `del` carried across would show the next change's author a deleted phrase on the page of the contract, ready to be copied back as current.

Validation holds the second half of this: an accepted specification that carries change markup in a unit is rejected, under document-validation's "Change markup in or around a semantic unit is rejected". The first half is the merge, which remains the agent's work under "Every synthesis and every contract update is agent-driven". This requirement states what a correctly applied modification leaves behind, not a program that applies it.

**The current text the merge writes is the text the change's specification projects.** document-schema's "A deleted word is recorded apart from what a unit states" keeps deleted words out of every statement and entry, and keeps inserted words in. So a comparison of a change's modified requirement with the accepted one, made on projected text, finds a merge that wrote the current text equal to the change. It finds a merge that kept a deleted word, or left out an inserted one, unequal. The marks need no reading of their own to be compared.

#### Scenario: An inserted word applied
- **GIVEN** a change whose specification states requirement `req-x` marked `modified`, with `<ins><code>sync</code>, </ins>` in one statement
- **WHEN** its contract is applied to the accepted specifications
- **THEN** the accepted requirement `req-x` states `sync` in that statement, as ordinary text, and carries no `ins` element

#### Scenario: A deleted word applied
- **GIVEN** a change whose specification states a requirement marked `modified`, with `<del>and </del>` in one statement
- **WHEN** its contract is applied
- **THEN** the accepted requirement's statement no longer holds that word, and the accepted specification carries no `del` element

#### Scenario: A deleted paragraph applied
- **GIVEN** a change whose specification states a requirement marked `modified`, with one paragraph wrapped in `del`
- **WHEN** its contract is applied
- **THEN** the accepted requirement no longer holds that paragraph

#### Scenario: Markup left in the accepted contract
- **WHEN** a merge leaves an `ins` or a `del` element in a unit of an accepted specification
- **THEN** validation of that accepted specification fails

#### Scenario: The history is read from the archive
- **WHEN** a reader wants to know which words a change added to a requirement, or deleted from it
- **THEN** the archived change's specification marks them, and the accepted specification does not

#### Scenario: A marked modification compared with the accepted text
- **GIVEN** a change whose specification marks an inserted word and a deleted word in a modified requirement, and an accepted requirement to which the merge gave the current text
- **WHEN** the two are compared as projected text
- **THEN** they are equal

#### Scenario: A merge that kept a deleted word
- **GIVEN** the same change, and an accepted requirement that still holds the deleted word
- **WHEN** the two are compared as projected text
- **THEN** they differ
