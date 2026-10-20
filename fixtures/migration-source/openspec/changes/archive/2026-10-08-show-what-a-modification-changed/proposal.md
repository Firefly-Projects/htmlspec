## Why

**A reviewer of an html-spec change sees `modified` on a requirement and has to find the edit
alone.** `evidence/the-measurement-2026-09-30.md` measures what that costs. The staged migration
holds 34 modified requirements in 19 changes. The median is 7 statement paragraphs and 7
scenarios, and the largest is 15 paragraphs and 18 scenarios (section 4). The plainest case is
`2026-08-31-sync-a-change-into-the-accepted-contract`. It modified a requirement of 10 paragraphs,
491 words, and 18 scenarios, and its whole edit was one word.

**OpenSpec has a view for this, and html-spec has none.** Rebuilt as it stood while that change was
open, `openspec show sync-a-change-into-the-accepted-contract --diff` prints one hunk: the third
paragraph twice, once marked `-` and once marked `+`, differing by `` `sync`, `` (section 1).
`htmlspec show` is an unrecognized subcommand, and exits 2 (section 3).

This set of changes makes the gap wider and the view possible:

- `state-a-modified-requirement-whole` has a change restate a modified requirement whole, so the
  text a reviewer searches is always the whole requirement.
- `mark-the-words-a-modification-changed` lets an author mark words with `<ins>` and `<del>`. Its
  design says a mark claims only what it marks, and that a diff against the accepted text is where
  an unmarked word shows.
- `see-a-modification-the-merge-did-not-take` compares a modified requirement's text with the
  accepted text for `sync` and archival. It reports where the two differ and never what they say,
  on purpose, and its design leaves a diff view to this change.

toolchain-packaging's "The command surface names its operations as OpenSpec names them" decides the
name: "Where an operation corresponds to one OpenSpec provides, it SHALL carry OpenSpec's name for
it." So the view is `htmlspec show <change> --diff`.

