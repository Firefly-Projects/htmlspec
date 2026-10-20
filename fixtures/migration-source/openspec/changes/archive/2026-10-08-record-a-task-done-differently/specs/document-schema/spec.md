## MODIFIED Requirements

### Requirement: A task declares completion state

A task unit SHALL declare whether it is complete in a machine-readable attribute, distinct from any visual checkbox. Completion state SHALL be one of `todo`, `in-progress`, `done`, or `dropped`. A dropped task SHALL state why.

**A `done` task MAY state a deviation: what was done, where that departs from the task's words.** The deviation SHALL be stated in one paragraph marked `data-field="deviation"`, and SHALL project into the task's `deviation` field. A task that states a deviation is `done`, because the work happened. A `done` task that states no deviation SHALL be read as done as its words say.

The reason is that none of the four states says "done, but not as written", and work lands that way. Marked `done` with a note, such a task claims its words were carried out, because a note is prose that no field and no count reads. Marked `dropped`, it claims the work did not happen, which is false. The deviation keeps both facts: the state says the work is finished, and the deviation says how it was done.

A task's words SHALL be kept as written when it states a deviation. They record what was planned, and designs, evidence, and later changes cite a task by them. The deviation is stated beside the words and SHALL NOT replace them.

The following SHALL make a document invalid:

- a `deviation` paragraph in a task whose state is `todo` or `in-progress`, because nothing has been done that could differ from the words;
- a `deviation` paragraph in a task whose state is `dropped`, because a dropped task's work did not happen and its reason already states why. A task whose work partly happened is `done`, and its deviation states which part;
- more than one `deviation` paragraph in one task, because a task has one account of what was done, and the projection does not choose between two.

A `deviation` paragraph in any unit other than a task names no field of that unit, and is invalid for that reason.

#### Scenario: Reporting progress
- **WHEN** a consumer reads a document containing ten tasks of which four are `done`
- **THEN** the consumer can report progress without parsing prose or rendered markup

#### Scenario: Dropped task without a reason
- **WHEN** a task declares state `dropped` and gives no reason
- **THEN** the document is invalid

#### Scenario: A task done another way than its words say
- **WHEN** a task declares state `done` and carries one `data-field="deviation"` paragraph
- **THEN** the document is valid, the task is `done`, and a consumer reads what was done from the task's `deviation` field

#### Scenario: A task done as its words say
- **WHEN** a task declares state `done` and carries no `deviation` paragraph
- **THEN** its `deviation` field is absent, and the task is read as done as its words say

#### Scenario: A deviation on a task not yet done
- **WHEN** a task whose state is `todo` or `in-progress` carries a `deviation` paragraph
- **THEN** the document is invalid

#### Scenario: A deviation on a dropped task
- **WHEN** a task whose state is `dropped` carries a reason and a `deviation` paragraph
- **THEN** the document is invalid, because a dropped task's work did not happen

#### Scenario: Two deviations in one task
- **WHEN** a `done` task carries two `deviation` paragraphs
- **THEN** the document is invalid

#### Scenario: A deviation outside a task
- **WHEN** a requirement, scenario, decision, evidence, risk, or verification unit carries a `data-field="deviation"` paragraph
- **THEN** the document is invalid, because the field belongs to a task
