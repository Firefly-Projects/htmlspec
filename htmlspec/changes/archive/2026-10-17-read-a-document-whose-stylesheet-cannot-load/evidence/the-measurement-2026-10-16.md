# What was measured before the change, 2026-10-15 and 2026-10-16

## Where a browser loads a stylesheet from, opening a file from disk

A scratch page, `a/page/test.html`, loaded four stylesheets, each colouring one line green:

| Stylesheet | Place | Safari | Chrome 154 | Firefox 158 |
| --- | --- | --- | --- | --- |
| `same.css` | beside the page | loaded | loaded | loaded |
| `sub/child.css` | a subfolder | loaded | loaded | loaded |
| `../parent.css` | one folder up | refused | loaded | loaded |
| `../../grand.css` | two folders up | refused | loaded | loaded |

Safari was read from the user's own screen on 2026-10-15: the first two lines green and the last
two red. Safari cannot be driven headless here. Chrome and Firefox were run headless on 2026-10-16,
and both loaded every stylesheet and every classic script from any folder. Edge is not installed.

Microsoft Edge is built on Chromium, so it is expected to behave as Chrome does. That was not
measured.

## Where each kind of document reaches the store

| Document | Reference to the store |
| --- | --- |
| an accepted specification, `htmlspec/specs/<name>.html` | `../assets/` |
| an accepted specification, `htmlspec/specs/<name>/spec.html` | `../../assets/` |
| a change's document, `htmlspec/changes/<name>/*.html` | `../../assets/` |
| a runbook's document, `htmlspec/runbooks/<name>/*.html` | `../../assets/` |
| an archived change's document | `../../../assets/` |
| a run's record, `htmlspec/runbooks/<name>/runs/<record>.html` | `../../../assets/` |

Every one is a parent folder, so in Safari, from disk, no document gets its stylesheet. The user
saw this first on a runbook in another project, then on that project's specification: both came
up as bare HTML, the masthead's fields stacked as a plain definition list. Firefox rendered the
same two files styled.

## What the store's stylesheet sets for the four selectors

From `htmlspec.css` at `sha256-03627137bcbb81c9`, comments removed. Rules outside any at-rule:

```
body { margin: 0; padding: 0; background: var(--hs-paper); color: var(--hs-ink);
       font-family: var(--hs-body-font); font-size: 1.0625rem; line-height: 1.55;
       text-rendering: optimizeLegibility; }
body > .hs-page, body { max-width: …; margin: 0 auto; padding: 0 var(--hs-gutter) 4rem; }
code, kbd, samp { font-family: var(--hs-mono-font); font-size: 0.88em; }
pre { font-family: var(--hs-mono-font); font-size: 0.8rem; line-height: 1.45;
      background: var(--hs-highlight); border-left: 2px solid var(--hs-rule);
      padding: 0.7rem 0.9rem; margin: 0.4rem 0; overflow-x: auto; white-space: pre; }
.hs-skip-link { position: absolute; left: -9999px; top: 0; … }
```

The custom properties resolve, in `:root`, to `--hs-paper: #fdfcfa`, `--hs-ink: #1a1a1a`,
`--hs-highlight: #f2efe9`, `--hs-rule: #d6d2ca`, `--hs-body-font: Charter, Georgia, "Times New
Roman", Times, serif`, and `--hs-mono-font: ui-monospace, "SF Mono", Menlo, Consolas, "Liberation
Mono", monospace`. The stylesheet declares no `@layer`. Its at-rules are three `@media` rules:
`(max-width: 34rem)`, `(min-width: 60rem)`, and `(prefers-reduced-motion: reduce)`.

## The block validates as the document without it does

A scratch project, made with `htmlspec init --tools none`, held two specifications from `htmlspec
new document specification`. One had the fallback block inserted before its first stylesheet link
and was then formatted. `htmlspec validate --strict` reported the same diagnostics for both: 8
`review/example-content` and 2 `review/stale-reference`, all from the template's example content.
`htmlspec format` kept the `<style>` element's text byte for byte and indented only the tag.

## The explainer this change started from

The block's shape, a 529-byte `@layer htmlspec-fallback` setting only properties the store's
stylesheet also sets, came from an explainer written on 2026-10-16 outside the repository. Its four
Chrome renders of the `runbook-portability` specification gave identical screenshots with the store
reachable, with and without the block, and a readable centred column with the store unreachable.
Task 3.2 repeats that measurement with this change's block, whose values are taken from the store.
