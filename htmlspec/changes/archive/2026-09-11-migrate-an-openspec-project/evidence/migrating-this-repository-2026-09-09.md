# Migrating this repository, and what the attempt found

**Date:** 2026-09-09
**Tasks:** 8.1–8.5, partially 8.2
**Method:** a scratch copy of this repository's `openspec/` at `/tmp/migration/source`, surveyed,
planned, and staged with the machinery sections 3 through 6 built. The plan is recorded as
`evidence/the-plan-for-this-repository-2026-09-09.html`.

**This is an incomplete migration, deliberately reported as one.** Stages 1 through 4 ran in
full, the mechanical half of stage 5 ran in full, and the hardest collapse ran in full. 60 prose
documents were not written. What each stage found is below, and section 8's remaining tasks are
marked accordingly rather than claimed.

## What ran, and what it produced

| Stage | State | Result |
| --- | --- | --- |
| 1. Survey | complete | 201 entities, 8 kinds, zero unrecognized |
| 2. Read | complete | all 16 proposals, 9 capability purposes, every delta's shape |
| 3. Gather evidence | complete | 87 evidence files, 16 versions, the release record |
| 4. Plan | complete | 201 task units, 236 derivations, coverage check passes |
| 5. Stage — carried | complete | 103 entities byte-identical |
| 5. Stage — collapsed | complete | 16 specifications, 120 requirements, 545 scenarios |
| 5. Stage — converted | **not done** | 60 prose documents |
| 6. Check | partial | the 16 specifications validate clean and are canonical |
| 7. Promote | not run | stage 5 is incomplete |

201 entities rather than the 198 recorded three days ago, and rather than the 193 in task 1.1.
The difference is the evidence this change keeps writing. **That is the fourth stale count in
this change**, and it is why no test here asserts a literal.

## The plan covers everything, and the derivation table holds

```
$ htmlspec migrate promote plan.html --survey survey.json
201 surveyed entities, 201 plan items
every surveyed entity has a disposition
```

236 identifier derivations, and the numbers are what make the destination keying load-bearing:

| Fact | Count |
| --- | --- |
| derivations recorded | 236 |
| destinations holding derivations | 26 |
| identifiers derived more than once across the project | **99** |
| real collisions — two *different* headings, one identifier, one document | **0** |

`req-bootstrapping-installs-agent-instructions-for-the-tools-a-project-uses` appears in six
documents: five archived specifications and the accepted one. An identifier-keyed derivation
table reports that as five collisions and sends a contributor to reconcile a conflict the source
never contained. This is the case a rename test cannot catch, and the mutation recorded in
`the-plan-and-the-gate-2026-09-09.md` proved that.

The renamed obligation holds two distinct identifiers in the archives —
`req-a-single-command-exposes-validate-format-and-init` and
`req-a-single-command-exposes-validate-format-bootstrap-and-create` — plus the accepted heading's,
which is the four the design predicted.

## The collapse, against the version written by hand

`2026-08-10-html-spec`: five deltas, and every one of task 1.5's six findings held.

| Finding | Hand-written | Produced |
| --- | --- | --- |
| requirements | 47 | **47** |
| scenarios | 138 | **138** |
| `## Purpose` paragraphs, each introducing its capability's group | 5 | **5** |
| the two "Formatting … without writing" obligations derive distinct identifiers | required | `req-formatting-can-be-checked-without-writing`, `req-formatting-can-check-without-writing` |
| verification units | 0, with the reason recorded | **0** |
| masthead summary written rather than carried | required | written, from the proposal and design |

Every source heading has a unit, every identifier is unique in the document, and the document
validates with 0 errors and is canonical.

Across all 16 changes: **120 requirements and 545 scenarios**, which are exactly the figures task
6.3 was written against. 6 `## Purpose` paragraphs carried (5 in the founding change, 1 in
`distinguish-a-requirement-from-an-example`), and **2 renames stated in prose** naming both
headings, with `data-change="modified"` on the unit — recorded as an open question rather than
mapped silently.

## The systematic fault, in the wild

The first collapse produced **281 errors in one document**. Not 281 problems:

```
281 errors in 2 classes:
   276  schema/invalid
     5  refs/broken-link
```

