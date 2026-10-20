# No accepted capability reaches a source project in another format

**Date:** 2026-09-07
**Task:** 1.7
**Method:** read all 97 requirement headings across the 9 accepted capabilities and searched
their text for anything reaching a project in a format other than HTMLSpec.
`grep -rn -i 'migrat\|openspec project\|source project' openspec/specs/*/spec.md` returns one
hit, and it is `document-schema:175` — "its visualization source project" — a sentence about a
figure's source, unrelated.

## What each capability governs, and where its reach stops

| Capability | Reqs | Governs | Where its reach stops |
| --- | --- | --- | --- |
| `project-bootstrap` | 10 | establishing a project: its root, declaration, one asset store, installed instructions, the archival version gate | Establishes an **empty** project. Nothing reaches content that already exists elsewhere. |
| `artifact-kinds` | 11 | documents that already exist: the four kinds, the change directory, the archival sequence, agent-driven synthesis | Assumes documents in the destination format. Nothing reaches a document in another. |
| `toolchain-packaging` | 22 | the command surface, exit codes, output forms, reproducible packaging, archival's behavior | Names every operation the surface has. `migrate` is not among them. |
| `document-schema` | 12 | the HTML element contract for semantic units | The destination vocabulary. Says nothing about producing it from another. |
| `document-validation` | 19 | structural and safety rules, diagnostics | Checks a document. Indifferent to where it came from. |
| `deterministic-formatting` | 7 | canonical serialization | Same. |
| `document-template` | 8 | the page a reviewer opens, offline runtime | Same. |
| `workflow-reporting` | 3 | what a workflow's report owes its contributor | **Reaches the migration's report**, but not the migration. |
| `contract-authority` | 5 | what states an obligation, what an agent owes when asserting one, what it does under silence | **Governs how the migration decides**, not what it does. |

The last two rows are the reason this is worth writing down rather than asserting. Two
capabilities do bear on migration, and neither of them governs it:

- **`workflow-reporting`** applies to the `migrate` workflow's report the moment that workflow
  exists, because its subject is "what a workflow owes the contributor who invoked it" and it is
  stated without a list of workflows. Task 7.7's `## What to report` section is written under it.
  But it says nothing about surveying, planning, staging, or promoting.
- **`contract-authority`** is what every decision in this change's mapping table was checked
  against, and its "Silence is recorded and not filled" is what rows 12–16 of that table follow.
  But it governs an agent's epistemics, not a migration's stages.

So a new capability is right: `project-migration` states the stages, the coverage gate, the
staging discipline, and the delta collapse, and none of those has a home in the nine.

## Nothing currently reports any of this

There is no `migrate` in the command surface. `toolchain-packaging` — "The command surface names
its operations as OpenSpec names them" (`spec.md:190`) — enumerates the operations, and
`crates/htmlspec-cli/src/main.rs` implements them: `validate`, `format`, `assets`, `archive`,
`init`, `list`, `status`, `sync`, `store`, `instructions`, `new`. Eleven, and no twelfth.

So today a contributor standing in an OpenSpec project gets no report about it at all — not a
refusal, not a diagnostic, nothing. The proposal's "There is no way out" is literally true: the
absence is total rather than partial.

## Two accepted requirements that bear directly on the version fault

Searching for migration coverage turned up the two requirements that independently confirm what
`evidence/the-mapping-table-and-its-audit-2026-09-07.md` found — and they are in
`toolchain-packaging`, not `project-bootstrap`, so the design's fault was checkable from a
second place as well.

**`toolchain-packaging` — "An archived change is named for its date and refuses to collide"**
(`spec.md:403`):

> Archival SHALL name the archived directory for the date of archival followed by the change's
> name.

Date, then name. **No version component.** So `<date>-<version>-<name>` is not merely
unsupported by the version gate — it contradicts a stated requirement about what an archived
directory is named. A migration composing that name produces 16 directories violating it.

**`toolchain-packaging` — "An archived change records the version it shipped in"**
(`spec.md:310`):

> Each archived change SHALL record the version the project declared when it was archived, in a
> form the project's own suite can read without version control history.

The version lives in a recorded form the suite reads, which is the `VERSION` file
(`crates/htmlspec-cli/src/project.rs:46`, and `scripts/check-release`'s `VERSION_FILE`). This is
the requirement the migrated archives satisfy by carrying `VERSION` across unchanged.

It also carries a **constraint the migration must respect**: recorded versions SHALL be distinct,
with a baseline exception for changes archived under one version before the obligation existed.
This repository has **seven changes recording `0.1.0`**. So a migrated project inherits a
seven-way version collision that `scripts/check-release` treats as a baseline, and the migration
must carry the changelog's baseline entry for it — otherwise the migrated project fails its own
release check on arrival. Neither the design nor the tasks mention this; it is recorded here and
belongs in the plan.
