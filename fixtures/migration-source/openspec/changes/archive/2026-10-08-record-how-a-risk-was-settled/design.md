## Context

See proposal.md for why. The approach depends on this shape of the current system:

- **A risk's shape lives in the schema and the projection, not in a requirement.** The schema's
  `risk` definition requires `unit`, `id`, `change`, `title`, `statement`, and `mitigation`.
  Projection section 4.7 reads: "`statement` projects from the single unmarked paragraph, and
  `mitigation` from a `data-field="mitigation"` paragraph. A risk missing either one is invalid."
  document-schema names `risk` only in its class vocabulary, its prefix table, its reference
  rule, and the scenario that lists the units a `renamed-from` paragraph may not sit in.
- **Every closed per-unit value is an attribute with a row in projection section 2.** The rows are
  `data-change` (default `unchanged`), `data-state` (task only), `data-status` (decision only), and
  `data-outcome` (verification only). Section 2.2 ignores every other `data-*` attribute. A
  per-unit attribute on the wrong unit is ignored silently too: see the evidence, variant E.
- **A conditional requirement is a validator rule.** The schema's `task` description says it "keeps
  to a conservative subset with no conditional composition", and that subset is what lets `typify`
  generate types from it. So `rules/relations.rs` owns `schema/missing-reason` (a dropped task
  states why) and `schema/missing-successor` (a superseded decision names its replacement). Each
  finds its enumerated value in the schema by a stem, through `status_named`, and
  `every_conditional_rule_finds_its_state` fails if a stem stops matching.
- **The model is permissive on purpose.** `model.rs` holds enumerated values as `String`, so an
  unrecognized value survives projection and the validator reports it as `schema/*`. A defaulted
  attribute is filled by the projection from a constant: `DEFAULT_CHANGE_STATE`, which
  `the_default_change_state_matches_the_schema` ties to the schema's `changeState.default`.
- **The formatter writes the contract's attributes first.** `LEADING` in
  `crates/htmlspec-format/src/attributes.rs` is `class`, `id`, `data-change`, `data-state`,
  `data-status`, `data-outcome`, `data-superseded-by`, `data-field`, `data-refs`. Every other
  attribute sorts after them, alphabetically.
- **The conformance base is the specification template, frozen.** `generate_fixtures.rs` renders
  the template, then rewrites its asset addresses and its declared version back to the committed
  form. The reason is in its own header: re-rendering "would also rewrite all 37 committed inputs,
  whose exact bytes are what each fixture pins." The template's example risk, with its sentence
  "A risk with no mitigation is a worry.", appears in 68 committed fixture inputs, 64 when this
  was written and four more from the changes earlier in this set.
- **The stylesheet shows one badge per heading.** Change state, task state, decision status, and
  verification outcome all write the heading's `::after`, with equal specificity. So a decision
  shows its status in place of its change state.

## Goals / Non-Goals

**Goals:**

- A risk states whether it was mitigated or accepted, and the model carries the answer.
- An accepted risk is valid with no mitigation. A mitigated risk is held to its mitigation, as
  every risk is now.
- Every document that validates today still validates, with the same meaning.

**Non-Goals:**

- **Judging a response.** The validator checks that a mitigated risk states a mitigation. It
  cannot check that an accepted risk ought to be borne, just as it cannot check that a decision
  ought to be accepted.
- **A history of a risk's response.** `data-change` already records what a change did to the
  unit.
- **Rendering the response.** See decision 9.
- **Migrating the archive's risks.** That is `migrate-into-the-extended-vocabulary`.

## Decisions

### 1. Two values, and an absent response is `mitigated`

`data-response` takes `mitigated` or `accepted`. A risk with no attribute is `mitigated`.

The default is what every existing risk already is. All 83 risk units in tracked HTML and all 11 in
the staged migration carry a mitigation (evidence, section 3). With this default, no document
changes meaning or validity, which is what a minor bump promises.

**Considered: a third value, `partial`.** Rejected. No settlement word in the archive says "partly"
(evidence, section 4). The two nearest are "Partially mitigated by the visible-source fallback" and
the runbook's "Partly held down and honestly not fully". Both are mitigations that say how far they
reach, and stating that reach is what a mitigation paragraph is for. "Mitigated, but not fully" is
true of nearly every mitigation. A third value would ask every author to judge a boundary that
nothing reads.

