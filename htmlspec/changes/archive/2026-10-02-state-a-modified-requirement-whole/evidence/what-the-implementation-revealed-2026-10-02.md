# What the implementation revealed

**Date:** 2026-10-02
**Tasks:** 4.3 and 4.4

## 4.3: the checks

- `cargo test --workspace --all-targets` inside `nix develop`: 1,132 passed, 0 failed. That is
  one more than `reject-change-markup-inside-a-unit` left, which is the test from task 2.1. The
  fixture from task 2.2 adds no test, because the conformance harness drives every fixture
  directory through the same tests.
- `scripts/check` inside `nix develop`: the first nine stages pass (format, lint, test, doctest,
  self-conformance, self-format, self-runbooks, self-runbooks-format, self-assets). The four
  Python stages fail there with `error: tool 'python3' not found`, as they did for the last change.
  Run outside the shell with the same Rust toolchain first on `PATH`, all four pass:
  - prompt: `all 4 prompt cases passed`
  - installed: `45 installed instruction file(s) match what init writes`
  - release-fixtures: `23 shared fixture case(s) read the same way both readers read them`
  - release: `25 archived change(s) record a version, and 19 version(s) are in the changelog`, and
    `declared version 0.19.0 has a changelog entry`

## 4.4: what the work found

- **The proposal and the design named the wrong contract version.** Both said `CONTRACT_VERSION`
  stays `0.2.0`. They were written before `reject-change-markup-inside-a-unit` archived and moved
  the contract to `0.3.0`. Both now say `0.3.0`. Nothing else in the planning artifacts depended
  on the number.

- **Task 2.2's restore step was no longer needed.** It told the implementer to restore four
  hand-authored fixtures with `git checkout`, because the generator used to delete them.
  `reject-change-markup-inside-a-unit` added the `HAND_AUTHORED` table, and the generator now
  keeps them. Regenerating wrote the new fixture and left every other fixture byte-identical:
  `git status` showed only the new directory.

- **The test written first had a flaw that HEAD could not show.** Test 2.1's first draft looked
  for the requirement's name in the raw body. Both `sync.md` and `archive.md` wrap the name across a
  line, so the test failed against the finished instructions. On HEAD it had failed earlier, at the
  "partial update" assertion, which hid the second assertion's flaw behind the first. Both phrases
  are now matched on text with every whitespace run collapsed, which the test's own comment already
  said was needed. **A test that fails on HEAD proves its first failing assertion, and nothing about
  the ones after it.** The finished instructions were the first input to reach the second assertion.

- **The procedure needed a decision the contract leaves to it.** The requirement says the merging
  agent reports an accepted scenario that is neither stated nor marked `removed`, and "neither keeps
  it nor deletes it". In an accepted specification, leaving a scenario in place keeps it. So
  `sync.md` holds the whole requirement unmerged until the contributor answers, and says the answer
  belongs in the change's specification as the scenario stated `unchanged` or marked `removed`. That
  is the one action that is neither reading. **The review should confirm it.** The review accepted
  it.

- **The first draft of `sync.md` treated the two reports differently, and that was wrong.** For a
  scenario marked `unchanged` whose text differs from the accepted one, the draft left that scenario
  as accepted and merged the rest of the requirement. The design names the usual cause: a change
  written before another change modified the same requirement. That cause can make the change's
  statements stale as well, and a statement carries no change state to show it, so merging the rest
  could revert the other change's statements without anyone deciding to. Step 4 now holds the whole
  requirement unmerged as well. This was settled after the mutation run. None of the four mutations
  touches that sentence, so the run's results stand. The instructions test, the installed copies,
  and `scripts/check-installed` were rerun after it.

- **Five of OpenSpec's generated files still carry the partial rule, not two.** The proposal's "Not in
  this change" names `.claude/skills/openspec-sync-specs/SKILL.md` and `.claude/commands/opsx/sync.md`.
  The same text is in `.codex/skills/openspec-sync-specs/SKILL.md`, `.pi/prompts/opsx-sync.md`, and
  `.pi/skills/openspec-sync-specs/SKILL.md`. Three carry `generatedBy: "1.7.0"`, and the Claude Code
  command and the Pi prompt carry no stamp. All five arrived in one commit, `2989654`, and are
  unchanged since. This record first said all five carry the stamp, and the review found two that do
  not. The proposal's reason covers all five: they govern this repository's OpenSpec-form work, not
  the product. **The review deferred them to promotion**, which removes the OpenSpec form.

- **A passage beside the new section raises a question this change does not answer.** `sync.md`
  "Merging one requirement", step 3, tells an agent to "Decide whether it is a new requirement or an
  addition to one already accepted. A change often sharpens a requirement that exists rather than
  adding a fresh one." The first bullet of "What to report" asks "whether you added a requirement or
  folded a scenario into one already there". That step works on identifiers in `absent`, so the
  requirement it folds in carries an identifier no accepted specification will ever hold. `sync` and
  archival then go on reporting it as absent. Under the new rule, a change that sharpens an accepted
  requirement states it as `modified`, with the accepted identifier, and the new section merges it.
  Step 3 describes a change's specification that stated a modification as an addition, which is the
  same kind of defect as a scenario left out. Neither passage is edited here, because no task names
  them and this change's subject is the requirement a change marks `modified`. **Offered to the
  review, and otherwise to `see-a-modification-the-merge-did-not-take`**, whose comparison is where
  an absent identifier is reported. **The review took it into this change.** Task 6.1 records the
  edit.

- **The new fixture is the only conformance fixture that would catch a rule against this shape.**
  Mutation 4, an error reported for a `removed` scenario inside a `modified` requirement, failed
  `valid-modified-requirement-stated-whole` and no other fixture
  (`mutation-testing-2026-10-02.md`). The fixture pins acceptance only. A `review/*` warning against
  the same shape would pass, as it would for every valid fixture.

- **The first measurement shows a summary as if the script printed it.** Section 3 of
  `the-measurement-2026-09-30.md` shows `count.py`'s output as six summary lines. The script prints one
  JSON row per block, and the summary was derived from that JSON. The repeated measurement says so.
  The counts themselves reproduce exactly.
