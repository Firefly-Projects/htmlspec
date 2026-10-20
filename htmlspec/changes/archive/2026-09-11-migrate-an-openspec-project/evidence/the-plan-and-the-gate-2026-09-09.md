# The plan's shape, and the gate that checks it

**Date:** 2026-09-09
**Tasks:** 4.1–4.8
**Method:** `crates/htmlspec-cli/src/plan.rs` (the reader and the comparison),
`commands.rs::migrate_promote` (the command), `crates/htmlspec-cli/tests/plan.rs` (the shape,
through the compiled program). Two deliberate mutations, one of which found a real gap in a test
I had already written.

## The plan's shape, and why the facts are notes

A plan needs three facts per entity the `tasks` kind does not carry: the source path, the
destination path, and the disposition. The projected model of a task holds `id`, `change`,
`title`, `state`, `reason`, `references`, and `notes`, and nothing else.

Three shapes were available and two are wrong:

| Shape | Why not |
| --- | --- |
| `data-source="..."` on the unit | The projection **ignores** an unknown `data-*` attribute (projection spec 2.2). A gate reading it would parse the HTML a second way and could disagree with the model about what the document says. |
| `<p data-field="source">` | `data-field` naming no field of the enclosing unit is **invalid** (projection spec 4.0). The plan would not validate. |
| `<p data-field="note">source: ...</p>` | Projects into `notes`, which the model carries. One reader, one answer. |

So each fact is a note paragraph opening with its name and a colon. `reason:` is read from the
note **or** from the task's own `data-field="reason"`, because a task whose `data-state` is
`dropped` already carries one and `schema/missing-reason` already requires it — refusing one
spelling would make the plan's shape depend on which of two equivalent markups an agent chose.

A note whose opening word is none of the five keys is ordinary prose. The convention is additive
on purpose: a contributor annotating a plan line writes a paragraph and nothing breaks.

## The shape is proven against the project's own checks, not asserted

`tests/plan.rs` builds the plan the way the workflow will: `htmlspec init`, then
`htmlspec new document tasks`, then the template's two example units replaced by the plan's own
(located by identifier rather than by position, so a template that gains a unit fails loudly).
One document then goes through four operations in turn:

```
htmlspec validate     → accepted, exit 0
htmlspec format       → formatted
htmlspec format --check → canonical, exit 0
htmlspec migrate promote → every surveyed entity has a disposition, exit 0
```

**The `format --check` after the `format` is the load-bearing step.** The formatter reorders
attributes and re-indents, so a shape leaning on attribute order or indentation would either be
rewritten by it or refused by it. Running the check after the format proves the shape is a fixed
point rather than something that parsed once. The test then re-reads the formatted file and
asserts the three note strings survived, so a formatter that normalized note text away would
fail here rather than silently emptying the plan.

One test rather than four, because the claim is that a *single* document satisfies all of them at
once. Four tests each building their own plan could pass while no one shape satisfied everything.

## The gate reports eight classes, every occurrence, together

`migrate promote <plan> --survey <payload.json>`. The comparison is its first act, before
anything is read from a staging tree.

| Class | What it catches |
| --- | --- |
| `unaccounted` | a surveyed entity no plan item names |
| `sourceless` | a plan item naming no source at all |
| `withoutDisposition` | a source with no decision recorded |
| `unknownDisposition` | a word outside the four, **with the word written** |
| `withoutReason` | folded or dropped with no reason |
| `duplicated` | one entity named by two items, so which disposition applies is undecided |
| `unsurveyed` | a plan written against a different survey |
| `collisions` | two headings deriving one identifier in one destination document |

The payload is read from a file rather than re-surveyed, because `project-migration` requires the
plan to be checked against "the list the plan was written from". Re-enumerating would compare it
against a second opinion taken later.

Exit codes follow `exit.rs`: **1** for an incomplete plan (a document failed a check and the
operation ran), **2** for a plan or payload that could not be read (nothing was determined about
the source). A plan that cannot be read reports as an operational failure rather than reporting
every entity as unaccounted, which would send a contributor to the source when the fault is the
file they named.

## The gate does not judge a disposition, and says so

`a_disposition_a_reader_would_disagree_with_passes` sends an accepted capability specification
into a change's evidence directory — plainly the wrong destination, plainly not this check's
business — and asserts the coverage is complete.

The passing report says it in words rather than leaving it to be inferred:

```
2 surveyed entities, 2 plan items
every surveyed entity has a disposition
the coverage check passed. Whether each disposition is right is what this plan is reviewed
for, and the check says nothing about it
```

A green coverage check otherwise reads as approval of the plan, which is the one thing it is not.

## Mutation 1: report only the first unaccounted entity

The fault task 4.3 forbids. Restricting the loop to `coverage.unaccounted.is_empty()`:

- `plan::tests::every_unaccounted_entity_is_named` **FAILED**
- `commands::tests::promotion_refuses_and_names_every_unaccounted_entity` **FAILED**

Both layers caught it, and the suite returned to green when restored.

## Mutation 2: the one that found a gap in my own test

Keying the derivation table on the identifier alone, discarding the destination document —
exactly the heading-keyed table `project-migration` forbids:

```
test result: ok. 205 passed; 0 failed
```

**The mutation survived every test, including the one written for it.** The reason is that
`one_obligation_renamed_across_changes_is_not_a_collision` uses a rename, and a rename produces
*distinct* identifiers. A wrong key still finds no duplicate among three different strings, so
the test passes either way. It pins that a rename is not reported as a collision; it does not pin
the keying that makes that true.

The case that needs the destination is the **ordinary** one, and this repository holds it 17
times:

```
17 headings stated by more than one archived change
  'Bootstrapping installs agent instructions for the tools a project uses'  in 5 changes
  'The command surface reports what a project holds'                        in 3 changes
  'Archival assesses the merge and never performs it'                       in 2 changes
  'Bootstrapping ends by naming the command to run next'                    in 2 changes
```

A change that modifies an obligation restates its heading, so one slug legitimately lands in
several archived specifications. An identifier-keyed table reports five collisions for the first
row and sends a contributor to reconcile a conflict the source never contained —
`artifact-kinds` "A specification states the settled contract as built" requires each archived
specification to state what *its* change established, so five documents stating one heading is
what a correct migration produces.

`one_heading_stated_by_several_changes_is_not_a_collision` uses those five real changes and
**FAILS** under the mutation. Restored, the suite is green.

The general lesson is the one this change keeps producing: a test written for a requirement is
not the same as a test that can fail when the requirement is violated, and the only way to tell
them apart is to break the code.

## State of the suite

- `cargo test`: **all green**, zero failures. 24 unit tests in `plan.rs`, 5 command tests in
  `commands.rs`, 2 integration tests in `tests/plan.rs`.
- `cargo clippy --all-targets`: **no warnings**.
- `cargo fmt --all`: applied.