**Considered: `disproved` or `retired`.** The reader of `conform-generated-frontmatter-names`
describes a risk "disproved by task 4". The reader of `pass-the-description-through` describes one
"retired for one of the two unconfirmed tools". Rejected. A risk that a check disproved is mitigated
by that check, and its mitigation paragraph names the check. A risk that a later change retires is
`modified` or `removed` in that change, which `data-change` already expresses.

**Considered: no default, so every risk declares one.** Rejected. It would invalidate all 83 risks
in this repository and all 11 staged, to record a fact each of them already states by carrying a
mitigation.

**Considered: infer the response from whether a mitigation paragraph is present.** Rejected by
the measurement. Five archived risks are accepted and name a partial measure, and one of them is
the only risk the migration drafted as a unit. Its reader said it directly: "'accepted' cannot be
carried as 'has no mitigation paragraph'".

### 2. An accepted risk may carry a mitigation, and needs no reason field

An `accepted` risk is valid with or without a mitigation paragraph. A paragraph it carries projects
into `mitigation`, as it does now, and states the part of the risk that something does hold down.

The precedent is a decision's status. document-schema's "A decision records its outcome and
rationale" requires something extra of only one status: a `superseded` decision must name its
replacement, because that value points at something the reader needs. `accepted` needs nothing
beyond what every decision carries. The same split holds for a risk. `mitigated` points at a
measure, so the measure is required. `accepted` is complete in itself.

**Considered: forbid a mitigation on an accepted risk.** Rejected. The five measured risks that are
accepted with a partial measure could not be written, and one of them is the migration's only
drafted risk unit.

**Considered: require a `data-field="reason"` paragraph on an accepted risk**, as a dropped task
requires one. Rejected, for three reasons:

- Every accepted risk in the archive says why right beside the word, or points to the subsection
  that does. Two examples: "Accepted: rewording is cheap, and the alternative direction of failure
  is…", and "Accepted, matching `specs/` and `archive/`." A required separate paragraph would split
  those sentences. That is the shape that already keeps most of the migration's risks as prose.
- A presence check cannot tell a reason from the word "Accepted." An author with no reason would
  write the word, and the rule would pass.
- A task's reason carries something its title cannot: the title says what the work was, and only
  the reason says why it stopped. A risk's statement already says how the change can fail. The
  judgment that the failure is tolerable belongs beside it, in the statement or in a note.

**Considered: an optional `reason` field.** Rejected. It adds vocabulary that nothing reads, and
`data-field="note"` already carries prose that is not a field.

**The contributor should confirm this decision.** It is the one place where this change chooses less
structure than a precedent in the contract, the dropped task, would suggest.

### 3. The response is an attribute, `data-response`, not a field paragraph

`record-a-renamed-requirement` chose a paragraph over a `data-renamed-from` attribute, because an
attribute is invisible in the rendered page. That reasoning was about free text: a previous heading
is prose a reader has to see. A response is a closed value, and every closed per-unit value in
this contract is an attribute: `data-state`, `data-status`, and `data-outcome`. A paragraph would
carry a closed value as free text, and the projection would have to normalize and match it.

**The attribute is `data-response`, and the field is `response`.** "Response" is the usual
project-management term for how a team answers a risk, whether by accepting, mitigating, avoiding,
or transferring it. This contract takes two of those answers. Checked with grep on 2026-10-01, the
word names nothing in `openspec/specs`, `crates/htmlspec-schema`,
`crates/htmlspec-assets/instructions`, or `assets/template-*.html`. Elsewhere in `crates/`, it
means what a command answers. `crates/htmlspec-cli/src/preflight/host.rs` uses it for a scripted
test runner's responses and the Nix evaluator's response shape, and one comment in
`crates/htmlspec-cli/src/exit.rs` uses it in passing. Neither is a domain term. The vendored
third-party bundles under `assets/sha256-*/` use the word in their own code.

