# The measurement repeated against the change

**Date:** 2026-10-09
**Task:** 5.1
**Binary:** `./target/debug/htmlspec`, built from this branch's working tree. `--version` prints
`htmlspec 0.28.0 (08f12a923d5c-dirty)`, because the change is uncommitted.

## 1. The scratch project from the measurement

This is the same project, at the same commit, as `the-measurement-2026-10-09.md` section 1. The
proposal holds the three paragraphs: two `<code>` quotations, the escaped `<pre><code>` example,
and the `<a href>`.

```
htmlspec format --check htmlspec/changes/demo/proposal.html
htmlspec/changes/demo/proposal.html is canonical
exit 0

htmlspec format htmlspec/changes/demo/proposal.html
htmlspec/changes/demo/proposal.html is canonical
exit 0
```

`git diff` afterwards is empty. **No fill is made, and the author's words pass the check as they
were written.** At HEAD the same run made four fills and failed the check.

Then the measurement's section 2, repeated:

| Edit | `validate` | `format --check` |
|---|---|---|
| The `<a>` names an absent address | 0 errors, 1 warning (`review/example-content`) | canonical, exit 0 |
| The `<link>` names the same absent address | `asset/unresolved at /link[href]` | fills `htmlspec.css` and reports "not canonically formatted", exit 1 |

The asset rules and the formatter now agree on both edits:

- Neither treats the hyperlink as a reference.
- Both act on the `<link>`. The asset rules report it, and the formatter fills it.

At HEAD the formatter also filled the hyperlink, which the asset rules ignore.

## 2. The staged migration

The staged tree, `~/projects/html-spec-migration/staged/`, was copied to the session scratchpad,
and only the copy was edited.

- `htmlspec format --check htmlspec` reports all 109 documents canonical, exit 0. The change
  reformats nothing the migration staged.
- In the copy, both of the drafter's `href<!-- -->=` workarounds in
  `htmlspec/changes/archive/2026-09-18-establish-a-runbook/` were restored to the author's
  `href=`. Then:

  | Document | The change | `htmlspec 0.18.0` |
  |---|---|---|
  | `design.html` | canonical, exit 0 | fills `htmlspec.css` inside the `<code>` quotation, exit 1 |
  | `tasks.html` | canonical, exit 0 | canonical, exit 0 |

  `htmlspec 0.18.0` is the binary installed in this machine's Nix profile. Its rewrite is HEAD's:
  `git log 6a591bd..08f12a9 -- crates/htmlspec-format/src/references.rs` is empty.

  `tasks.html` never needed the workaround. Its quotation, `href="../../assets/sha256-…"`, names no
  file, and the rewrite keeps a reference whose file the runtime does not ship. That is the case
  `review-log.md` records for log-a-runbook-run. **Of the two staged workarounds, one was needed,
  and the change makes it unnecessary.**