**Why now.** The project's migration out of OpenSpec is staged and waiting. Once it lands, this
project writes its changes in html-spec form and reviews every modified requirement there. Without
this change the gap would be recorded as an open question in the migrated documents, or worked
around. Today it is worked around in prose: five staged changes narrate in their summary what moved
inside a modified requirement (`mark-the-words-a-modification-changed`'s evidence, section 4).

This change follows `mark-the-words-a-modification-changed`, whose marks it shows. It also depends on
`see-a-modification-the-merge-did-not-take`, whose pairing of a change's requirement with the accepted
one, and whose comparison of their text, it reuses rather than restates.

## What Changes

- **`htmlspec show <change> --diff` shows each modified requirement beside the accepted one.** For
  each requirement whose content `sync` compares, which is one the change marks `added` or `modified`
  and one marked `unchanged` that holds a changed scenario, it finds the accepted requirement with the
  same identifier and shows the two paragraph by paragraph and scenario by scenario, in human form and
  with `--json`.
- **It uses the comparison `sync` and archival already make.** The accepted requirement is the one
  `see-a-modification-the-merge-did-not-take`'s comparison pairs it with, and a requirement is shown
  as differing exactly where `sync` reports its content as not taken. The view adds what that
  comparison left out on purpose: which paragraphs differ, and which words.
- **Changed words are marked inside a paragraph.** Paragraphs both sides hold are counted and not
  reprinted. A paragraph the change edits is shown beside the accepted paragraph it pairs with, and
  the words only one side holds are marked on that side. A paragraph that shares fewer than half its
  words with any partner is shown whole on each side.
- **An author's `<ins>` and `<del>` are shown as the author marked them, where they account for the
  whole difference.** Where they do not, the view marks words by its own comparison and says the
  author's marks leave part of the difference unmarked.
- **Scenarios are paired by identifier**, which `state-a-modified-requirement-whole` has every
  carried-forward, altered, or withdrawn scenario keep. Added, withdrawn, and left-out scenarios are
  each shown as such.
- **Requirements the change removes or carries forward are listed, and not compared.**
- **Nothing in the view reads as merge text.** Every text is labelled with the document that holds
  it: "the change states" or "the accepted specification holds". No text combines the two, and the
  human form is not a patch.
- **It reads and never writes.**
- **`show <change>` without `--diff` refuses**, names `--diff`, and exits 2. No other view of a
  change is defined yet.
- **An archived change is refused**, and the refusal says why. Compared with today's accepted text,
  an archived change shows nothing, or shows a later change's edit reversed (evidence, section 6).
- **The instructions point reviewers at the view**: `verify.md` where a change already has a
  specification, and `archive.md` before the merge, where a difference the change did not make is
  another change's later edit that the restatement would revert.
- **Unchanged:**
  - the document contract. `CONTRACT_VERSION` does not move, because no unit, field, attribute, or
    projection rule is added.
  - `sync`, archival, and their payloads. The view reuses their comparison and changes none of its
    results.
  - validation, the exit-code contract, and every other command.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `toolchain-packaging`:
  - **Added: "The command surface shows a modified requirement beside the accepted one".** It is
    added rather than folded into "The command surface reports whether a change reached the accepted
    contract". That requirement's report says where content differs, "SHALL NOT propose the text of
    a merge", and, as `see-a-modification-the-merge-did-not-take` leaves it, carries no text of
    either side. A view that carries both texts is a different output with a different name. Folded
    in, it would put a text-carrying view under a requirement whose report is defined by carrying
    none. The new requirement ties the two together instead: it takes "differs" from that
    requirement's comparison, from the same source.
  - **Modified: "The command surface names its operations as OpenSpec names them".** `show` joins the
    fixed spellings, and `--diff` joins the selectors. The scenarios "An operation OpenSpec also
    provides" and "The selectors carry OpenSpec's spellings" gain `show` and `--diff`. Every other
    paragraph and scenario is today's accepted text, unchanged. `resolve-a-reference-across-its-change`
    modifies this requirement first, in its scenario "Validating one document", so task 1.0 refreshes
    this block from the accepted text once that change archives.
  - **Checked and unchanged:** "Exit codes distinguish outcomes", "Output is available in human and
    machine forms", and "Specifications can live in a repository other than the one being worked
    on". The view follows each as it stands, and the new requirement cites each. "Archival assesses
    the merge and never performs it" is unchanged too: its prohibition on merge text is about
    archival, and the new requirement states its own.

## Impact

- `crates/htmlspec-cli/src/main.rs`: `Command::Show`, taking the change as a positional argument as
  OpenSpec's `show` does, with a `--diff` flag and help text. A missing `--diff` is reported by name,
  as `Command::Run` reports a missing `--as`.
- `crates/htmlspec-cli/src/show.rs`, new: the view's types, the alignment, the pairing, the human and
  structured forms, and the command. The core is pure, and files are read only at its edge.
- `crates/htmlspec-cli/src/commands.rs`:
  - The index of accepted requirements is taken out of `merge_state` into one function that `sync`,
    archival, and the view all call.
  - `see-a-modification-the-merge-did-not-take`'s content comparison is made callable by the view.
  - The refusals `sync` makes before comparing are shared with the view.
  - `Refusal` gains a variant, so a runbook is refused with the view's own reason.
- `crates/htmlspec-cli/src/project.rs`: `ProjectRoot` answers whether a directory is an archived
  change, by position, beside `is_change_evidence`.
- `crates/htmlspec-parse`: a reader of the `ins` and `del` runs in a unit's statements and scenario
  entries, beside the projection and sharing its walk. It adds no field to the model.
- Tests:
  - unit tests for the alignment and for the view, one per scenario of the new requirement;
  - a parity test with `sync`'s content comparison;
  - a property test that the marks reader agrees with the projection on every mapping fixture;
  - CLI tests in a new `crates/htmlspec-cli/tests/show.rs`;
  - `no_shipped_body_names_an_absent_command`, which covers the new instruction text.
- `crates/htmlspec-assets/instructions/verify.md` and `archive.md`, with their installed copies
  regenerated.
- **Not in this change**, and recorded so it is not assumed to follow:
  - **Comparing an archived change with the requirement as it stood before it.** That needs an order
    over archived changes' specifications. The program orders only runbook snapshots, by recorded
    version (evidence, section 5). The text the accepted specification held then is recorded only
    in git. So none of the staged migration's 24 archived changes is served by this view.
  - **A default view for `show <change>`**, such as OpenSpec's printing of the proposal.
  - **`show` for an accepted specification**, which OpenSpec's `show` also offers. An accepted
    specification is a page a browser renders.
  - **Comparing the text of removed or carried-forward requirements.** Whether they reached the
    contract is `sync`'s report, and `sync` does not compare their text either.
  - **Notes and the `renamed-from` paragraph.** The comparison does not treat them as content, so
    the view does not show them.
  - **Presentational markup.** The view compares projected text, so a change to emphasis, code
    spans, or links alone shows nothing.
  - **A pointer from `sync.md` to the view.** The merge reads the change's specification for
    meaning and works through `sync`'s own worklist. A view that shows both texts side by side
    invites the wholesale copy that `sync.md` forbids.
  - **Colour in the human form.** The marks are characters, so they survive a pipe and a log.
  - **A validation rule.** Nothing here runs in `validate`.
