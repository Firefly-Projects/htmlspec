# The instructions this change does not edit, and why

**Date:** 2026-10-02
**Task:** 3.6

Each workflow body under `crates/htmlspec-assets/instructions/` was read for what it says about a
change's specification or the merge into the accepted specifications. Line numbers are at this
change's working tree, after `sync.md` and `archive.md` were edited. All nine bodies carry the
shared "Name the requirement, or name the silence" section, which says "A change's own
specification becomes binding when archival applies it, and not before". That sentence is about
when a specification binds, and it agrees with the new rule.

| Body | What it says about the specification or the merge | Verdict |
| --- | --- | --- |
| `new.md` | "Do not write the specification" (lines 206-214): "A specification is produced **at archival**, from the completed change and from what was actually built." | Silent on how to state a modified requirement. No edit |
| `continue.md` | "Where this stops" (98-110): "the specification is produced at archival, from what was actually built — so writing one now would state intent rather than record a result." | Silent. No edit |
| `ff.md` | "What this does not do" (131-136): "It does not implement the change, and it does not write the specification." | Silent. No edit |
| `apply.md` | lines 123-125: "This workflow does not produce the specification, advance the project's version, or update the accepted specifications. Those belong to archival" | Silent. No edit |
| `verify.md` | lines 110-116: it does not "produce the specification, … apply contract changes to the accepted specifications, or move the change to the archive. Those four are archival's work" | Silent. No edit |
| `explore.md` | lines 78-80: "Read the accepted specifications." It reads them and never writes them | Silent. No edit |
| `migrate.md` | The table "What a source project holds, and where each part goes" (line 78 onward) has rows for `## ADDED Requirements` and `## RENAMED Requirements`, and none for `## MODIFIED Requirements`. The RENAMED row (85) takes the state `modified` "where the delta also restates the requirement under `## MODIFIED Requirements`". Lines 212 and 238 are about the identifier a restated heading resolves to | Not edited, as the proposal's "Not in this change" records. A migrated specification is bound by the new requirement like any other, and completing a source block that left a scenario out needs the git record of the source's merge. That obligation belongs to a change whose subject is migration, `migrate-into-the-extended-vocabulary` |
| `assets/template-specification.html` | One example requirement, `req-example`, `data-change="added"`, with one scenario carrying no `data-change` (lines 40-53). Nothing about modifying a requirement | No edit, for design decision 6's reason: the example is a placeholder `review/example-content` asks every author to replace, and the template is pinned by sha256 in `assets/vendor.lock.json` |

**Every body is either silent or edited.** The two that perform the merge, `sync.md` and
`archive.md`, are the two this change edits.
