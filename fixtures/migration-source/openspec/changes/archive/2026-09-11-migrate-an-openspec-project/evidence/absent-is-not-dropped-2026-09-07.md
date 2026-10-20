# `config.yaml` holds one live key, so its context is absent rather than dropped

**Date:** 2026-09-07
**Task:** 1.6
**Method:** read `openspec/config.yaml` and filtered out comment and blank lines.

## What the file holds

One live key:

```yaml
schema: spec-driven
```

That is the whole of it. Filtering comments and blanks leaves exactly one line. The remaining
35 lines are three commented-out example blocks the file shipped with:

| Commented block | Fields it illustrates |
| --- | --- |
| `# Project context (optional)` | `context:` — tech stack, conventions, domain knowledge |
| `# Per-artifact rules (optional)` | `rules:` — per-artifact authoring constraints |
| `# Per-operation guidance (optional)` | `operations:` — advisory guidance for apply and archive |

`context`, `rules`, and `operations` are all **absent**. None is set, none is empty-valued —
they were never written.

The design (`design.md:250–262`) names `context` and `rules`. There is a third,
`operations`, which the design does not mention. It is absent for the same reason and takes the
same disposition, but the survey must enumerate the field set from the file rather than from the
design's list of two.

## Why this is the case worth recording

`project-bootstrap` — "A project declares its name, version, and expected toolchain"
(`openspec/specs/project-bootstrap/spec.md:34`):

> The declaration SHALL NOT carry authoring rules or project context. Those have no consumer,
> and a field nothing reads is the case the preceding paragraph excludes.

So had this repository set `context:`, there would be nowhere in the destination to put it, and
the plan would owe a dropped-with-a-reason line naming that requirement. It did not set it. The
plan therefore owes a *different* line: the field was empty in the source.

Those are two different facts about the project a contributor is receiving, and the plan
requirement demands they be distinguishable:

> An entity that is dropped because the destination format has nowhere to put it SHALL say so,
> and SHALL be distinguishable from an entity that was empty in the source.

A contributor reading "`config.yaml` context: dropped, the declaration cannot carry project
context" would reasonably go looking for what they lost. There is nothing to look for. A
contributor reading "`config.yaml` context: empty in the source" knows immediately that no
content went missing.

**`schema: spec-driven` is itself a dropped entity**, and this one is dropped rather than
absent. It names the source format's workflow schema. The destination has no schema selector —
there is one artifact vocabulary — so the value has no destination. The plan owes it a
dropped-with-a-reason line, and the reason is that the destination format has one workflow set
rather than a choice among several.

## This repository supplies both sides of the distinction, but not from `config.yaml`

The absent/dropped pair cannot be proven from `config.yaml` alone: it gives one absent case
(`context`, `rules`, `operations`) and one present-and-dropped case (`schema`), and the second
is a single scalar rather than the block of content the requirement is written about.

The **present-and-dropped** side with real content is `.openspec.yaml`, recorded in
`evidence/the-mapping-table-and-its-audit-2026-09-07.md` row 15: 8 files, each carrying a live
`created:` date, all 8 dropped because the destination records a change's date in its archive
directory name — and all 8 disagreeing with that name. Content that existed, was read, and is
gone.

So the pair a test can be built on is:

| Fact | Source | What the plan must say |
| --- | --- | --- |
| absent | `config.yaml` `context`, `rules`, `operations` | empty in the source; nothing was lost |
| dropped | `.openspec.yaml` `created:` ×8 | present, and dropped because the destination has no place for it |

Task 3.7's version-absence case is a third instance of the same distinction — a change with no
`VERSION` file has an absent version, not a dropped one — and it also cannot be exercised
against this repository, since all 16 archived changes record a version.
