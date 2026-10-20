## Why

The project set out to port OpenSpec to HTML. What it built is a single document shape carrying all seven semantic units at once. That shape loses the feature that makes OpenSpec worth porting: **the separation of a change into stages.**

The stages are not filing conveniences. Each has a different job and a different degree of formality:

- A **proposal** argues value and need. It is the most human artifact and needs substantial freedom in how it is written. It is not where the contract is stated.
- A **design** turns that argument into architecture and technical considerations. It reasons in prose and is deliberately not obliged to reduce those considerations to work items.
- **Tasks** enumerate the work to be done.
- A **specification** is the settled contract that survives the change.

The current schema represents none of this. It defines one document shape with a required `status` field enumerating `exploring → proposed → accepted → implementing → verified → archived`: six stages of one document where the project needs four kinds of document. The separation was not rejected; it was flattened into optionality. Every unit is optional, so a proposal and an as-built specification are distinguishable only by the sections they happen to contain. Nothing tells an agent which content is appropriate to the artifact it is writing.

Two facts make the problem concrete. First, `status` does no work: no validation rule reads it, and no behavior differs between `exploring` and `archived`. Second, the project does not yet author its own changes in its own format. This change and its predecessor were written as OpenSpec Markdown, while the existing html-spec project documents were generated or converted rather than authored through the html-spec workflow.

There is a second observation worth acting on in the same change. Four of the seven units acquire their authoritative form retrospectively: evidence and verification can be completed only after implementation, while requirements and scenarios reach final form once the implemented behavior is known. The seven-unit vocabulary is therefore not the shape of a proposal. It is the shape of a **specification** produced when a change is archived, describing what the change actually established.

## What Changes

- **A declared artifact kind.** Every document declares one of four kinds: `proposal`, `design`, `tasks`, or `specification`. The kind identifies the document's role and replaces `status`, whose stages belong to the change rather than to each file.

- **Authoring guidance and validity are separate.** Each kind has intended content, which guides authors and agents, and enforced structure, which validation checks. Departing from intended content does not itself make a document invalid.

  | Kind | Intended content | Enforced structure |
  | --- | --- | --- |
  | `proposal` | The value, need, and scope of the change | None |
  | `design` | Architecture and technical reasoning | None |
  | `tasks` | Implementation work items | At least one task unit |
  | `specification` | The settled contract established by the completed change | No task units; requirements without verification produce a warning |

- **The upstream kinds are unconstrained.** Validation reports nothing about which units a proposal or design carries or omits. Prose alone conforms, and a requirement unit inside a proposal also conforms: an author discussing a prospective requirement, in prose or as a unit, is doing the proposal's job. That requirement is not authoritative. The contract is what the specification states, not what the proposal considered.

- **Exactly three downstream rules apply.** They are three distinct kinds of rule, not a general placement matrix.

  | Rule | Applies to | Severity |
  | --- | --- | --- |
  | **Placement:** a specification must not carry task units | `specification` | Error |
  | **Completeness:** a tasks document must carry at least one task unit | `tasks` | Error |
  | **Advisory:** a specification requirement with no verification | `specification` | Warning |

  No other unit-and-kind combination is prohibited or reported.

- **The contract rules continue to apply to every kind.** Structural freedom is not permission to ship a document that will not open. Unambiguous markup, resolvable references, no remote or executable content, and asset references whose declared types match their bytes remain required for every document. Kind determines how much semantic structure is required, never whether the document works.

- **Every synthesis and contract update is agent-driven.** There are four distinct operations: creating a design using the proposal; creating tasks using the proposal and design; producing the change's specification from the completed change; and updating the project's accepted specifications from that specification. The first three synthesize new artifacts while leaving their source artifacts intact. Only the fourth is a merge, and it merges into the accepted contract rather than between stages. In every case, an agent reads and writes under stated rules and guides; no program parses one artifact to emit another.

- **A change is a directory of documents.** It contains four files with four declared kinds rather than one file carrying every stage.

- **Archival is specified end to end.** A change directory contains one specification artifact. At archival, the agent creates or updates that artifact from the completed change, including its proposal, design, tasks, and implemented result. `htmlspec validate` checks the specification. Once it validates, the agent applies its contract changes to the project's accepted specifications, and the change directory is archived intact. The change specification and the as-built specification are the same file. Only two specification documents are involved: the change specification and the project's accepted contract.

Non-goals, deferred deliberately: the `/htmlspec:*` command lifecycle beyond archival, the Markdown-to-HTML converter, and any change to the definitions of the seven semantic units. This change decides the intended role of each artifact kind and enforces only the three downstream rules stated above.

**BREAKING:** `document-template` currently requires a lifecycle status on every document. This change replaces that field with `kind`, so every document that conforms today is invalid until migrated. The project has three project documents and forty-one fixtures to migrate, with no external consumers.

## Capabilities

### New Capabilities

- `artifact-kinds`: Defines the four artifact kinds, the intended content of each, and the separation between authoring guidance and the three enforced rules. It also defines a change as a directory of kinded documents, specifies the archival sequence, and requires synthesis and contract updates to be agent-driven.

### Modified Capabilities

- `document-template`: Replaces the per-document lifecycle status with a declared artifact kind. The identity requirement lives here rather than in `document-schema`, which continues to own the shapes of the semantic units and is otherwise unchanged.
- `document-validation`: Adds the placement error for task units in a specification, the completeness error for a tasks document with no task units, and the advisory warning for a requirement without verification. It states that no other unit-and-kind combination is checked.
- `toolchain-packaging`: `init` accepts a kind and can create a complete change directory. Archival gains an agent-driven operation that produces the change specification and updates the accepted contract, with the program validating the specification the agent writes.

## Impact

- **Every existing document must be migrated.** The three project documents and forty-one conformance fixtures currently declare `status` and must declare `kind` instead. The fixtures are generated from stated edits to a base document, so most of the migration is one change to the generator. `valid-minimal` and its siblings become `specification` documents because that is the shape they already have.

- **Removing `status` eliminates no existing behavior but remains schema-breaking.** No validation rule reads the field, and nothing behaves differently at `exploring` than at `archived`, so no workflow depends on it. Because it is currently required, however, every conforming document becomes invalid until it declares a kind.

- **Synthesis is a skill, not code.** OpenSpec's sync guidance states the governing principle: "This is an **agent-driven** operation… Unlike programmatic merging, you can apply partial updates. The delta represents *intent*, not a wholesale replacement." The deterministic part is `htmlspec validate`, which checks the agent's output against html-spec's structural and contract rules. This change adds no parser that reads one artifact and emits another.

- **The port gets smaller.** Mapping the OpenSpec proposal, design, and tasks files to kinded HTML documents is nearly mechanical because none requires semantic units to be invented from prose. The difficult work—finalizing which implemented behaviors are contractual—moves to archival, when the completed result is available.

- **Dogfooding becomes possible.** With artifact kinds, the project can author its own changes directly in its own format, exercising a workflow that forty-one generated fixtures cannot. The unconstrained proposal kind makes that practical: writing a proposal in HTML should be no harder than writing one in Markdown.

- **The rule set is smaller than in the first draft.** An earlier version made every misplaced unit a non-downgradable error, putting constraints where the work needs freedom. That approach was removed. This revision also removes language that implied a broader placement matrix: only the three stated downstream rules are enforced.
