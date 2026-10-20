# What the formatter rewrites today, measured against HEAD

**Date:** 2026-10-09
**Tasks:** 1.1. Task 5.1 repeats it once the work is done.
**Binary:** `htmlspec 0.28.0 (08f12a923d5c)`. `--version` prints `08f12a923d5c-dirty`, because the
working tree holds two untracked files outside the product. `git status` shows no modified tracked
file.

## 1. One formatter run in a scratch project

A scratch Git repository, initialized and given one change:

```
htmlspec init --tools claude      exit 0
htmlspec new change demo          exit 0
```

`init` stores the runtime under `htmlspec/assets/`, so the change's documents have a store above
them. The proposal's "What changes" placeholder paragraph was replaced with three paragraphs. Two
quote markup as prose, and one links to the stylesheet for reading:

```html
<p>A template links its stylesheet with <code>href="../../assets/htmlspec.css"</code>, and a script with <code>src="../../assets/htmlspec.js"</code>.</p>
<pre><code>&lt;link rel="stylesheet" href="../../assets/htmlspec-print.css"&gt;</code></pre>
<p>The <a href="../../assets/htmlspec.css">stylesheet</a> is linked for reading.</p>
```

The document was committed, and then the following was run:

```
htmlspec format --check htmlspec/changes/demo/proposal.html
htmlspec/changes/demo/proposal.html: htmlspec.css now names the bytes stored at ../../assets/sha256-03627137bcbb81c9/htmlspec.css
htmlspec/changes/demo/proposal.html: htmlspec.js now names the bytes stored at ../../assets/sha256-84427613110957a7/htmlspec.js
htmlspec/changes/demo/proposal.html: htmlspec-print.css now names the bytes stored at ../../assets/sha256-04ac9a24fc436a87/htmlspec-print.css
htmlspec/changes/demo/proposal.html: htmlspec.css now names the bytes stored at ../../assets/sha256-03627137bcbb81c9/htmlspec.css
htmlspec/changes/demo/proposal.html: not canonically formatted
exit 1
```

The document's own two `<link>` elements already name stored bytes, so all four fills come from the
three paragraphs. `htmlspec format` on the same file exits 0 and writes this diff:

```diff
-<p>A template links its stylesheet with <code>href="../../assets/htmlspec.css"</code>, and a script with <code>src="../../assets/htmlspec.js"</code>.</p>
-<pre><code>&lt;link rel="stylesheet" href="../../assets/htmlspec-print.css"&gt;</code></pre>
-<p>The <a href="../../assets/htmlspec.css">stylesheet</a> is linked for reading.</p>
+<p>A template links its stylesheet with <code>href="../../assets/sha256-03627137bcbb81c9/htmlspec.css"</code>, and a script with <code>src="../../assets/sha256-84427613110957a7/htmlspec.js"</code>.</p>
+<pre><code>&lt;link rel="stylesheet" href="../../assets/sha256-04ac9a24fc436a87/htmlspec-print.css"&gt;</code></pre>
+<p>The <a href="../../assets/sha256-03627137bcbb81c9/htmlspec.css">stylesheet</a> is linked for reading.</p>
```

Three of the four fills change the author's words: two `<code>` quotations and one escaped
`<pre><code>` example. None of the three is an attribute of any element. `htmlspec validate`
accepts the document both before and after, because the words were valid either way. After the
run, `format --check` exits 0. **So the author's prose must be wrong for the check to pass.**

The fourth fill is the `href` of an `<a>`, which is an attribute but not one the document loads.

## 2. The asset rules do not judge a hyperlink

The `<a>` was then pointed at an address the store does not hold,
`../../assets/sha256-0000000000000000/htmlspec.css`:

```
htmlspec validate htmlspec/changes/demo/proposal.html
  0 errors, 1 warning: the document is accepted
```

The warning is the placeholder summary's `review/example-content`. Next, the document's own
stylesheet `<link>` was pointed at the same absent address:

```
  error: asset/unresolved at /link[href]: <link href="../../assets/sha256-0000000000000000/htmlspec.css"> names bytes this project does not store. …
```

The asset rules read the attributes listed in `REFERENCING_ATTRIBUTES`
(`crates/htmlspec-validate/src/rules/assets.rs`): thirteen element and attribute pairs, `<a href>`
not among them. Its comment gives the reason: "a hyperlink is a citation, not a dependency".

The formatter reads a different list, `ATTRIBUTES` in `crates/htmlspec-format/src/references.rs`:
`href="` and `src="`, matched as substrings anywhere in the source. **The two disagree on what an
asset reference is**:

- The formatter fills text, which the asset rules never read, and hyperlinks, which they read and
  deliberately skip.
- The formatter misses `<object data>`, which the asset rules judge.
- Because it matches substrings, the formatter would also fill `data-src="…"` or `xlink:href="…"`.
  This is read from the code; it was not run.

## 3. Where it has already cost something

The self-migration's drafters met the fault twice, and both times they changed the author's markup
to work around it:

- `~/projects/html-spec-migration/review-log.md`, "Formatter defect found in drafting"
  (2026-09-29). Three sources quote such markup:
  - in content-address, the drafter split the `<code>`;
  - in establish-a-runbook, the drafter put an HTML comment between `href` and `="`;
  - log-a-runbook-run's path names no file, so it is not rewritten.
- The comment workaround is still in two staged documents:
  - `staged/htmlspec/changes/archive/2026-09-18-establish-a-runbook/design.html`;
  - `…/tasks.html`.
- `openspec/changes/archive/2026-10-08-migrate-into-the-extended-vocabulary/evidence/the-proof-run-2026-10-08.md`,
  "The formatter rewrites a quoted attribute in prose", found it again in the proof run.

None of this repository's own tracked HTML documents quotes an asset attribute in prose or links to
an asset from an `<a>`. A search of `git ls-files '*.html'` for either form finds nothing. Fixing
this rewrites no committed document.
