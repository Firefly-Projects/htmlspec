# Final self-conformance: the finished toolchain against real documents

**Tasks:** 9.1 (convert a real capability spec), 9.2 (re-run the group 3 documents),
9.3 (review in a browser and fix what the contract gets wrong).

**Run on:** 2026-08-08, macOS 15.6, Firefox. Toolchain at `0.1.0`, contract at `0.1.0`.

## What was done

**9.1** — `specs/deterministic-formatting/spec.md` was converted into a conforming
HTMLSpec document at `openspec/changes/html-spec/documents/deterministic-formatting.html`,
scaffolded with `htmlspec init` and checked with `htmlspec validate` and
`htmlspec format --check`. It is the spec whose implementation had just been written,
chosen so the conversion could be judged for faithfulness rather than only for
validity.

**9.2** — The three hand-authored group 3 documents were re-run through the finished
toolchain, including strict validation, which did not exist when they were written.

**9.3** — All four documents were reviewed in Firefox, with the DOM measured by a probe
script rather than by eye, since screen capture is unavailable in this environment.

## Results

Every document is accepted, including under `--strict`:

| Document | Units | Errors | Warnings | Canonical | Strict |
| --- | --- | --- | --- | --- | --- |
| `deterministic-formatting.html` (converted) | 36 | 0 | 14 | yes | passes |
| `complex-change.html` | 19 | 0 | 14 | yes | passes |
| `simple-proposal.html` | 10 | 0 | 4 | yes | passes |
| `technical-investigation.html` | 5 | 0 | 2 | yes | passes |

Every warning is `review/*` — a dependent unit referencing a unit the document marks
`added`. That is correct and expected for a document where everything is new, and it is
precisely why strict mode does not promote those: the documents pass `--strict` while
still carrying 34 review markers between them.

Measured in the browser:

| Document | Stylesheets | Figures | Rendered | Horizontal overflow |
| --- | --- | --- | --- | --- |
| `deterministic-formatting.html` | 2 | 1 | 1 | none |
| `complex-change.html` | 0 | 0 | — | none |
| `simple-proposal.html` | 0 | 0 | — | none |
| `technical-investigation.html` | 0 | 4 | 0 | none |

Body prose never overflows horizontally in any document at 1100px, which is the
narrow-width requirement. The converted document's chart renders offline from `file://`
with the source still present and surviving a DOM save (`svg=1, srcPresent=true,
savedOk=true`).

## The conversion needed no contract change

This is the result worth stating plainly. **The conversion of a real capability spec
required no new vocabulary, no rule relaxation, and no schema edit.** It validated on
the first attempt and was canonical as hand-authored.

Everything the Markdown spec expressed had a home:

- Each `### Requirement:` became a `requirement` with its `SHALL` sentences as
  statements, and each `#### Scenario:` became a nested `scenario` with its
  Given/When/Then as a description list.
- The spec's one non-scenario paragraph — the note under "no diff churn" explaining that
  the requirement constrains the formatter and not the diff — became
  `data-field="note"`, which is exactly the field the group 3 dogfood exercise added for
  prose that is not a statement.
- Material the Markdown had nowhere to put gained a home: the design's rationale became
  three `decision` units, the measurements became three `evidence` units, the long-line
  trade-off became a `risk` with a mitigation, and how each requirement was checked
  became three `verification` units with methods.

That last point is the more interesting finding. The HTML document carries **more** than
the Markdown source it came from, because the Markdown spec had no vocabulary for
decisions, evidence, or verification and those lived in a separate `design.md`. Rendering
the same material as one addressable document is the thing this format was proposed for,
and it is now demonstrated rather than argued.

## Findings

### 1. The group 3 documents reference no shared assets — not a contract defect

All three hand-authored documents load zero stylesheets and zero scripts. They were
written in group 3, before the stylesheet, the template, and the enhancement layer
existed in group 6, so there was nothing for them to reference.

The consequence a reviewer sees: unstyled markup, and the four figures in
`technical-investigation.html` showing captions and sources rather than renderings.

**This was verified to be the cause rather than assumed.** Copies of the three documents
were given the shared asset references and re-measured: `technical-investigation.html`
then rendered **2** figures. Two and not four is also correct — its four figures are one
Mermaid diagram, one Vega-Lite chart, one MathML equation, and one code excerpt, and only
the first two have a renderer. MathML renders natively with no library, and a code
excerpt is already its own presentation.

**Resolution: left as they are, deliberately.** These three documents are fixtures whose
purpose is to pin the vocabulary, and the conformance suite asserts their projected
models. Adding presentation to them would change bytes that a fixture exists to fix,
for no gain in what they test. The document that demonstrates the full presentation is
the converted spec from 9.1, which is styled, scripted, and rendering.

The absence is also itself evidence for a requirement: three documents with no
stylesheet and no script are readable, correctly ordered, and validate — which is the
no-JavaScript readability rule holding in its strongest form.

### 2. Two validator defects, found by writing the template-conformance test

Both were fixed in the validator rather than worked around, and both are recorded in
`browser-verification.md` where they were found. Restated here because they are contract
corrections and this is the self-conformance record:

- **`refs/broken-link` rejected valid section anchors.** It resolved links against the
  projected *model*, so a link to `<section id="hs-requirements">` was reported broken,
  because a plain section anchor is not a semantic unit. The rule's own documentation
  already said such a link is valid. Links now resolve against every `id` in the tree,
  which is what a browser scrolls to.

- **Strict mode promoted `review/*` warnings to errors.** That imposed the phase gate
  the dependency model exists to avoid, contradicting a requirement that says such a
  report "SHALL NOT block continued work, SHALL NOT impose a phase gate".
  `Namespace::is_promotable` is now the mirror of the `safety/*` floor. The four
  documents above passing `--strict` with 34 review markers is the demonstration.

### 3. One template defect: the vendored libraries were never loaded

The template referenced `htmlspec.js` and the stylesheets but none of the four vendored
libraries, so `window.mermaid` was always undefined and no figure could ever render. The
enhancement script guards each renderer and returns quietly, so the failure was silent:
every document would have shown the scripting-disabled fallback and every reader would
have assumed that was the design.

Fixed, and
`the_template_loads_every_vendored_library_the_script_renders_through` now asserts it
against the vendored asset list.

### 4. Diagnostics explain themselves

The task asks whether any diagnostic fails to explain itself. Reading the 34 review
warnings and the deliberately-broken conformance fixtures: no. Each names the rule, the
unit, the offending identifier, and what to do — for example, "this task references
req-example, which this change marks added. Reconcile it when convenient; work continues
either way", which states the severity policy in the message rather than leaving a reader
to infer it from a warning label.

The one diagnostic that could mislead is `viz/*` on a Mermaid diagram, and it carries an
explicit note that validation is shallow rather than reporting a clean pass it cannot
justify.

## Contract changes resulting from this task

**None to the schema or the projection specification.** The vocabulary at `0.1.0`
represented every document put to it, including one it had not seen. Three
implementation defects were fixed (findings 2 and 3); no requirement was relaxed, no
unit type was added, and no rule was found too strict.

That is the acceptance signal the design named: the contract is credible once it has
survived contact with content it did not anticipate.

## Not verified

**Print output.** Carried over from `browser-verification.md` and still open: printing
to PDF needs a print dialog this environment cannot drive. A reviewer should print
`deterministic-formatting.html` before this change is archived, checking for a clipped
diagram, a unit split across a page fold, the absence of the navigation column, and
external references legible as parentheticals.

**Engines other than Firefox.** No vendor-prefixed CSS and no browser-specific API is
used, and the vendored libraries support all three engines, so the risk is low and
untested.
