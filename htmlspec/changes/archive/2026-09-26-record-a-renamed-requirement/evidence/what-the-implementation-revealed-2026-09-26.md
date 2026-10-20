# What the implementation revealed

Task 9.3. The findings in the order they changed something, with where each is recorded in full.
**Two of them changed the design, and the contributor decided both.** The rest changed the code or
the instructions within the design's intent.

## The two that changed the design

1. **The projected `title` carries the identifier span** (task 1.2). The design compared renames
   against a title "which excludes the `hs-id` span", and the title does not exclude it. Asked
   whether to change projection section 3.1 or to derive a name, the contributor chose a *heading
   name*: the title with the unit's own identifier removed from its end. It is derived from the
   model and is not a new field. Design decision 3, projection section 4.1.2, `HeadingName` in
   `htmlspec-parse`. `evidence/the-premises-2026-09-26.md`.
2. **A minor bump would have invalidated every document** (task 2.3). `schemaVersion` was a `const`
   and the validator matched it exactly, so `0.2.0` alone would have rejected every document
   declaring `0.1.0`. Asked, the contributor chose `0.2.0` with both versions accepted.
   `schemaVersion` is now a closed enumeration, and new documents declare the newest. Design
   decision 8.

## What task 1.3 found that the design did not anticipate

Five of OpenSpec 1.13.1's rename rules have no counterpart here: a duplicate `TO:`, the near-miss
"typo'd header" guards on both sides, "target already exists", and a chain of renames inside one
delta. One reason covers all five. OpenSpec finds a rename's requirement by heading, and html-spec
finds it by identifier, so `renamed-from` is never a lookup key. Design decision 9 names them as
deliberate differences. The chain case also reached `migrate.md`, which collapses a chain inside one
source delta to its first `FROM:`.

## What changed within the design

- **One misplaced rename would have been two diagnostics.** Projection section 4.0 already faulted
  a `data-field` naming no field of its unit, so a scenario carrying a `renamed-from` paragraph
  would also have drawn `rename-outside-requirement`. Section 4.0 now makes an exception, and the
  projection records such a paragraph beside the model rather than faulting it.
- **The projection discarded an empty paragraph before reading its `data-field`**, so
  `rename-empty` could never have fired. `renamed-from` paragraphs are now kept as a
  `Vec<RenamedFrom>` with an explicit `Empty` case.
- **A later change restating a renamed heading** (`2026-08-31-sync-a-change-into-the-accepted-contract`
  restates C after the B → C rename) is outside the chain the design describes. It resolves through
  the chain, or the one obligation would carry a second identifier in exactly that document.
  `migrate.md` says so. `evidence/the-implementation-2026-09-26.md`, section 7.2.
- **`plan.rs`'s rename test asserted the rule this change replaces** ("one obligation, three
  identifiers"), and `migrate.rs` still called a rename an open question. Both now state the new
  rule, and the test uses this repository's real chain.
- **The lockfile had no place for a revision left in place.** Keeping the previous stylesheets for
  the documents that name them would have left committed bytes that nothing pins, so the lockfile
  gained a `retained` list and a test for it.
- **The fixture generator's base would have moved every committed input to `0.2.0`.** It now
  declares `0.1.0` explicitly, keeping 52 inputs byte-identical and the suite a standing check that
  `0.1.0` is still accepted. The generator also deleted four hand-written fixtures, two more than the
  trap was known to reach, and all four were restored.
- **A test hard-coded the stylesheet's address** (`promote.rs`), and went stale when task 8.1 moved
  it. The mutation run is what exposed it. `evidence/mutation-testing-2026-09-26.md`.
- **A hard-coded `0.1.0` in an asset test** and a `README.md` sentence both stated the old version.
  Both now read or state `0.2.0`.

## What is left, and whose it is

- **A document declaring `0.1.0` that carries `renamed-from` validates.** The accepted vocabulary is
  the newest version's. A version-aware rule was not added. Design decision 8 states it as a known
  limit, not an oversight.
- **Nothing checks that an accepted specification left out `renamed-from` after a merge.** Design
  decision 5 and its risk, unchanged.
- **`scripts/check` needs `cargo` and `python3` in one environment, and the development shell
  supplies only `cargo`.** It ran here with the shell's `cargo` put on the system `PATH`.
- **The paused migration of this repository** resumes against this contract, and its plan's two
  open rename notes become `renamed-from` paragraphs under
  `req-a-single-command-exposes-validate-format-and-init`. That migration is its own work, and
  task 7.2 checked the identifier it will use.