**Considered: `data-disposition`, with a `disposition` field.** This design used that name until the
contributor rejected it on 2026-10-01. In project-migration, "disposition" already names the fate
of a migration plan entry: converted, folded, carried, or dropped, written as
`<p data-field="note">disposition: converted</p>` in a plan task. The migration instructions are
exactly where both meanings meet. `migrate-into-the-extended-vocabulary` had to write "a risk's
`data-disposition`" at every mention to keep them apart. The evidence's scratch documents were
written under the old name, and its note of 2026-10-01 says so.

### 4. The model always carries the response, as a string

`Risk` gains `response: String`, always serialized. The projection fills it from
`data-response`, trimmed per section 2.3. When the attribute is absent, it fills it from
`DEFAULT_RISK_RESPONSE`, which is `"mitigated"`. A test asserts that the constant equals the
schema's `response.default`, as `the_default_change_state_matches_the_schema` does for `change`.
The schema requires `response`, as it requires `change`. The projection always supplies it, so a
consumer never has to apply the default itself.

**Considered: leave `response` out of the model when the HTML declares none.** Rejected. A
consumer would have to restate the default, which document-schema's "The element contract is
published as a machine-readable schema" says tools do not do. The model would also spell one fact
two ways.

**Considered: a Rust enum for the value.** Rejected, for the reason the header of `model.rs` gives.
An enum would reject `data-response="partial"` inside the projection, and the author would lose
the `schema/invalid` diagnostic that names the permitted values.

### 5. A mitigated risk without a mitigation is `schema/missing-mitigation`

The schema drops `mitigation` from the risk's `required`. A new rule in `rules/relations.rs`,
`mitigated_risks_state_a_mitigation`, reports each risk whose response is `mitigated` and whose
`mitigation` is absent. It finds the value `mitigated` with
`status_named("mitigat", "definitions/risk/properties/response")`, and
`every_conditional_rule_finds_its_state` gains that stem. `mitigated` joins the schema-owned
literals in `tests/schema_rust_split.rs`. `accepted` does not, because it is also a decision status
and a word this project's prose uses constantly.

The message names both remedies. It also says that a risk that declares no response is `mitigated`,
because the author most likely to meet this rule wrote the risk before the attribute existed.

A risk with an unrecognized response gets `schema/invalid` from the enumeration, and nothing from
this rule. One defect gets one diagnostic.

**Considered: JSON Schema `if`/`then`.** Rejected. The schema keeps to a subset `typify` can
generate from, and both existing conditional requirements are validator rules for that reason.

**The rule identifier changes for a document that is already invalid.** Variant A in the evidence
fails today with `schema/invalid`, "mitigation is a required property". After this change it fails
with `schema/missing-mitigation`. document-validation's "Diagnostics are actionable and
machine-readable" asks that "the same violation" keep its identifier across versions. The violation
is not quite the same. The old rule was that every risk states a mitigation, and that rule no longer
exists. The `0.2.0` contract made the same kind of move: a `renamed-from` paragraph outside a
requirement stopped being a section 4.0 fault and got a rule of its own. The CHANGELOG entry names
this move, as the `0.2.0` entry named that one.

### 6. A response on another unit is ignored

Section 2's new row reads `data-response` (risk only), like `data-state` (task only). On any
other unit it projects nowhere. Measured today, `data-state` and `data-outcome` on a decision are
ignored with no diagnostic (evidence, variant E).

**Considered: an error for a `data-response` outside a risk**, as a `renamed-from` paragraph
outside a requirement is an error. Rejected here, because the same gap exists for all four per-unit
attributes, and a rule for one would leave the other three. The proposal records it under "Not in
this change".

### 7. The formatter writes the response with the other state attributes

`data-response` joins `LEADING` after `data-outcome`. `LEADING` holds "the ones the contract
reads", and after this change the contract reads this one. No existing document carries the
attribute, so no formatted file moves.

### 8. The template shows the attribute, and the conformance base keeps its old risk

The specification template's example risk becomes:

```html
<section class="risk" id="risk-example" data-change="added" data-response="mitigated">
  <h3>The example risk <span class="hs-id">risk-example</span></h3>
  <p>State what could go wrong, in one unmarked paragraph.</p>
  <p data-field="mitigation">State what holds it down. A risk this change bears as it stands declares <code>data-response="accepted"</code> instead, needs no paragraph like this one, and says in its statement why it is borne.</p>
</section>
```

