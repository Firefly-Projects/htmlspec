# What this change builds on, read as built

**Date:** 2026-10-05
**Task:** 1.1
**Tree:** `main` at `e112067`, after `resolve-a-reference-across-its-change` was archived (PR #44)

Each check reads the code or the instruction as it stands, and says whether the design matched it.
Where it did not, `design.md` was corrected before any test was written.

## `Placement` and `Subject::placement()`

`crates/htmlspec-validate/src/references.rs` declares `Placement` with the five variants design
decisions 4 and 5 name: `Loose`, `ChangeDocument`, `ChangeSpecification`, `AcceptedSpecification`,
and `RunbookDocument`. `ChangeSpecification` is documented as "A change's `specification.html`,
directly within the change's directory", active or archived.

`Subject` in `crates/htmlspec-validate/src/engine.rs` holds `pub scope: &'a Scope`, and
`Subject::placement()` returns `self.scope.placement()`. So the rule reads the placement as the design
says, through the scope rather than a field of its own.

A suite outside the crate can build a scope at any placement with `Scope::gather(placement, |_|
Declared::default())`, where `Declared` is `htmlspec_validate::references::Declared`. It is not
re-exported at the crate root. `conformance/tests/references.rs` already builds scopes this way, so
task 2.6's suite needs nothing new from the public API.

Every place the command line validates goes through `validate_source` in
`crates/htmlspec-cli/src/commands.rs`, which derives the scope from the path: `validate` (line 148),
`archive` (1449), `sync` (2533), promotion's `conformance` (3232), and `new document`'s check of what
it writes (894). **As designed.**

## The contract version

`crates/htmlspec-schema/src/lib.rs` line 70: `pub const CONTRACT_VERSION: &str = "0.3.0";`. Only
`reject-change-markup-inside-a-unit` moved it in this set. So this change moves it to `0.4.0`, as the
design's Migration Plan expects. **As designed.**

## The instructions tasks 7.1 and 7.2 edit

- `crates/htmlspec-assets/instructions/sync.md`, "## Merging a modified requirement" (line 142). Its
  six numbered steps compare and apply statements and scenarios as the comparison reports them, and
  its step 2 says "A statement paragraph carries no change state, so the change's paragraphs are the
  requirement's paragraphs: one it leaves out was deleted". The new sentence belongs after the
  numbered steps, where the section says what the merge writes.
- `crates/htmlspec-assets/instructions/archive.md`, step 2, "**A requirement the change modified.**"
  (line 187), with its example and five bullets. **One bullet needs reconciling, not only an
  addition:** "The statements are the requirement's statements once merged. A statement paragraph
  carries no change state, so leave out a paragraph the change deleted". Once a paragraph deleted
  whole may be kept in `<del><p>…</p></del>`, that sentence would forbid what task 7.1 permits.
  Task 7.1 rewords it to say the deleted paragraph is left out or wrapped in `del`, and either way
  supplies no statement.
- `archive.md` step 5's bullet "A requirement the change modified takes the statements and the
  scenarios the change states" (line 325) is where task 7.2's pointer to `sync.md` goes.

**As designed**, apart from the bullet above, which the design did not foresee.

## Whether the comparison reads projected text

`see-a-modification-the-merge-did-not-take` is built and archived. `merge_state` projects every
accepted specification with `htmlspec_parse::project`, and `content_difference` compares
`normalized_all(&requirement.statements)` with the accepted requirement's, and each scenario as a
`ScenarioText`: its projected title, `given`, `when`, and `then`, each normalized. So once a `del`'s
content leaves every field, the comparison reads the current text with no change of its own.
**As designed**, and design decision 7 needs no MODIFIED block of toolchain-packaging. The design's
Context and decision 7, which still called the comparison planned, now describe it as built.

## The stylesheet revision task 6.1 retires

`assets/vendor.lock.json` records `htmlspec.css` under `own` at
`aac8967609f80d9f90b9a92adff22e495a5d4863618b5b12a3cf89c9b08453d1`, and the print stylesheet at
`04ac9a24fc436a87…`. `record-a-partial-verification` is not yet implemented, so `aac8967609f80d9f` is
still the revision to retire. **As designed.**

## The rule this change relaxes, as its review left it

Not one of the task's listed checks, but design decision 5 is written against it. `reject-change-markup-inside-a-unit`'s
rule, in `crates/htmlspec-validate/src/rules/change_markup.rs`, gained two cases in its review (PR
#40), which archived it:

- `Reach::Itself`: an element whose own `class` names a unit is that unit, and is reported at itself
  with a message telling the author to put the unit on an element that is not change markup and to
  mark its change state.
- An element named `ins` or `del` outside the HTML namespace, inside SVG or MathML, is not change
  markup.

The design's `Refusal` had no variant for the first, and its `MarkupContext` did not say whether the
element is a unit. Both are corrected in decision 5: `Refusal::IsAUnit` comes first, and the context
records it. Decision 3's conditions and its table gained the case, and the re-based requirement
(task 1.0) excludes it from the permission in words. The namespace test needs no change: an element
it rejects is never judged.

A `renamed-from` paragraph is recognized in the projection by `data-field="renamed-from"`, the
constant `RENAMED_FROM_FIELD` in `crates/htmlspec-parse`, which the rule can name rather than spell.
