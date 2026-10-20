# The instructions this change does not edit, and why

**Date:** 2026-10-02
**Task:** 4.3

`grep -n "absent\|untakenRenames\|sync --change\|withdrawn"` over the seven other workflow bodies in
`crates/htmlspec-assets/instructions/` finds no line in any of them. The search was widened to the
stems `untaken`, `sync`, `withdraw`, `removed`, and `reached`, case-insensitive, to catch a sentence
the task's pattern would miss:

| Body | `absent` | `untaken` | `sync` | `withdraw` | `removed` | `reached` | Verdict |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `new.md` | 0 | 0 | 0 | 0 | 0 | 1 | silent. The one hit, line 170, is "the decisions reached" in a design |
| `continue.md` | 0 | 0 | 0 | 0 | 0 | 0 | silent |
| `ff.md` | 0 | 0 | 0 | 0 | 0 | 0 | silent |
| `apply.md` | 0 | 0 | 0 | 0 | 0 | 1 | silent. Line 47, "the decisions reached" |
| `verify.md` | 0 | 0 | 0 | 0 | 0 | 1 | silent. Line 53, "the decisions reached" |
| `explore.md` | 0 | 0 | 0 | 0 | 0 | 0 | silent |
| `migrate.md` | 0 | 0 | 0 | 0 | 0 | 0 | silent. Its mentions of an accepted specification are about where migration stages one, and it names no set `sync` reports |

**No other body names a set the comparison reports or says what happens to a withdrawn
requirement**, so none needs an edit. The two that do, `sync.md` and `archive.md`, are the two this
change edits. `new.md`, `continue.md`, and `ff.md` leave the specification to archival, and `apply.md`
and `verify.md` leave the merge to it, as `state-a-modified-requirement-whole`'s
`the-other-instructions-2026-10-02.md` recorded.
