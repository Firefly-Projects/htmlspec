# What the implementation revealed

**Date:** 2026-10-01
**Task:** 5.4

- **Task 4.3's expected result was wrong. The bump rewrote twelve committed fixtures.** The task
  said the generator's base keeps declaring `0.1.0`, so no committed fixture would change. The base
  does, but the twelve `schema/rename-*` fixtures declare the newest version through
  `declaring_the_current_version`, and the newest version is read from the template. After the
  bump, the generator moved each of them from `0.2.0` to `0.3.0`: the `htmlspec-version` meta
  element and the masthead's Contract line, 24 lines in all and nothing else. That is the function
  doing what its comment says ("Redeclares the frozen base at the newest contract version"), so the
  regenerated fixtures are kept. No stage compares the committed fixtures with the generator's
  output, so keeping the old bytes would have left the next run to rewrite them anyway. The
  conformance suite passes with them. **Every later change that moves the contract will see the
  same.** `record-a-task-done-differently`, `mark-the-words-a-modification-changed`,
  `record-a-partial-verification`, and `record-how-a-risk-was-settled` each bump the contract, and
  the last two add fixtures through the same function, so each bump rewrites a larger set.
- **Two later changes name the contract this change replaces.** The Migration Plans of
  `state-a-modified-requirement-whole` and `resolve-a-reference-across-its-change` say the contract
  stays at `0.2.0`. Each still moves nothing, so what stays is now `0.3.0`, and their re-measure
  tasks are where the wording follows.
- **The generator needs a feature to run.** `cargo run -p htmlspec-conformance --bin
  generate_fixtures`, as tasks 2.3 and 2.4 give it, fails: the binary has `required-features =
  ["generate-fixtures"]`, so it stays out of a default build. The generator's own header named the
  same incomplete command, and now names `--features generate-fixtures`.
- **The hand-authored table checks itself in both directions.** Beside keeping the four fixtures
  from the stale sweep, the generator now refuses a name that is in both tables, which would write
  over a committed fixture. It also refuses a listed fixture whose `input.html` or `expected.json`
  is missing, so an entry cannot outlive the fixture it keeps. The second file was added at review. With the table in place, a run with no other edit left
  `fixtures/conformance/` untouched: 64 generated fixtures and the four kept.
- **The first wording of the fix was wrong for an added unit.** It told the author to mark the
  unit `data-change="modified"` in every case. Task 5.1's measurement showed the case against it:
  the verification there is `added`, and so is the base requirement in the invalid fixture. A unit
  a change adds has no earlier text, so its markup is removed with nothing else to do. The message
  now says to write the text as it now reads, and states when a unit is marked `modified` instead of
  ordering it. Telling the two cases apart in the rule would mean comparing the unit's change state
  with the literal `added`, which `schema_rust_split.rs` forbids in rule code, and the sentence
  serves both cases.
- **The quotation of a wrapped unit includes its identifier.** A `del` around a requirement is
  quoted as `<del>No other spelling req-no-other-spelling No other spelling SHALL …</del>`, because
  the template puts the identifier in the heading's `hs-id` span and the quote is the element's own
  text. The diagnostic is located at the unit, so the repetition costs a reader nothing.
- **The quotation is not the projection's normalized text in one case.** The rule quotes the
  element's text content with whitespace normalized, as projection section 6.1 normalizes a field.
  Unlike the projection's `collect_text`, it does not skip `script`, `style`, `template`, or
  `noscript`, because that function is private to the parse crate. A change element holding one of
  those elements quotes its text where the projection keeps none. The quote only helps an author
  find the element, and the location is the unit either way, so no test pins the difference.
- **Four statements of the version policy gave a reason `0.3.0` made false.** The README, the
  comment on `CONTRACT_VERSION`, the schema's `schemaVersion` description, and the contract
  changelog's policy each said earlier versions stay accepted because a minor bump invalidates no
  document. The README was corrected with the bump. The other three were first left alone, on the
  ground that the `0.x` clause sits beside them, and the review of the pull request showed that each
  still asserted the reason unconditionally. Each now gives the reason that holds: a document
  declaring an earlier version is still an existing document, and only a major bump withdraws a
  version. The policy table's Minor row is unchanged. It states what a minor bump means once the
  vocabulary is committed, and the `0.x` clause directly below it covers this release.
- **Reproducing the measurement took three choices the evidence did not quote:** the template's
  `not-run` outcome, contents links only to the sections the document keeps, and a verification
  naming both requirements in the wrapped case. Each was settled by a diagnostic the evidence
  records. `the-measurement-repeated-2026-10-01.md` lists them, and its appendix holds the script,
  so the measurement no longer depends on reconstruction.
- **Two mutations each have a narrow catcher, as the design expected.** One test catches a
  substring match on the reserved class. No conformance fixture catches a rule that looks only at
  ancestors, and four unit tests do. `mutation-testing-2026-10-01.md`.
- **`scripts/check` passes, but not in one shell.** Inside `nix develop`, its four Python stages,
  `prompt`, `installed`, `release-fixtures`, and `release`, each fail with "tool 'python3' not
  found". That is the same environment fault the proposals for this set of changes met, and not
  this change's. Those four were run outside the shell with the shell's `cargo` on `PATH`, and all
  four passed: 4 prompt cases, 45 installed instruction files, 23 release fixture cases, and the
  release check. The other nine stages passed inside the shell. Both runs, before the release and
  after it at `0.19.0`, gave the same result.
