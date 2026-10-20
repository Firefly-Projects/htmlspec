# No accepted requirement covers this, and one deliberately excludes it

Task 1.3, recorded 2026-08-31. The point is to show the change fills a hole rather than
reopening a settled decision.

## The requirement that excludes it from archival

`toolchain-packaging`, `Archival assesses the merge and never performs it`:

> Archival SHALL determine whether the change's contract reached the accepted specifications,
> and SHALL do so by comparing requirement identifiers rather than by reading requirements for
> meaning.
>
> Archival SHALL NOT edit an accepted specification, SHALL NOT propose the text of a merge, and
> SHALL NOT treat an absent identifier as an error. Whether an absent requirement was withdrawn
> deliberately or omitted by mistake is a judgment, and `artifact-kinds` reserves it for an
> agent.

This is correct and this change does not touch it. What it does is assign the excluded
operation somewhere: the requirement says archival must not perform the merge, and no
requirement anywhere says who does.

## The requirement that fixes the spellings and omits this one

`toolchain-packaging`, `The command surface names its operations as OpenSpec names them`:

> The spellings are therefore fixed rather than left to an implementer: `validate`, `archive`,
> `init`, `new`, `list`, `status`, `instructions`, and `store` name the operations OpenSpec
> also provides.

Eight names. `sync` is absent, and OpenSpec provides it.

## Every capability, checked

```console
$ ls openspec/specs/
artifact-kinds  deterministic-formatting  document-schema  document-template
document-validation  project-bootstrap  toolchain-packaging  workflow-reporting
```

- `artifact-kinds` governs the documents and reserves the merge for an agent — it says the merge
  must be agent-driven, not who offers it.
- `project-bootstrap` governs establishing a project.
- `document-validation`, `document-schema`, `document-template`, `deterministic-formatting`
  govern documents themselves.
- `workflow-reporting` governs how a report addresses a contributor.
- `toolchain-packaging` governs the command surface, and is where the hole is.

No capability states an operation whose subject is the accepted specifications. The change adds
requirements to `toolchain-packaging` rather than creating a capability, because the command
surface is what gains an operation.
