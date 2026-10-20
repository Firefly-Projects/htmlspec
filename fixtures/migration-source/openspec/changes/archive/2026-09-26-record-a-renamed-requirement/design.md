## Context

See proposal.md for why. The shape of the current system that the approach depends on:

- **Identity already survives a rename.** document-schema's "Every semantic unit has a stable unique
  identifier" keeps a renamed requirement's `id`. So nothing here touches identifiers. What is
  missing is a place to state the heading the requirement replaced.
- **Change states are one attribute with a closed vocabulary.** `data-change` is `added`, `modified`,
  `removed`, or `unchanged` (the `changeState` definition in
  `crates/htmlspec-schema/schema/document.schema.json`). It gates nothing, and review warnings and
  presentation read it.
- **Unit fields already come from marked paragraphs.** The projection specification's section 4
  (`crates/htmlspec-schema/doc/projection.md`) projects `data-field="rationale"`, `"reason"`,
  `"mitigation"`, `"method"`, and `"note"` paragraphs into named fields. A requirement currently
  accepts `statement` and `note`.
- **The merge check compares identifiers only.** `merge_state` in
  `crates/htmlspec-cli/src/commands.rs` is the one comparison that `sync` and archival share, and
  it partitions a change's stated identifiers into reached and absent. A renamed requirement's
  identifier is already held, so an unapplied rename lands in "reached".
- **OpenSpec's semantics are the reference.** `openspec` 1.13.1 (`dist/core/specs-apply.js`)
  applies RENAMED before REMOVED, MODIFIED, and ADDED. It requires a MODIFIED block to name the
  new heading, and it rejects a requirement that is both RENAMED and REMOVED. It rejects a RENAMED
  target that collides with an ADDED requirement and a rename into a spec that does not yet exist.
  It treats a rename whose source is gone and whose target is present as already applied, which is
  a no-op and not an error. Main specs carry no RENAMED section.

## Goals / Non-Goals

**Goals:**

- A change can state that a requirement replaced a heading, readably and in the projected model.
- A renamed-and-rewritten requirement records both facts, as OpenSpec's RENAMED plus MODIFIED does.
- An unapplied rename is reported by the checks that report an unapplied change.
- Migration carries OpenSpec's RENAMED sections without an open question.

**Non-Goals:**

- **Renaming any unit other than a requirement.** OpenSpec renames requirements only, and no
  observed case needs more. A scenario's heading can change under `modified` today, and nothing
  addresses scenarios across changes by heading.
- **A rename history in the accepted contract.** The accepted specification states the current
  heading, and the archive holds the history, as OpenSpec's main specs do.
- **A program that performs the merge.** This stays the agent's work under artifact-kinds' "Every
  synthesis and every contract update is agent-driven". The program only reports.
- **Inferring a rename.** Nothing detects that a heading changed and proposes a `renamed-from`.
  A rename is stated by its author or not at all.

## Decisions

### 1. A rename is a field on the requirement, separate from `data-change`

The requirement's own heading is the new name. A single `<p data-field="renamed-from">` paragraph
holds the previous heading. `data-change` keeps describing what happened to the content.

| Source (OpenSpec delta) | html-spec requirement |
| --- | --- |
| RENAMED only | `data-change="unchanged"` + `renamed-from` |
| RENAMED + MODIFIED (new heading) | `data-change="modified"` + `renamed-from` |
| RENAMED + REMOVED | invalid, as OpenSpec rejects it |
| RENAMED `TO:` = an ADDED heading | invalid: an `added` requirement cannot carry `renamed-from` |

**Alternatives considered.**

- **A fifth change state, `renamed`.** Rejected, and the contributor rejected it as well. A unit
  carries one state, so a requirement renamed and rewritten in one change loses one of the two
  facts. Both of this project's own renames are that case.
- **A `data-renamed-from` attribute on the requirement element.** This was the first shape proposed,
  and it was chosen over the fifth state for the same reason. It was then replaced by the field for
  three reasons:
  - An attribute is invisible in the rendered page unless a stylesheet exposes it, so the previous
    heading is present only for tools.
  - A field paragraph is ordinary text, so it reads with scripts off and with styles off.
    document-validation's "No-JavaScript readability is enforced" asks for content present in the
    markup, and a paragraph meets that more fully than an attribute value does.
  - The projection already has the machinery for fields, validation already reports an unknown
    `data-field`, and the formatter already handles paragraphs. An attribute needs a new projection
    path, a new formatter case, and a new schema property shape.

  The substance of the contributor's choice was "separate from the change state", and a field keeps
  that. **This is a departure from the preview the contributor selected, and the proposal and this
  decision say so, so that it can be reversed at review.**

### 2. `unchanged` plus `renamed-from` is a pure rename

