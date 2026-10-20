# The view, measured on the one-word case

**Date:** 2026-10-07
**Tasks:** 7.1, which rebuilds the one-word case in html-spec form and runs the view on it, and 7.2,
which marks the word, adds an unmarked edit beside it, and runs the two refusals
**Binary:** `./target/debug/htmlspec`, built from this branch with every task of groups 1 to 6 done.

## The scratch project

A copy of the staged migration, `~/projects/html-spec-migration/staged`, in the session scratchpad.
Three edits put it back in the state `sync-a-change-into-the-accepted-contract` was reviewed in:

1. `htmlspec/changes/archive/2026-08-31-sync-a-change-into-the-accepted-contract` moved to
   `htmlspec/changes/sync-a-change-into-the-accepted-contract`, without its `VERSION`.
2. In `htmlspec/specs/toolchain-packaging/spec.html`, the accepted
   `req-a-single-command-exposes-validate-format-and-init` replaced by the requirement as
   `2026-08-30-finish-the-archival` states it, with each `data-change` set to `unchanged` and the
   `renamed-from` paragraph left out, as an accepted specification states a requirement.
3. From the same file, the two requirements the change adds taken out:
   `req-the-command-surface-reports-whether-a-change-reached-the-accepted-contract` and
   `req-the-workflow-instructions-carry-a-stage-for-applying-a-change-s-contract`. Later changes
   merged both, so today's text holds them. Without this step, `sync` reports one of them reached and
   the other as content not taken, which is not the state the change was reviewed in. Task 7.1's
   recipe now names the step.

`spec.nix` was raised to `0.99.0`, so `archive --check` reaches the comparison. Before any task of
group 3, `sync --json` on this project gave `absent` holding the two added requirements, and
`untakenContent` holding one entry, `req-a-single-command-exposes-validate-format-and-init` with
`firstDifferingStatement` 3 and no scenario named. Task 3.1 checked that payload, and archival's,
byte for byte after the refactor.

## Task 7.1: the view

```
$ htmlspec show sync-a-change-into-the-accepted-contract --diff        # exit 0, nothing on stderr
sync-a-change-into-the-accepted-contract states 3 requirement(s), and compares 3 of them.
Each text is quoted, as projected, from the document named beside it. {+ +} marks words only the change states, and [- -] words only the accepted specification holds.

req-the-command-surface-reports-whether-a-change-reached-the-accepted-contract, added
  The command surface reports whether a change reached the accepted contract
  held by no accepted specification, so nothing is compared

req-the-workflow-instructions-carry-a-stage-for-applying-a-change-s-contract, added
  The workflow instructions carry a stage for applying a change's contract
  held by no accepted specification, so nothing is compared

req-a-single-command-exposes-validate-format-and-init, modified
  The command surface names its operations as OpenSpec names them
  compared with htmlspec/specs/toolchain-packaging/spec.html
  statements: 9 of 10 paragraphs held by both, words marked by comparison
    paragraph 3, and paragraph 3 in the accepted specification
      the accepted specification holds:
        The spellings are therefore fixed rather than left to an implementer: validate, archive, init, new, list, status, instructions, and store name the operations OpenSpec also provides. format and assets name operations it does not. The selectors --json, --specs, --change, and --store carry OpenSpec's spellings for the same reason.
      the change states:
        The spellings are therefore fixed rather than left to an implementer: validate, archive, init, new, list, status, {+sync,+} instructions, and store name the operations OpenSpec also provides. format and assets name operations it does not. The selectors --json, --specs, --change, and --store carry OpenSpec's spellings for the same reason.
  scenarios: 18 of 18 held by both
```

With `--json` (exit 0, nothing on standard error), the payload says the same: both added
requirements `held-by-none`, and the modified one `differs`, with `marks` `comparison`, ten
paragraphs of which nine are `both`, and one `pair` at position 3 and accepted position 3 whose
`changeStates` hold one run with `onlyThisSide`, `"sync, "`, and whose `acceptedHolds` hold none.
All 18 scenarios are `both`.

The SHA-256 of each of the 290 files in the project was taken before both runs and after them, and
the two lists are identical.

**Beside OpenSpec's hunk** (`the-measurement-2026-09-30.md`, section 1): both find one paragraph, the
third, and the one word. OpenSpec prints that paragraph twice whole, as a `-` line and a `+` line, and
leaves the reader to find `` `sync`, `` by comparing them. The view marks `{+sync,+}` on the change's
line and nothing on the accepted one, and counts the other nine paragraphs and the 18 scenarios
without printing them. OpenSpec printed both added requirements whole. The view gives each one line,
because no accepted specification holds either, which is the state the merge has to change.

## Task 7.2: the author's marks, and the two refusals

**The word marked as the change's author would mark it.** In the change's specification, `<code>sync</code>, `
became `<ins><code>sync</code>, </ins>`, the example `mark-the-words-a-modification-changed` gives.
The specification still validates (0 errors). The view's statements line now ends "words marked as the
author marked them", the pair and its one marked word are as before, and the payload's `marks` is
`author`.

**An unmarked edit beside it.** The same paragraph then lost "therefore" from "The spellings are
therefore fixed", without a mark. The text read with each deletion kept and each insertion left out
is no longer the accepted text, so the marks no longer account for the difference:

```
  statements: 9 of 10 paragraphs held by both, words marked by comparison, because the author's marks leave part of the difference unmarked
    paragraph 3, and paragraph 3 in the accepted specification
      the accepted specification holds:
        The spellings are [-therefore-] fixed rather than left to an implementer: validate, archive, init, new, list, status, instructions, …
      the change states:
        The spellings are fixed rather than left to an implementer: validate, archive, init, new, list, status, {+sync,+} instructions, …
```

`marks` is `unreconciled`, and both words are marked: `sync,` on the change's side and `therefore` on
the accepted one.

**The archived change, by its path.**

```
$ htmlspec show htmlspec/changes/archive/2026-08-30-finish-the-archival --diff      # exit 2
htmlspec: htmlspec/changes/archive/2026-08-30-finish-the-archival is an archived change. The accepted specifications hold its edit together with every later change's, so a comparison with them shows a later change's edit, reversed, where there is one, and nothing where there is none. `show --diff` compares a change before its merge. What an archived change modified is in its specification's marks, where it has them, and in the repository's history
```

With `--json`, standard output is empty (0 bytes) and the same report goes to standard error.

**Without `--diff`.**

```
$ htmlspec show sync-a-change-into-the-accepted-contract                            # exit 2
htmlspec: show has one view today, `htmlspec show <change> --diff`, which shows each requirement the change adds or modifies beside the accepted one. Pass --diff
```

With `--json`, standard output is empty.