276 were one wrong mapping rule. I had projected scenario steps as `<p><strong>WHEN</strong>…`,
and the projection specification says a scenario's `given`/`when`/`then` come from a description
list — each `dt` a label, each following `dd` an entry. **One fix, and 276 diagnostics went to
zero.**

The 5 were the template's contents list pointing at sections the collapse replaced with
capability groups. Also one fix.

This is the check-last requirement's own argument arriving unprompted. Had the run halted at the
first error I would have repaired one scenario by hand and continued into 137 more. Seeing 276
identical diagnostics made the shape obvious in seconds. **The requirement is right, and this is
the first time it was tested by something other than a test.**

## Evidence crosses byte-identical: 103 of 103

87 evidence files, 16 recorded versions, and the 4 already-converted documents. Including:

- **4 nested below `evidence/`** — `evidence/transcripts/` in
  `distinguish-a-requirement-from-an-example`, which a flat glob misses.
- **3 that are not markdown** — `mutation-testing.txt`, `prove-listing.py`, `prove-sequence.py`.
- **4 already in this format** — carried as evidence rather than as documents.

Byte-identity is asserted rather than assumed, comparing the source bytes against the staged
bytes for every carried entity.

## A finding the migration produced: carried HTML evidence is reachable by `validate`

**The one genuine defect this exercise found, and it has no clean resolution inside this
change's scope.**

The 4 documents from the earlier hand conversion are carried as evidence, byte-identical, which
is right: they declare contract version 0.1.0 and reference an asset store `86c33f4` deleted, so
carrying them as documents would import dangling references into a project whose asset check
exists to report exactly that. The plan says so and the requirement "A source entity already in
the destination format is not assumed usable" predicts it.

But `discover::documents` collects every `.html` file beneath a project, and evidence is not
excluded. So:

```
$ htmlspec validate htmlspec/changes
20 document(s): 2 error(s), 138 warning(s)

$ htmlspec validate .../evidence/deterministic-formatting.html
asset/unresolved: <link href="../../../../../assets/sha256-3e4014c3004abea5/htmlspec.css">
  names bytes this project does not store
```

Worse, `htmlspec format htmlspec/changes` **rewrote one of them** — it is an `.html` file, so the
formatter treated it as a document and repointed its asset references at the staged store. That
destroyed the byte-identity evidence exists for, and it was caught only because the identity
check ran afterwards. Restored from the source.

Three options, worked:

| Option | Result |
| --- | --- |
| suffix them `.html.txt` | The tree validates clean. But it renames a file whose whole value is that nobody touched it, and a reader cannot open it in a browser. |
| exclude `evidence/` from discovery | Correct in shape, and **out of scope**: it changes what `validate`, `format`, and `assets` read for every project, not only migrated ones. That is a contract question about what a project's documents are, decided in a change whose subject is something else — the exact fault the layout requirement exists to prevent. |
| **carry them under their own names and record the conflict** | The migrated project's `validate` reports 2 errors it cannot clear. Honest, and it names the change that settles it. |

**Taken: the third**, under `contract-authority`'s "Silence is recorded and not filled" third
route. The contract states nothing about whether a change's evidence is subject to document
discovery, so this migration does not settle it. **The change that settles it is one whose
subject is what a project's documents are** — and until then a migrated project holding HTML
evidence carries two asset diagnostics that name real dangling references in files nobody should
be resolving.

This also means **task 6.1's obligation cannot be fully satisfied for this repository's own
migration**: the complete staged tree cannot pass `validate` while those four files sit in it
under their own names. The machinery is right; the source has a shape the destination has no
policy for.

## What was not done, and why it is not marked done

**60 prose documents.** 51 change documents and 9 accepted specifications, re-expressing about
133,000 words of source prose. Each needs an agent to decide which paragraph states an
obligation, which comparison becomes a table and which becomes a diagram, and what each
argument was for — the work `project-migration` requires to be agent-driven and that no script
here performs.

The mechanical carries and the collapse are in this staged tree because both have rules. The 60
do not, which is the whole premise of the workflow. Writing them mechanically would produce
exactly the failure this change exists to prevent: 60 documents that pass every check and have
lost the reasoning. So they are absent and reported as absent, and tasks 8.6, 8.7, and 8.8 stay
open because each needs migrated prose documents to read.

Section 8's remaining tasks are marked in `tasks.md` with what was and was not verified.