The `changeState` description of `unchanged` becomes: "the unit's statements and scenarios are
carried forward untouched. A requirement whose heading changed says so with a `renamed-from`
paragraph." Without this, a pure rename has no correct state: `modified` claims the obligation
changed, and `unchanged` claims nothing changed.

**Alternative:** require `modified` for every rename. Rejected, because it maps OpenSpec's
RENAMED-only case onto the state for a content change. That is the substitution project-migration's
"recorded as open, not mapped to the nearest state" exists to prevent.

### 3. The placement rules are errors

The rules are:

- the field appears only on a requirement;
- not on an `added` or `removed` requirement;
- at most once per requirement;
- not empty;
- not equal to the requirement's own heading.

These are structural facts about one document, like "A decision records its outcome and rationale",
so they are errors in `htmlspec-validate` and not review warnings.

**The heading a rename is compared with is the requirement's *heading name*.** That is the projected
`title` with the unit's own identifier removed from its end and the result trimmed. This was first
written as "the projected `title`, which excludes the `hs-id` span", and task 1.2 measured that
premise false. The projection takes a title from all of the heading's text, and nothing
special-cases `class="hs-id"`. So the template's `<h3>The old heading <span class="hs-id">req-x</span></h3>`
projects `The old heading req-x` (`evidence/the-premises-2026-09-26.md`). Compared with that title,
a `renamed-from` of `The old heading` would never be equal.

The name is derived from the model alone: the title and the identifier the unit already carries.
So the projection is unchanged, and a heading written with the span, without it, or with no space
before it names the same heading. Changing projection section 3.1 to drop `.hs-id` text from every
title was the alternative. It was rejected because it reshapes the projected title of every existing
document to serve two comparisons that need only a derived name.

A `renamed-from` paragraph outside a requirement is reported by its own rule. Projection section
4.0 does not also report it as a field the unit lacks, so one defect produces one diagnostic, and
that diagnostic says a rename belongs on a requirement. An empty `renamed-from` paragraph is kept by
the projection so the "empty" rule can see it. Every other empty paragraph is still discarded.

### 4. The merge comparison gains a fourth set: untaken renames

`merge_state` keeps its three sets (stated, reached, absent) and adds `untakenRenames`. That set
holds each stated requirement that carries `renamedFrom`, whose identifier the accepted
specifications hold, and whose accepted heading name differs from the change's heading name
(decision 3). Each entry names the identifier, the change's heading, and the accepted heading. Heading
names rather than titles, so an accepted specification that writes the `hs-id` span and a change that
does not are not reported as a rename.

- An untaken rename is **not** also in `reached`, so "reached" keeps meaning "the contract took this".
- An identifier that no accepted specification holds is `absent` and never an untaken rename. One
  finding per requirement.
- "Not synced" becomes `absent` or `untakenRenames` being non-empty. Archival's structured
  `notSynced`-style value and its question derive from that, through the same function, so `sync`
  and archival cannot disagree. That keeps the invariant toolchain-packaging's "The operation and
  archival agree" scenario states.
- The accepted heading equal to the **new** heading means the rename was applied: reached, and
  nothing reported. This is OpenSpec's "source gone, target present is a no-op".

**Alternative:** compare every requirement's heading, renamed or not. Rejected. A heading that
differs without a stated rename is either an unstated rename or an edit made directly to the accepted
contract, and neither is this change's subject. Reporting it would make the check read headings for
a purpose the contract never gave it.

### 5. The accepted specification does not carry `renamed-from`

The `sync` and `archive` instructions state that a merged rename changes the heading, keeps the
identifier, and drops the `renamed-from` paragraph. The program does not enforce this, because it
cannot tell an accepted specification from a change's by content. See Risks.

### 6. Migration keeps one identifier through a chain of renames

`migrate.md`'s mapping table replaces its row "`## RENAMED Requirements` → *no state exists*" with
"→ `renamed-from` on the `TO:` requirement, state from the rest of the delta". The identifier rule
changes to follow document-schema:

- A requirement named in a RENAMED `TO:` takes the identifier its `FROM:` heading already resolved to.
- So a chain `A → B → C` resolves to `id(A)` in every change and in the accepted specification.

The plan's `derives:` notes stay one per stated heading. The renaming change's note reads
`derives: B -> id(A)`, and a `renamed:` note records `A -> B`. The coverage check's collision rule
is unchanged. It compares headings within one destination document, and a renaming change's
specification states only `B`.

A heading that changed through REMOVED plus ADDED, with no RENAMED, derives independently. The
source did not say that the two are one obligation.

### 7. Presentation labels the field with generated content

