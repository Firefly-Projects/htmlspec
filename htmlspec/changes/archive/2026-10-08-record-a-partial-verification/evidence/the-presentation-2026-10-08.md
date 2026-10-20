# A partial verification on the page, with styles on and off

**Date:** 2026-10-08
**Task:** 5.2
**Stylesheet:** `assets/sha256-03627137bcbb81c9/htmlspec.css`, the revision task 5.1 added. The print
stylesheet is unchanged, `sha256-04ac9a24fc436a87`.

The specification was written in a scratch project set up with `init --tools none` by this
change's binary, so its store holds `sha256-03627137bcbb81c9` and the document names it and
declares `0.6.0`. It is the template's specification with the measurement's renames, its
verification set to `partial`, a method paragraph, and two `unchecked` paragraphs:

```html
<section class="verification" id="verify-the-check-reached-part" data-change="added" data-outcome="partial" data-refs="req-a-check-covers-its-scenario">
  <h3>The example verification <span class="hs-id">verify-the-check-reached-part</span></h3>
  <p data-field="method">Ran the asset check against a project holding a second store, and read its exit status and rule.</p>
  <p data-field="unchecked">The Then names both locations, and the record gives only the rule and the exit status.</p>
  <p data-field="unchecked">The check did not reach a second store beside a change's documents.</p>
</section>
```

`htmlspec validate` accepts it, exit 0, with `review/stale-verification` on the verification and
`review/stale-reference` on the template's evidence unit. The styles-off copy is the same file with
every `<link rel="stylesheet">` removed.

Each page was rendered headless at 1100 by 3400 pixels with scripts off, and each render was read as
an image: the styles-on page with Firefox 157 (`javascript.enabled` false in the profile) and with
Chrome 154 (`--blink-settings=scriptEnabled=false`), and the styles-off page with Firefox. Chrome
rendered once and then hung on every later run in this session, reporting
`CVDisplayLinkCreateWithCGDisplay failed`, so Firefox carries the styles-off render.

## With styles on

- **The outcome word shows.** The verification's heading carries a bordered `PARTIAL` badge, in
  the amber of `--hs-state-modified`, the color `blocked` uses, beside the identifier. Both browsers
  drew it the same way.
- **Both `unchecked` paragraphs read as sentences**, set as body paragraphs after the method, with
  the spacing every paragraph in a unit has. Nothing is drawn before or after them.

## With styles off

- The heading reads "The example verification verify-the-check-reached-part", with no outcome word,
  as every badge is generated content and none is drawn without the stylesheet. The outcome stays
  in the markup and in the model.
- **Both `unchecked` paragraphs read as sentences**, as ordinary paragraphs after the method:
  "The Then names both locations, and the record gives only the rule and the exit status." and
  "The check did not reach a second store beside a change's documents."

## What the renders show beyond the task

**On the page, an `unchecked` paragraph looks exactly like the method.** Three paragraphs in a row,
set alike, and only their wording says which states what was done and which what was not. Design
decision 7 chose this: a generated label is for `renamed-from`, whose bare heading could be taken
for a claim, and `unchecked` holds a sentence the author writes, as `method` does. The renders
bear that out for these two sentences, each of which says "did not" or "only" in its own words. A
terse paragraph such as "The second location." would not. Task 9.1 puts it to the contributor.
