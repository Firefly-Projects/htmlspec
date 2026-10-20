# What the implementation revealed

**Dates:** 2026-10-05 and 2026-10-06
**Tasks:** 8.4 and 8.5

## 8.4: the checks

- `cargo test --workspace --all-targets`: 1,246 passed, 0 failed.
- `scripts/check`: the first run failed its `format` stage, because the new tests had not been run
  through `rustfmt`. `cargo fmt --all` reflowed ten places in five files and changed no code. After
  it, the nine stages that run inside `nix develop` pass. The four Python stages fail there with
  "tool 'python3' not found", as they have for every change in this set, and pass outside it with the
  same Rust toolchain first on `PATH`:
  - prompt: `all 4 prompt cases passed`
  - installed: `45 installed instruction file(s) match what init writes`
  - release-fixtures: `23 shared fixture case(s) read the same way both readers read them`
  - release: `declared version 0.22.0 has a changelog entry`, because the release is group 9's.

## 8.5: what the work found

- **The rule this change relaxes had grown since the design was written.** `reject-change-markup-inside-a-unit`'s
  review, archived with it, made an element whose own class names a unit a case of its own and left
  SVG and MathML elements named `ins` or `del` alone. The design's `Refusal` had no variant for the
  first. Task 1.1 added `Refusal::IsAUnit`, and the re-based requirement excludes such an element from
  the permission in words (`what-it-builds-on-2026-10-05.md`).

- **The decision's context became a sum type.** The design sketched `MarkupContext` as a set of
  facts. As built it is `IsAUnit`, `EnclosesAUnit`, or `Inside { unit, marked, part }`, with `Marked`
  and `Part` in place of flags, so a context that is a unit and inside one cannot be built, and the
  first two refusals are read off the context rather than tested.

- **Rule 8.6.1 had to cover walks that are not a unit's.** The design put the rule in the walk over a
  unit's own descendants. The document's summary is read from the header's paragraphs, and a figure's
  caption, `pre`, and `math` from the figure's descendants, neither through that walk, so a paragraph
  or a caption wholly inside a `del` would still have projected. The rule is stated as "the content of
  a `del` supplies no field of anything that encloses the `del`", both walks filter what sits inside
  one, and the mapping fixture gained a deleted summary paragraph, a deleted caption before the real
  one, and a whole figure inside a `del`. It also gained `<del class="decision">`, which is that unit
  and projects as one, recording no deletion.

- **The tracked count of change markup was not zero, and the contributor decided what the two
  documents say.** `reject-change-markup-inside-a-unit` had generated two conformance fixtures
  carrying `del` and `ins` since this change was written. Neither verdict changes. One fixture's
  statement now projects as its current text, and its note, which gave the old projection as its
  reason, is reworded (task 1.3).

- **Two old tests pinned wording this change makes wrong, and they were updated to the new
  requirement, not weakened.** One asserted that a loose document is told to mark the unit
  `modified`, which design decision 5 forbids outside a change's specification. The other asserted
  the reason "the withdrawn words are still part of what the requirement states", which rule 8.6.1
  makes false. Each still pins one message for each element (task 4.1).

- **A command-line test first passed for the wrong reason.** `archive --check` compares the declared
  version with the last archived one before it reads the specification, so the first draft of the
  archival test stopped at "the declared version is still 0.0.0" and never read the markup. The test
  now advances the version and asserts that the output names `specification.html` (task 2.8).

- **The fixture generator redeclares twelve fixtures at every contract bump.** The rename fixtures are
  built by `declaring_the_current_version`, and `reject-change-markup-inside-a-unit`'s bump rewrote the
  same twelve. Task 5.3 expected none. Each changed in two lines.

- **The retained stylesheet's reason had to be found, not assumed.** No tracked HTML document names
  `aac8967609f80d9f`. The 109 documents the self-migration has staged do, and keep it once promoted,
  so that is the reason `vendor.lock.json` gives (task 6.1).
  `record-a-partial-verification`'s task 5.1 already says to retire whatever revision `own` records
  when it runs, so it will retire `ebf35a36fe4dc1a7`, and needs no edit.

- **The projection tool in the migration folder links the parser it was built against.** The copy
  built for task 1.2 still kept a deleted word in the statement after the projection changed, which
  task 7.1's check showed. Every measurement after it used a rebuilt copy.

- **The screen-reader check needs a person.** Headless Chrome could not start in the session's
  sandbox, and outside it never wrote its output. Headless Firefox, with scripts turned off in its
  profile, rendered the page, and print was approximated by forcing the print stylesheet's rules on.
  No screen reader can be driven from the session, so `the-presentation-2026-10-05.md` gives the
  VoiceOver procedure. The contributor ran it on 2026-10-07 with Safari, and all four markers were
  announced.

- **The design's claim about a figure was checked against the binary before the changelog stated
  it.** A caption wholly in a `del` reports projection rule 7.2, the schema's required `caption`, and
  `a11y/figure-without-caption`. A Mermaid source wholly in a `del` reports `viz/diagram-empty` and
  `a11y/figure-without-fallback`. A deleted first keyword reports `viz/diagram-unrecognized`.

- **The work was paused once, mid-group, for another change.** On 2026-10-05 the contributor asked for
  a change to the command line's spellings. The work so far was committed on this branch as a work in
  progress, the proposal was written on its own branch (PR #45), and this change resumed where it
  stopped.