A `[data-field="renamed-from"]::before { content: "Renamed from"; }` rule, plus quotation styling,
goes in the screen and print stylesheets. This follows the existing "text channel" for change
states, which the stylesheet already supplies as generated content. Without the stylesheet, the
paragraph still reads as the previous heading. Both stylesheets are content-addressed, so this is a
new revision of each at a new address, added through `assets/vendor.lock.json`'s procedure. The
templates move to the new addresses, and documents that already exist keep the revision they name.

### 8. Schema version

The change adds an optional field and makes no existing document invalid. That is a minor bump
under the `crates/htmlspec-schema/CHANGELOG.md` bump table, to `0.2.0`, recorded there with the
projection specification's section 4 edit.

**The schema accepts both `0.1.0` and `0.2.0`.** Before this change, `schemaVersion` was a `const`
and the match was exact. So a bump to `0.2.0` alone would invalidate every document that declares
`0.1.0`, which is every document written so far, 23 in this repository alone. That contradicts the
bump table's own meaning of minor ("every existing document still validates"). The policy deferred
a compatibility range "until a second version exists", and this change creates one. So the question
is settled here: `schemaVersion` becomes a closed enumeration of the versions this toolchain accepts,
and the match stays exact against that set. `CONTRACT_VERSION`, and the version every template and
new document declares, becomes `0.2.0`. A patch or minor bump adds its version to the set, and a
major bump is where versions leave it.

**Alternatives considered.** Bumping to exactly `0.2.0` and rewriting every document in this
repository was rejected, because documents in other projects would stop validating although the
proposal says documents already written are unaffected. Not bumping was also rejected. The `kind`
change did replace `status` without a bump, but that precedent contradicts the bump table this
decision is bound by.

The accepted vocabulary is the newest version's. A document declaring `0.1.0` that carries a
`renamed-from` paragraph validates here and fails on a `0.1.0` toolchain with the unknown-field
fault. A document declaring `0.2.0` fails there too, on its version. So a downgrade is loud either
way. A rule tying each field to the version that introduced it is not added. No second field has
needed it, and the schema keeps to a subset with no conditional composition.

### 9. Deliberate differences from OpenSpec's rename rules

Task 1.3 listed each rule in `openspec` 1.13.1's `specs-apply.js` against its counterpart here
(`evidence/the-premises-2026-09-26.md`). Five of them have no counterpart, on purpose:

- a duplicate `TO:` heading in one delta;
- a `FROM:` spelled like an accepted heading apart from case or spacing ("typo'd header");
- a `TO:` heading that already exists alongside the `FROM:` ("target already exists");
- a `TO:` heading that differs from another requirement only in case or spacing;
- a chain of renames inside one delta. Here one requirement carries at most one `renamed-from`, so a
  chain inside one change is stated as its first heading, and migration collapses a source chain
  to its first `FROM:`.

The reason is the same for all five. OpenSpec finds the requirement a rename applies to by its
heading, so those rules stop a rename from reaching the wrong requirement or none. Here the
requirement is found by its identifier, and `renamed-from` is never a lookup key. The merge
comparison reads the accepted requirement that carries the same identifier and reports that
requirement's actual heading. So a mistyped `renamed-from` misdirects nothing, and two requirements
sharing a heading are still two identifiers.

## Risks / Trade-offs

- **An accepted specification can keep a `renamed-from` paragraph that a merge left behind.** →
  The `sync` instructions say to drop it. A stale one is still a true statement about the last
  rename, so it misleads less than a wrong heading does. If it recurs, a later change can add a check
  scoped to `htmlspec/specs/`.
- **An unlabeled line when styles are off.** → The field is the previous heading as plain text, and
  it sits beside a heading that differs from it. This is the same degradation the change-state
  badges already accept ("Removing channel 3 entirely would leave the document fully legible").
- **The heading comparison and the formatter.** A heading reflowed by `htmlspec format` must still
  compare equal. → The comparison uses the projected `title` after whitespace normalization, which
  is what the projection already guarantees stable across formatting.
- **The fourth set changes a payload that agents read.** → `untakenRenames` is additive. The
  `stated`, `reached`, and `absent` keys and their meanings are unchanged, except that an untaken
  rename moves out of `reached`. That move is the defect this change fixes, and the release note says
  so.
- **The migration plan written before this change** records two renames as open. → It is paused, and
  the migration resumes against this contract. The two notes become `renamed-from` paragraphs, and
  the renamed toolchain-packaging requirement keeps `req-a-single-command-exposes-validate-format-and-init`
  as its identifier.

## Migration Plan

This is additive for documents. No existing document carries the field, and no existing
requirement's meaning changes. Rollback is a revert: a document that adopted the field fails
validation on the older toolchain with the unknown-field error that already exists, so a downgrade
is loud and not silent.