The attribute is written even though it is the default. A template is where an author learns the
vocabulary, and nobody uses an attribute they never see. The old sentence, "A risk with no
mitigation is a worry.", is false once an accepted risk exists.

`generate_fixtures.rs` rewrites the base's risk back to its committed form, beside the addresses and
the version it already rewrites. Its own header gives the reason: regenerating must not rewrite
committed inputs whose bytes each fixture pins. Without this step, 68 committed inputs would change
a paragraph that pins no rule, and the base would carry a newer attribute under its frozen `0.1.0`.
The new fixtures make their own edits, and declare the current version through
`declaring_the_current_version`.

**Considered: a second, accepted example risk in the template.** Rejected. Every generated fixture
would gain a unit, and one sentence in the existing example teaches the same thing.

### 9. The stylesheet does not render the response in this change

A heading carries one `::after` badge, and a decision's status already replaces its change-state
badge. A response badge would hide a risk's change state in the same way, and that is a
presentation decision of its own. Rendering it would also need new content-addressed revisions of
both stylesheets, a retained revision of each, and a new address in every template, which is what
`record-a-renamed-requirement` decision 7 did. A risk's mitigation paragraph carries no label today
either, so a reader already tells a risk's statement from its mitigation by the prose. After this
change the page reads as it does now, and the response is in the markup and the model.

## Risks / Trade-offs

- **[Two authors settle one partly held-down risk differently.]** One writes `mitigated`, with a
  paragraph that names the gap. The other writes `accepted`, with a paragraph that names the partial
  measure. → The response records the change's answer to the risk, not how much of it remains.
  The answer is `mitigated` when it is the measure, and `accepted` when it is to bear the risk. The
  only thing that reads the value is the rule that decides whether a mitigation is required, and
  both versions satisfy it.
- **[Accepting a risk becomes a way to avoid writing a mitigation.]** → An accepted risk says, in
  its markup and in its statement, that the change bears it, and a reviewer reads that. The
  validator cannot judge whether bearing it is right, just as it cannot judge a decision's status.
- **[A document that is already invalid reports a different rule.]** → See decision 5. The
  CHANGELOG entry names the move, and the new message names both remedies.
- **["Accepted" gains a second meaning.]** `accepted` is also a decision status, and "the accepted
  specifications" is this project's name for its contract. → Each value is read only from its own
  unit's attribute, and the new requirement defines the risk's. In four instruction bodies, the
  phrase "the risks accepted" becomes "the risks and how each was settled", so an agent does not
  read it as a filter on the new value.
- **[A consumer assumed that every risk carries `mitigation`.]** → The field is absent only on a
  risk that declares `accepted`. `response` is always in the model, so a consumer can read it
  first. While the contract is `0.x`, a minor release may make a required field conditional.
- **[A downgrade is quiet for an accepted risk that carries a mitigation.]** An older toolchain
  ignores the attribute, so it reads such a risk as mitigated, with no diagnostic (evidence, variant
  C). → An accepted risk without a mitigation fails loudly on an older toolchain (variant B). In the
  quiet case the risk loses its label, and every sentence the author wrote is kept.

## Migration Plan

This change is additive for documents. No existing document carries `data-response`. All 87
risks in tracked HTML (83 when this was written) and all 11 in the staged migration project as
`mitigated`, with the mitigation they already state. Each validates, and its model gains one field.

**The contract moves one minor version** from the version that is current when this change lands.
It was `0.2.0` when this was written and `0.4.0` when the change was built, and the two changes
below this one in the set, `record-a-task-done-differently` and `record-a-partial-verification`,
move it to `0.5.0` and `0.6.0`. So this change moves it from `0.6.0` to `0.7.0`. The new version
joins the `schemaVersion` enumeration, and every earlier version stays in it.

**The toolchain release is a minor increment, expected to be 0.27.0.** The contributor confirms the
increment at the release task.

**The release runbook is not edited.** Its two risks go on validating as `mitigated`. Declaring
`accepted` there is a later release of the runbook, which moves `runbooks/release/VERSION`. That
file stays at `0.1.0` in this change, so this archive's `runbooks.json` is byte-identical to the
previous archive's.

Rollback is a revert. A document that adopted `accepted` without a mitigation then fails with
`schema/invalid`, which is loud.
