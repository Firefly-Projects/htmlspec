## Why

html-spec can keep a renamed requirement's identity, and it cannot say that the requirement was
renamed.

The identity half is already contract. document-schema's "Every semantic unit has a stable unique
identifier" states, in its scenario "Renaming a requirement", that a requirement whose title changes
but which describes the same obligation keeps its `id` rather than regenerating one from the new
title. So a change that renames a requirement writes the new heading on the old identifier, and
everything that addresses the requirement by identifier keeps resolving.

**What no document can state is the heading it replaced.** The change-state vocabulary is `added`,
`modified`, `removed`, and `unchanged`, and none of them means "this requirement used to be called
something else". A reader of a renaming change sees a requirement under a heading the accepted
contract does not hold, and nothing in the document tells them which accepted requirement it is
except the identifier they would have to go and look up.

OpenSpec, which html-spec is a spinoff of, records this directly. A delta carries a
`## RENAMED Requirements` section of `FROM:`/`TO:` pairs, applied before anything else, and a
requirement that is renamed and rewritten in one change appears in both RENAMED and MODIFIED — the
MODIFIED block under the new heading. A rename is a fact about the heading, and a modification is a
fact about the content, and OpenSpec keeps them apart.

**This project met the gap migrating itself.** Two of its own archived changes rename a requirement
— both in `toolchain-packaging`, and both also rewrite it in the same delta:

- `2026-08-17-project-bootstrap-cli`: "A single command exposes validate, format, and init" →
  "A single command exposes validate, format, bootstrap, and create", with a paragraph explaining
  that the requirement's subject changed rather than its topic.
- `2026-08-30-finish-the-archival`: that heading → "The command surface names its operations as
  OpenSpec names them".

project-migration's "A source distinction the destination cannot express is recorded as open, not
mapped to the nearest state" forbade mapping either onto `modified`, and named this as the change
that would settle it. The migration plan for this repository records both as open, carries each
rename only as prose, and so loses the one link a reader of the archive needs: from the heading the
contract holds today back to the headings the history used.

There is a second, quieter cost. Archival and `sync` decide whether a change reached the accepted
contract by comparing requirement identifiers (toolchain-packaging's "Archival assesses the merge and
never performs it"). A renamed requirement keeps its identifier, so a rename that was never applied to
the accepted specification is reported as reached. **Today a forgotten rename is invisible to every
check the project has.**

## What Changes

- **A requirement MAY state the heading it replaced**, in one paragraph marked
  `data-field="renamed-from"`. The requirement's own heading is the new name. This is OpenSpec's
  `FROM:`/`TO:` pair, expressed on the unit rather than in a section, which is where html-spec already
  expresses every other change state.
- **The rename is separate from the change state.** A requirement renamed and rewritten carries
  `data-change="modified"` and a `renamed-from` paragraph. A requirement renamed and otherwise
  untouched carries `data-change="unchanged"` and a `renamed-from` paragraph. OpenSpec's two cases
  — RENAMED alone, and RENAMED with MODIFIED — both survive.
- **Validation rejects a rename that cannot mean anything**: on a unit other than a requirement, on
  an `added` requirement (a new requirement has no previous name), on a `removed` requirement (OpenSpec
  rejects RENAMED and REMOVED on one requirement), more than once in one requirement, empty, or equal
  to the requirement's own heading.
- **The projection and the published schema carry it** as the requirement's `renamedFrom` field, so a
  consumer reads the previous heading without parsing prose.
- **A rename reaches the accepted contract as a new heading under the same identifier**, and the
  accepted specification does not carry the `renamed-from` paragraph. The history lives in the
  archived change that renamed it, as OpenSpec's main specs carry no RENAMED section.
- **The merge comparison sees a rename.** Where a change's requirement states the heading it replaced,
  the reporting operation and archival compare that requirement's heading with the accepted one, and
  report a rename the accepted specification has not taken. The comparison is of two strings, not a
  reading for meaning.
- **Migration carries a source rename as the heading it replaced**, not as an open question. The
  renamed requirement keeps the identifier derived from the heading it was established under, so one
  obligation holds one identifier across the changes that renamed it.
- **Presentation shows the previous heading** without scripts, because the paragraph is ordinary
  markup. No stylesheet change is required for the heading to be readable.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `document-schema`: a requirement may state the heading it replaced, and the rules that bound it.
- `artifact-kinds`: the merge applies a rename as a new heading on the same identifier and does not
  carry the previous heading into the accepted contract.
- `toolchain-packaging`: the reporting operation and archival's merge assessment compare headings for
  renamed requirements and report a rename that has not been applied.
- `project-migration`: a source RENAMED section is carried as the heading a requirement replaced, and
  a renamed requirement keeps one identifier across the changes that renamed it.

## Impact

- `crates/htmlspec-schema/schema/document.schema.json` and `doc/projection.md` — the `renamedFrom`
  field on a requirement, and the field table in section 4. The schema's `schemaVersion` policy
  decides the bump.
- `crates/htmlspec-parse` — project the field.
- `crates/htmlspec-validate` — the placement and consistency rules, as errors.
- `crates/htmlspec-cli/src/commands.rs` — `merge_state`, shared by `sync` and archival, gains the
  heading comparison and a fourth reported set.
- `crates/htmlspec-assets/instructions/` — `sync.md` and `archive.md` state how a rename is merged;
  `migrate.md` replaces its "no state exists" row for `## RENAMED Requirements`. Installed copies
  under `.claude/`, `.codex/`, and `.pi/` are regenerated, not edited.
- `crates/htmlspec-cli/tests/migrate.rs` — the test that fails the day a rename becomes expressible.
- Conformance fixtures for each validation rule, valid and invalid.
- Documents already written are unaffected: no existing document carries the field, and a
  requirement without it means exactly what it meant before.
- The paused migration of this repository: its plan's two open rename questions become
  `renamed-from` paragraphs, and the renamed `toolchain-packaging` requirement keeps the identifier
  derived from its first heading.
