## Why

A `tasks` document shipped with the template's own words as its masthead summary — *"Replace this
paragraph with what this work delivers."* It passed `htmlspec validate` with **zero diagnostics**,
passed `format --check`, and would have passed archival. A human reader found it by opening the
file and noticing that the other two changes in the same directory had real summaries.

This is not a one-off. The report naming it says **"I have seen it happen several times."**

The gap is exact. `review/example-content` reports a unit whose **identifier** is one a template
writes — `req-example`, `task-dropped-example`. `htmlspec_assets::example_identifiers` builds that
list by scanning the templates for `id="…"` attributes, so it sees identifiers and nothing else.
Every template also writes a masthead paragraph, and that paragraph carries no identifier:

```
$ htmlspec new document tasks t.html --subject "A change"
$ # replace the two example task units, leave the masthead alone
$ htmlspec validate t.html
  no diagnostics: the document is accepted
```

**The summary is not trapped in markup — it is in the model.** The projection specification, section
9.1.1, projects `summary` from every `p` element inside the document's `header`. So this is
checkable, and the reason it goes unchecked is that nothing compares that projected text against
what the templates write.

The cost is worse than an unhelpful paragraph. `document-schema` states what `summary` is for: *"What
the document is for, in its own prose. Projected from the paragraphs of the document header, so that
the argument a change rests on is addressable in the model rather than trapped in presentation."* A
document whose summary is the placeholder has an empty answer where the model promises the argument
— and every consumer reading `summary` gets instructions addressed to the author instead.

This is the same shape this project has now closed three times: a rule whose subject is real and
whose check does not reach it. `project-bootstrap` states the general form — a rule nothing checks
is not a rule.

## What Changes

- **Extend example-content detection from identifiers to prose.** `htmlspec-assets` gains the
  placeholder paragraphs alongside the placeholder identifiers, read out of the templates rather
  than listed beside them, for the reason `example_identifiers` already gives: a list restates a
  fact the templates already make and goes stale the first time one changes.
- **Report a document whose summary is a template's placeholder.** `review/example-content` gains
  this case, at the same severity and for the same reason — `htmlspec new document` writes the
  template and the contract requires that result to be valid, so an error there would report a
  defect the program itself introduced.
- **Refuse it at archival**, where `review/example-content` already escalates. A change archived
  with a placeholder summary puts "Replace this paragraph" into the project's permanent record.
- **Cover every artifact kind.** All four templates write a masthead placeholder and all four are
  invisible today. A rule covering only `specification` would leave the kind this was reported on
  unchecked.

**Not in scope:** judging whether a real summary is a *good* one. That is reading, and
`artifact-kinds` reserves it for an agent. This detects text the program itself wrote.

## Capabilities

**New Capabilities:** none.

**Modified Capabilities:**

- `contract-authority` — its requirement *"Generated example content is non-normative and its
  survival is reported"* is where this lives, not `document-validation`. I checked: the rule's
  contract is stated there, in the capability whose subject is what carries authority.

**That requirement currently says the opposite of what this change needs**, and the sentence is
deliberate rather than incidental:

> The check SHALL be on the identifier rather than on the example's text. Text is edited while an
> identifier survives, so a half-replaced example carries new prose under the example's
> identifier — and that is the likely case rather than an edge one.

That reasoning is sound and this change does not overturn it. **A masthead placeholder is the case
it does not cover**, because the summary has no identifier to survive — so "check the identifier"
is not a narrower rule there, it is no rule at all. The modification adds the summary as a second
site with its own reason, and leaves the identifier rule standing as the primary one for units.

## Impact

- `crates/htmlspec-assets/src/lib.rs` — the placeholder prose beside `example_identifiers`, read
  out of the templates by the same discipline.
- `crates/htmlspec-validate/src/rules/review.rs` — `example_content_is_replaced` gains the summary
  case.
- `crates/htmlspec-cli/src/commands.rs` — `example_content_in`, which archival and `sync` share.
- `fixtures/conformance/` — a fixture pinning the case, which is what makes this a contract rather
  than an implementation detail.
- No change to the schema, the projection, or the document model. The text this reads is already
  projected.

**All 50 existing fixtures carry the placeholder prose, and none of them breaks.** I checked rather
than assumed: `valid-minimal` already reports `review/example-content` seven times while its
`expected.json` declares no rules, and it passes. The reason is that
`every_fixture_reports_exactly_the_rules_it_declares` asserts each declared rule *is* reported and
does not assert that nothing else is — so a new warning is invisible to the suite, and acceptance is
unaffected because `review/*` never gates.

**That is worth recording as a second finding rather than relief.** A test named "exactly" that
checks one direction is the same shape as the defect this change closes: a check whose name claims
more than it reaches. It is not this change's subject, and the change that tightens it needs to
decide what the 50 fixtures should declare — so it is named here and left open.
