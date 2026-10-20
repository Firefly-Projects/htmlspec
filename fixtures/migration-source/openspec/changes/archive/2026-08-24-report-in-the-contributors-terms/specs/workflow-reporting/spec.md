## ADDED Requirements

### Requirement: A report answers the contributor in their own terms

Every workflow SHALL end by reporting to the contributor who invoked it, and that report's first claim SHALL be stated in the vocabulary of what the contributor asked for rather than the vocabulary of the artifacts that hold it.

A report is the only part of a workflow a contributor is guaranteed to read. The documents a workflow produces serve the next agent and the archive, and a contributor opens them by choice. So the report SHALL carry the burden of showing that the request was understood, and a report whose every claim names a file, a document, a task, or a count cannot show that however faithfully it was written.

This obligation SHALL be one of register rather than a sequence of required parts. Workflows differ in what they produce: one begins work that does not exist, one acts on a change already present, one reaches a conclusion and deliberately writes nothing. A fixed order of parts would suit the first and distort the last, so what each workflow reports first SHALL be that workflow's own subject, expressed as the contributor would express it.

A report's first claim SHALL be a fact about what the contributor asked for. Reassurance is not a report, and neither is enthusiasm about progress. The obligation to speak in the contributor's terms SHALL NOT be read as license to persuade or to soften, because a report that reads warmly while withholding what happened is a worse failure than one that reads as bookkeeping.

Where an artifact already states the thing in the contributor's terms, the report SHALL surface that statement rather than describe the file containing it. An agent directed to summarize invents; an agent directed to surface quotes what is already written. A proposal that states what working looks like has already answered what a contributor is getting, and a report that describes the proposal instead of quoting it discards work already done.

The facts about artifacts — where a change landed, which files exist, what a check reported — SHALL remain available in a report. They SHALL appear as annotation rather than as subject, because a contributor sometimes needs them and never needs them first.

Where a workflow's boundary means the contributor does not yet have a working thing, the report SHALL say so plainly. A workflow that produces documents and no running code SHALL name that limit and name what comes next. A report that speaks in the contributor's terms while leaving them to infer that nothing runs yet has used the register to obscure rather than to inform.

#### Scenario: A described application reported as paperwork
- **WHEN** a contributor describes an application and the report's first claim names a directory, a document count, or a task order
- **THEN** the report is wrong, because none of those tells the contributor whether their description was understood

#### Scenario: A report surfacing what an artifact already states
- **WHEN** a proposal states a concrete sequence describing what working looks like
- **THEN** the report quotes or compresses that sequence rather than reporting that a proposal exists and argues a case

#### Scenario: A workflow that produces no artifact
- **WHEN** a workflow reaches a conclusion and deliberately writes nothing
- **THEN** its report leads with the question and the answer, because that is what the contributor asked for, and no obligation to name a produced artifact applies

#### Scenario: A report that reads warmly and hides the boundary
- **WHEN** a report opens in the contributor's terms but does not say that no running code exists yet
- **THEN** the report is wrong, because the register obligation is satisfied only by a report that informs

#### Scenario: An artifact fact a contributor needs
- **WHEN** a contributor needs to know where the change landed or what a check reported
- **THEN** the report states it, as annotation rather than as its opening claim

### Requirement: A report names what its references point at

A report SHALL gloss every pointer on its first use, naming in plain words what that pointer refers to, in the same sentence.

A pointer is a reference whose referent a reader cannot recover from the reference itself: a task identifier, a rule or requirement number, a diagnostic code, a count of warnings with no kind attached. A contributor reading a report SHALL NOT have to open an artifact to learn what the report is talking about, because a report that requires its own sources to be read has not reported.

A self-describing reference is not a pointer and SHALL NOT be glossed. A command, a flag, or a file path names its own referent, and glossing it adds words that carry no fact.

The distinction SHALL be settled by removing the pointers from the claim. Where the claim still stands without them, the pointers annotate a stated fact and no gloss is owed. Where the claim collapses, the pointers are the claim, and each one SHALL name its referent. Without this test the obligation fires on every identifier a report mentions, which produces a report padded with glosses nobody needed.

A pointer's own text SHALL be reproduced exactly. The gloss SHALL accompany it rather than replace it, so that a contributor who wants to find the referent still can.

A sequence of pointers presented as a list SHALL name each item, not only the first. A list whose first entry is named and whose remainder is numerals presents itself as readable while requiring every entry after the first to be looked up.

#### Scenario: A task order that collapses without its pointers
- **WHEN** a report presents an ordered list of task identifiers and removing them leaves no claim standing
- **THEN** each identifier names in plain words what the task does, because the identifiers are the claim rather than an annotation on it

#### Scenario: A self-describing reference
- **WHEN** a report names a command, a flag, or a file path
- **THEN** no gloss is owed, because the reference already names its referent

#### Scenario: A count with no kind
- **WHEN** a report states a number of warnings or errors without naming what they were
- **THEN** the count is a pointer and owes a gloss, because a contributor cannot act on a number alone

#### Scenario: A partly named list
- **WHEN** a report names the first item of a sequence and leaves the remainder as numerals
- **THEN** the report is wrong, because every entry after the first must be looked up

### Requirement: An agent settles what it can and reports the decision

A workflow SHALL settle a question it is competent to settle, and SHALL report the decision it took rather than presenting the question to the contributor.

Where a value is arbitrary and cheap to change, the agent SHALL choose one, state that the choice is its own, and continue. A report that asks a contributor to adjudicate a threshold, a label, or an ordering that follows from what they already described spends their attention on a decision the agent could have made and they can still reverse.

Where a workflow added scope the contributor did not ask for, the report SHALL name that scope and offer its removal, rather than present it as a question about what the contributor wants. The agent introduced it, so the agent SHALL own it.

What remains for the contributor SHALL be what genuinely requires them: a question whose answer changes the shape of the work, or a boundary only they can draw. A report that mixes these with settleable details makes both harder to see.

#### Scenario: An arbitrary value presented as a question
- **WHEN** a workflow needs a threshold or a label that the contributor's description does not determine
- **THEN** the agent chooses one, states that the value is its own and is cheap to change, and does not ask

#### Scenario: Scope the contributor did not request
- **WHEN** a workflow adds a behavior the description did not ask for
- **THEN** the report names it as the agent's addition and offers its removal

#### Scenario: A question that genuinely needs the contributor
- **WHEN** an answer would change the shape of the work
- **THEN** the report states it as an open question, distinguished from anything the agent settled itself
