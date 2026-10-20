# Evidence read as documents, measured against HEAD and against the change

**Date:** 2026-09-29
**Tasks:** 1.1, 1.2, and 5.1
**Binary:** `htmlspec 0.17.0 (dfb2b0448023)` for the "before" half, and the built change for the
"after" half.

## The seven files

The subject is this repository's own HTML evidence, which is what the migration will carry:

| File | Where it is now | Declares |
| --- | --- | --- |
| `a-migrated-design-2026-09-09.html` | `2026-09-11-migrate-an-openspec-project/evidence/` | 0.1.0 |
| `the-collapse-produced-2026-09-09.html` | the same | 0.1.0 |
| `the-plan-for-this-repository-2026-09-09.html` | the same | 0.1.0 |
| `deterministic-formatting.html` | `2026-08-10-html-spec/documents/` | 0.1.0 |
| `proposal.html`, `design.html`, `tasks.html` | `2026-08-14-separate-artifact-kinds/documents/` | 0.1.0 |

The migration's 2026-09-09 plan gives the last four the destination `evidence/`, because they name
a store that no longer exists. So in a migrated project, all seven sit beneath some change's
`evidence/`.

## Before: task 1.1, `validate` and `format` in a project

A scratch project from `htmlspec init --tools none`, one change from `htmlspec new change`, and the
seven files copied into its `evidence/`. The four hand-converted documents went into
`evidence/converted/`, to exercise nesting.

| Command | Exit | What it said about evidence |
| --- | --- | --- |
| `validate` | 1 | `10 document(s): 14 error(s), 70 warning(s)` |
| `format --check` | 1 | every evidence file: two stale references and "not canonically formatted" |
| `format` | 0 | rewrote all seven |

The errors, per file:

| Document | Errors | Warnings | Rule |
| --- | --- | --- | --- |
| the change's `design.html` | 0 | 1 | |
| the change's `proposal.html` | 0 | 1 | |
| the change's `tasks.html` | 0 | 3 | |
| each of the seven evidence files | **2** | 0 to 47 | `asset/unresolved` |

**Every error is an evidence file's.** 14 errors across 7 files, two each: the stylesheet and the
print stylesheet. Six files name `e4f5de7…` and `0350261…`, and `deterministic-formatting.html` names
`3e4014c…` and `da53340…`. This project never held any of those addresses. The change's own documents raise no error.

Then `format`, compared against a pristine copy with `cmp`:

```
CHANGED evidence/a-migrated-design-2026-09-09.html
CHANGED evidence/converted/design.html
CHANGED evidence/converted/deterministic-formatting.html
CHANGED evidence/converted/proposal.html
CHANGED evidence/converted/tasks.html
CHANGED evidence/the-collapse-produced-2026-09-09.html
CHANGED evidence/the-plan-for-this-repository-2026-09-09.html
```

**Seven of seven rewritten.** The 2026-09-09 record found one, because it ran `format` over one
directory. Across a whole project, `format` repoints every carried record's stylesheet at the
current store. A record's worth is that nobody touched it, and `format` touches all of them.

## Before: task 1.2, promotion

A staged tree built with `init`, holding one archived change (`VERSION` plus the three
`migrate-an-openspec-project` HTML evidence files), with a plan that carries all four entities.
The script is `scratchpad/evidence-measure/promote.sh`, and its setup mirrors
`crates/htmlspec-cli/tests/promote.rs`'s `Migration::stage`.

```
checked 3 document(s) and 8 stored asset(s) across the complete staged project
4 surveyed entities, 4 plan items
every surveyed entity has a disposition

13 conformance diagnostic(s) across 3 staged document(s):
  …/evidence/a-migrated-design-2026-09-09.html: asset/unresolved <link href="../../../assets/sha256-e4f5de7cecc4f691/htmlspec.css"> …
  … (12 more, all naming one of the three evidence files)
htmlspec: promotion does not proceed. … The destination is unchanged
promote exit 1
```

**The coverage check passes and the conformance check refuses, and it refuses on evidence
alone.** The "3 document(s)" it checked are the three evidence files. The staged project has no
other document. Each file draws two `asset/unresolved` errors and two "names bytes this project
does not store" findings. The plan file also draws "not canonically formatted". It is the one
file whose stylesheet references are bare `assets/…` paths, which is the only difference I found
between it and the other two. I did not establish that this is the cause. That makes 13.

This is the refusal the proposal predicts, by the route it names. Nothing else in the tree is
wrong.

## After: task 5.1, the same two measurements against the built change

The same scratch project, with the seven files restored from the pristine copy first:

| Command | Before | After |
| --- | --- | --- |
| `validate` | exit 1, `10 document(s): 14 error(s), 70 warning(s)` | exit 0, `3 document(s): 0 error(s), 5 warning(s)` |
| `format --check` | exit 1, every evidence file reported | exit 0, the change's three documents canonical |
| `format` | exit 0, **seven of seven rewritten** | exit 0, **seven of seven byte-identical** |
| `validate <the change>/evidence` | exit 1 | exit 1: the records are checked when named |
| `validate <one record>` | exit 1 | exit 1 |

"3 document(s)" is the change's own proposal, design, and tasks, and nothing else. The last two
rows are the half that must not move. A contributor who asks about a record still gets the
ordinary answer, including its `asset/unresolved` errors.

And promotion, with the same script:

```
4 surveyed entities, 4 plan items
every surveyed entity has a disposition
every check passed and nothing was moved. Run this without --check to promote

promote exit 0
same a-migrated-design-2026-09-09.html (at repository)
same the-collapse-produced-2026-09-09.html (at repository)
same the-plan-for-this-repository-2026-09-09.html (at repository)
```

The check passes, promotion moves the tree, and the three records arrive byte-identical. This
repository's own migration no longer has an obstacle that belongs to the product.
