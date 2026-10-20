## 1. Measure what the project expresses today

- [x] 1.1 Record that a change's declaration carries `schema` and `created` and nothing relational, that the enumeration returns names and task counts, and that validation checks a change against its schema and never against a sibling — with the command output as evidence rather than as a claim
- [x] 1.2 Record that `refs/unresolved` resolves a reference against the identifiers of the same projection, so a reference naming another change is an error today. This is the measurement that decides whether the declaration can reuse the reference machinery, and it decides against it
- [x] 1.3 Record that `archived_name` prepends `YYYY-MM-DD-` to an archived directory, so a predecessor matched by string equality fails exactly when the predecessor is archived. Demonstrate the failure against a real archived change rather than reasoning about it
- [x] 1.4 Record the two precedents this change follows: `asset/duplicate`, whose subject is a project rather than a document and which is therefore absent from the per-document rule set, and `merge_state`, which already compares one change against the accepted specifications. A cross-change check is established practice here, not a new category
- [x] 1.5 Quote the motivating proposal from the project that stated the ordering in prose, and record which of its four changes would have been archivable out of order with nothing reported

## 2. The contract states what an ordering is

- [x] 2.1 Add `A change states the change it follows` to `artifact-kinds`, as a declaration the author writes rather than a relation the program infers
- [x] 2.2 State the three prohibited inferences — from names, from creation dates, from requirement identifiers — each with the reason it is refused. The name case cites the accepted sentence that nothing depends on a change's first name
- [x] 2.3 State that at most one predecessor is carried, and that a change stating none is complete as it stands and is not reported
- [x] 2.4 Add `A stated predecessor is resolved across the archive`, naming the three outcomes and requiring that they be reported distinctly
- [x] 2.5 State that an archived predecessor is matched with its leading date removed, recognised by the same test archival uses, because equality fails in the one case the resolution exists to detect
- [x] 2.6 State that a cycle is reported as a contradiction and that the program chooses no ordering within one
- [x] 2.7 Modify `Archival produces the specification and then updates the accepted contract` so an unresolved predecessor is a reported finding rather than a refusal, with the payload stating it as a value. Keep every accepted paragraph and scenario intact — the requirement is a superset, checked rather than assumed
- [x] 2.8 Modify `The command surface reports what a project holds` so the enumeration carries the predecessor and whether it resolved, and so the order still does not follow what the listing reports. Keep the completion clauses from 0.8.0 intact
- [x] 2.9 Run `openspec validate --all`

## 3. Reading the declaration

- [x] 3.1 Add the `follows` field to the change declaration, typed so an absent field and an empty one are the same state and neither is an error
- [x] 3.2 Give the resolution outcome a sum type with three cases — archived, active, unmatched — rather than an optional boolean, so no caller can represent "unmatched" as "unfinished"
- [x] 3.3 Implement the match against the archive with the leading date removed, reusing the existing date-prefix test rather than a second copy of it
- [x] 3.4 Add the test that fails for a naive equality match: a change whose predecessor is archived under a dated directory resolves as archived
- [x] 3.5 Add tests for the active and unmatched cases, asserting that the two are distinguishable from the resolution alone
- [x] 3.6 Implement cycle detection as a bounded walk over predecessors, and test the two-change cycle and the three-change cycle
- [x] 3.7 Confirm a declaration with no `follows` field parses, resolves to nothing, and reports nothing

## 4. What the operations report

- [x] 4.1 Carry the predecessor and its resolution into the enumeration's structured payload, leaving the accepted-specification arm untouched
- [x] 4.2 Render the predecessor in the human report without disturbing the alignment the completion columns established, and test that the columns still line up when some changes state a predecessor and others do not
- [x] 4.3 Add the unresolved predecessor to archival's findings, so it flows through the existing report, question, and `--yes` path with no new mechanism
- [x] 4.4 Extend archival's question to name the predecessor, as a pure function of the findings so the wording is testable with no terminal
- [x] 4.5 Carry the condition into archival's structured payload as a value, beside the existing findings
- [x] 4.6 Confirm the finding is absent when the predecessor is archived, and absent when no predecessor is stated. A report that fires either way is one a reader learns to skip
- [x] 4.7 Confirm archival still proceeds on an affirmative answer and on `--yes`, since an abandoned sequence must remain archivable

## 5. Prove it against a real sequence

- [x] 5.1 Build a project holding a three-change sequence, archive the middle change first, and confirm archival names the predecessor and asks rather than refusing
- [x] 5.2 Archive the same sequence in order and confirm nothing about ordering is reported at any step
- [x] 5.3 Rename a predecessor and confirm its successor reports an unmatched name rather than an unfinished predecessor
- [x] 5.4 Construct a two-change cycle and confirm it is reported as a contradiction with no ordering chosen
- [x] 5.5 Reproduce the motivating case: the four-change sequence from the missile-launcher project, with the contract first, and confirm the out-of-order archival that is silent today is reported
- [x] 5.6 Update the workflow instructions that select or archive a change to describe the predecessor the listing now carries
- [x] 5.7 Add the changelog entry and advance the version
