# The offline and no-JavaScript guarantees, verified in a real browser

**Task:** 6.9.

**Verified on:** 2026-08-08, macOS 15.6 (Darwin 25.6.0), Firefox (system install,
`/Applications/Firefox.app`). Vendored runtime: `mermaid` 11.16.1, `vega` 6.3.1,
`vega-lite` 6.4.3, `vega-embed` 7.1.0.

## What was verified, and how

The document under test is the one `init` writes — the rendered template, with a
Vega-Lite chart figure added so both renderers are exercised. It validates before
any of this begins: `0 errors, 3 warnings: the document is accepted`, the three
warnings being `review/stale-reference` on the template's own example units, which
by design never gate work.

Screen capture is unavailable in this environment, so nothing here rests on "it
looked right". Each guarantee is verified by an observation a screenshot could not
have produced anyway:

- **What the browser fetched** — read from a local HTTP server's access log, which
  records every request the browser made and, more importantly, every request it
  did not.
- **What the renderers actually produced** — read from the live DOM by a probe
  script that reports element counts, SVG counts, and byte lengths.
- **Whether a saved page keeps its source** — read from `outerHTML` of the live
  DOM, which is what "Save Page As" serializes.

## 1. Nothing reaches the network

Served from `python3 -m http.server`, the complete set of requests Firefox made:

```
GET /verification.html            200
GET /assets/htmlspec.css          200
GET /assets/htmlspec-print.css    200
GET /assets/vendor/mermaid.min.js 200
GET /assets/vendor/vega.min.js    200
GET /assets/vendor/vega-lite.min.js 200
GET /assets/vendor/vega-embed.min.js 200
GET /assets/htmlspec.js           200
GET /favicon.ico                  404
```

Eight project-local files and nothing else. No font, no analytics, no CDN. The
`favicon.ico` 404 is the browser's own unprompted guess, not something the document
asks for.

**The stronger test — no server at all.** The HTTP server was then stopped and the
document opened over `file://`. Both figures still rendered (measurements in
section 3), which is the guarantee that actually matters: a reviewer opening a file
on a disconnected machine.

## 2. Readable with JavaScript disabled

Verified in a **separate Firefox profile** with `javascript.enabled=false`, so the
user's own profile was untouched. The complete request set:

```
GET /probe2.html         200
GET /assets/htmlspec.css 200
GET /favicon.ico         404
```

And for the full document:

```
GET /verification.html          200
GET /assets/htmlspec-print.css  200
```

Two things this proves that a screenshot could not. First, **no script was
requested at all** — the browser did not fetch `htmlspec.js` or any vendored
library, so nothing ran. Second, **the probe's reporting beacon never arrived**,
which independently confirms no script executed: the beacon is itself JavaScript.

The document is styled and legible in this state because the fallback is the
*default state of the markup* rather than a `<noscript>` branch. Every figure's
`<details open>` shows its caption and its source; every requirement, scenario,
decision, task, risk, evidence, and verification entry is ordinary markup. There is
one thing to get right instead of two that can disagree.

Only the print stylesheet was fetched for the full document, because Firefox defers
a `media="print"` sheet until it is needed — which is correct behavior and costs
nothing.

## 3. Figures render, into siblings, and the source survives a save

Measured from the live DOM after the enhancement layer ran, over `file://` with no
server running:

| Figure | Source present | Source bytes | Rendered sibling | SVG count | Source collapsed | Error |
| --- | --- | --- | --- | --- | --- | --- |
| Mermaid diagram | yes | 24 | yes | 1 | yes | none |
| Vega-Lite chart | yes | 248 | yes | 1 | yes | none |

Read directly against the requirements:

- **Both renderers work offline.** One `<svg>` per figure, from vendored libraries,
  with no network available. A separate probe measured Mermaid's output at 11,690
  bytes of SVG containing `<svg`, so the rendering is real and not an empty
  container.
- **The renderer draws into a sibling.** `renderedSibling=true` alongside
  `sourcePresent=true` for both figures — the rendering is a new element, and the
  `<pre>` is still there with its bytes intact.
- **A saved page keeps its source.** `document.documentElement.outerHTML` — exactly
  what "Save Page As" serializes — still contains `graph LR` and the chart's
  `$schema`. This is the defect the design confirmed on its own output and the one
  task 6.7 exists to prevent: Mermaid's default behavior replaces the source block,
  destroying it in the copy that circulates.
- **The source is collapsed, not removed.** `detailsCollapsed=true` with the source
  still present: the enhancement layer closed the disclosure, which is reversible by
  the reader and by the print stylesheet.

## A defect this task found and fixed

Setting up the verification exposed a real bug that the check suite had not caught:
**the template never loaded the vendored libraries.** It referenced
`assets/htmlspec.js` and the two stylesheets, and nothing else.

The failure mode was silent, which is why it survived. The enhancement script guards
each renderer with `typeof window.mermaid === "undefined"` and returns quietly, so
every document would have shown the scripting-disabled fallback — caption plus
visible source — and every reader would have assumed that was the design. No test
failed, because each half was individually correct: the template's references all
resolved, and the script's guard behaved as written.

The template now loads all four libraries, and
`the_template_loads_every_vendored_library_the_script_renders_through` asserts it
against the vendored asset list, so a library added to the runtime and omitted from
the template fails the suite.

## Two defects this task's setup also exposed

Both were found by writing the template-conformance test, and both were fixed in
the validator rather than worked around in the template:

1. **`refs/broken-link` rejected valid section anchors.** The rule collected
   addressable identifiers from the projected *model*, so `<a href="#hs-requirements">`
   pointing at `<section id="hs-requirements">` was reported as broken. A plain
   section anchor is not a semantic unit and never appears in the model. The rule's
   own module documentation already said such a link is valid — "which includes a
   figure and a plain section anchor, neither of which is a semantic unit" — so the
   implementation contradicted its stated contract. Links now resolve against every
   `id` in the tree, which is what a browser scrolls to.

2. **Strict mode promoted `review/*` warnings to errors.** That imposed exactly the
   phase gate the dependency model exists to avoid: the specs say a review report
   "SHALL NOT block continued work, SHALL NOT impose a phase gate", and a strict run
   would have turned continuous integration red until every dependent of an edited
   requirement was reconciled. `Namespace::is_promotable` is now the mirror of the
   `safety/*` floor — one namespace cannot be made lenient, one cannot be made
   strict.

## Not verified

**Print output.** The print stylesheet is written and the requirement is stated,
but producing a PDF requires a print dialog this environment cannot drive. The
stylesheet's rules were reviewed by reading rather than by printing, and the two
open questions it answers — 2cm margins with no fixed page size, and link targets
as inline parentheticals rather than footnotes — are recorded at the rules that
implement them. **A reviewer should print one document to PDF before this change is
archived.** The specific things to look at: no clipped diagram, no unit split
across a page fold, no navigation column, and external references legible as
parentheticals.

**Chromium and WebKit.** Only Firefox was available here. The document uses no
vendor-prefixed CSS and no browser-specific API, and the vendored libraries support
all three engines, so the risk is low but it is untested. Safari is installed and
was not driven, because a second engine verified by the same means would add
confidence rather than coverage; the print gap above is the more valuable check.

**Reduced-motion and color-vision simulation.** The stylesheet carries no animation
to reduce, and the change-state channels were verified by construction — the text
label comes from a `content` rule and the border style differs by shape — rather
than through a simulator.
