## Context

Four templates, four masthead paragraphs, one per artifact kind:

| Template | The paragraph it writes |
| --- | --- |
| `template-proposal.html` | "Replace this paragraph with the case for the change. …" |
| `template-design.html` | "Replace this paragraph with what this design settles. …" |
| `template-tasks.html` | "Replace this paragraph with what this work delivers. …" |
| `template-specification.html` | "Replace this paragraph with what this document is for. …" |

Each is exactly one `<p>` inside `<header class="hs-masthead">`, and the projection specification
section 9.1.1 projects `summary` from every `p` in the document's header. So the text this change
reads is already in the model, addressable, and normalized — nothing here parses HTML that the
projection does not already parse.

Two existing facts shape the approach:

- `htmlspec_assets::example_identifiers()` reads example identifiers out of the templates by
  scanning for `id="…"`, computed once in a `OnceLock`. The discipline is stated in its own doc
  comment: a `const` array of the current identifiers "would be a second statement of a fact the
  templates already make", and `project-bootstrap` names that shape as the one that produced a gate
  comparing against a value no project ever had.
- `review/example-content` is a **warning**, and `strict` mode never promotes a `review/*` rule.
  Archival escalates it to a refusal through `example_content_in`, which archival and the comparison
  operation share.

## Goals / Non-Goals

**Goals**

- A document whose summary is its template's placeholder is reported, for every artifact kind.
- Archival refuses one, on the same path that already refuses an example identifier.
- The placeholder text is derived from the templates, so rewording a template does not need a
  second edit.

**Non-Goals**

- Judging whether a real summary is a *good* one. That is reading, and `artifact-kinds` reserves it
  for an agent.
- Reporting an absent summary. A different fact, and whatever governs it already governs it.
- Tightening `every_fixture_reports_exactly_the_rules_it_declares`, which the proposal records as a
  separate finding.

## Decisions

### The comparison is whole-summary equality, not a substring search

**This is the decision the whole change turns on.** A substring search for "Replace this paragraph"
would fire on a document whose author wrote *"Unlike the template's 'Replace this paragraph' text,
this summary states the actual argument"* — a correct summary reported as a placeholder.

Worse, it would fire on this project's own explainers and on any document discussing the rule. A
rule that fires on documents about itself is the shape that teaches an agent to ignore it, which is
the failure the pointer rule's deletion test was added to prevent one release ago.

So the comparison normalizes whitespace and compares the **whole** projected summary against the
whole placeholder. A summary that *is* the placeholder is reported. A summary that merely contains
it is the author's and is not.

The cost, accepted: a document whose summary is the placeholder plus one sentence of the author's
own is not reported. That is a half-edited masthead, and it is the same trade the identifier rule
already makes in the other direction — it reports a half-replaced unit because the identifier
survives, and here the text is what survives, so the boundary falls elsewhere. Reporting both halves
of both cases needs a rule that guesses at intent, and this project does not write those.

### The placeholder text is read out of the templates, not listed beside them

`example_placeholders()` scans each template's `<header class="hs-masthead">` for its `<p>` text,
normalized the way the projection normalizes it, and caches the set in a `OnceLock` — the same shape
`example_identifiers()` already has, for the same stated reason.

A `const` array of the four sentences would be a second statement of a fact the templates already
make. The scenario "A template that rewords its placeholder" exists to pin that: rewording a
template must not need a second edit, because the edit nobody remembers is the one that goes stale.

**The normalization has to match the projection's**, or the comparison fails on whitespace the model
never carried. That is one shared helper rather than two, since two would drift and the drift would
be invisible — a rule that silently stops matching is worse than one that never matched.

### Reported at the document, not at a unit

`Location::unit(id)` is what the identifier case uses, and a summary has no identifier. So the
summary diagnostic is located at the document, which `Location::document()` already provides and
which the schema rule already uses.

The message names the summary rather than a unit, and says what to do: state what the document is
for. A contributor reading "this document still carries the identifier a template writes" for a
paragraph with no identifier would be looking for a unit that does not exist.

### Severity is unchanged, and the reason is unchanged

A warning at validation and a refusal at archival. The requirement already states why, and the
reason applies here verbatim: creation writes the template, and the contract requires that result to
be valid, so an error at creation would report a defect the program itself introduced.

`htmlspec new document` writes a placeholder summary every time. An error would make every freshly
created document invalid, which contradicts `document-template`'s requirement that creation produce
a valid document.

## Risks / Trade-offs

- **A rule that fires on documents about itself** → whole-summary equality rather than a substring
  search. This is the risk that shaped the main decision rather than one mitigated after the fact.
- **The normalization drifts from the projection's** → one shared helper, and a test comparing the
  scanned placeholder against the projected summary of a freshly created document. If those two
  disagree, the rule silently stops matching, so the test asserts they agree rather than asserting
  the rule fires.
- **A half-edited masthead is not reported** → accepted and stated above. Detecting it needs a rule
  that guesses at intent.
- **50 fixtures carry the placeholder** → none breaks, verified rather than assumed: `valid-minimal`
  already reports `review/example-content` seven times with an empty declared rule list and passes.
  The suite asserts declared rules are present and does not assert others are absent.

## Migration Plan

Additive. The rule reports a case nothing reported before, at warning severity, so no existing
document becomes invalid and no fixture's `valid` field changes. Archival gains a refusal on a
condition that has never been intentionally archived.

One fixture is added, pinning a document whose units are replaced and whose summary is not — the
case this change exists for, which is also the case no existing fixture covers.
