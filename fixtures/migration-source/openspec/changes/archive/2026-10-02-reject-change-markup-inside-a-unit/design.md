## Context

See proposal.md for why. The shape of the current system that the approach depends on:

- **The projection is doing what its specification says.** `crates/htmlspec-schema/doc/projection.md`
  section 8.2 makes every element without a reserved class presentational, and "their inline text
  content contributes to whichever field encloses them". In `crates/htmlspec-parse/src/project.rs`,
  `own_descendants` walks through any such element, and `collect_text` concatenates text with only
  `script`, `style`, `template`, and `noscript` left out. So `del` and `ins` are read like `span`. No
  crate, fixture, template, instruction, or accepted specification names either element
  (`evidence/the-measurement-2026-09-30.md`, section 4).
- **The model holds text, not markup.** A rule that reads only the model cannot see change markup.
  A rule receives `Subject { projection, store }` (`crates/htmlspec-validate/src/engine.rs`), and
  `Projection.document` holds the parsed tree. Five rule modules already read it: `a11y`, `assets`,
  `executable`, `refs`, and `safety`.
- **There is a precedent for judging an element by where it sits.** `executable.rs` reports a
  `code` element by what it contains. Its rule, like every rule in `ALL`, is one `fn` over the
  subject.
- **What a unit is has one definition.** Projection sections 1.1 and 1.2: an element creates a unit
  when its `class` holds a reserved token, matched whole and case sensitively. `UnitKind::from_class`
  is public in `htmlspec_parse`, and `enclosing_unit` in `project.rs` asks the same question of
  ancestors.
- **Rule namespaces are a closed set that decides severity** (`diagnostic.rs`). `schema/*` is
  contract conformance and an error. `review/*` is a warning that strict mode never promotes.
- **Every rule needs a valid and an invalid conformance fixture** (`every_rule_has_both_a_valid_and_an_invalid_fixture`),
  and `known_rules()` lists them. The generator, `conformance/src/bin/generate_fixtures.rs`,
  deletes every fixture directory missing from its table. Four hand-authored fixtures are missing
  from it. `2026-09-26-log-a-runbook-run/evidence/the-generator-deletes-two-fixtures-2026-09-18.md`
  recorded two of them, and `2026-09-26-record-a-renamed-requirement` found four and restored them
  by hand.
- **`crates/htmlspec-validate/tests/schema_rust_split.rs`** forbids rule code from spelling any
  schema-owned literal, including each accepted contract version.

## Goals / Non-Goals

**Goals:**

- A document with change markup in or around a unit fails validation, with one error per element
  that names the unit and the element.
- The rule and the projection cannot disagree about what a unit is.
- The change narrows what validates and nothing else. Every document that still validates projects
  the same model as before.
- The change planned to give the markup meaning can relax this rule without breaking any document
  this change accepted.

**Non-Goals:**

- Giving `ins` and `del` meaning. The projection is untouched.
- `s` and `strike`. HTML defines `s` as content that is no longer accurate or relevant and not as
  an edit, and `strike` is its obsolete spelling. Neither is what the later change gives meaning to.
- The workflow instructions. The diagnostic states the fix. An instruction added here would be
  replaced by the change that tells an agent how to mark an edit.
- The formatter. `format --check` reports a block-level `<del><p>` as not canonical, and `format`
  only moves the `p` onto its own line inside the `del`, leaving the projected statements as they
  were (evidence, section 3).

## Decisions

### 1. One rule, `schema/change-markup`, and an error

The rule lives in `schema/*`, whose concern is conformance to the schema and the projection
specification. A document whose projection states what its author withdrew does not conform in
the sense that matters: its model is wrong.

**Considered: a `review/*` warning.** Rejected. A review warning never gates work, and strict mode
does not promote it. Measured: `validate --strict` accepts the document today with its one
`review/*` warning, so a warning here would leave the wrong contract accepted by every check a
project can run. A review warning says a dependent unit may be out of date. This statement is wrong
now.

**Considered: `safety/*`.** Rejected. Nothing executes and nothing reaches the network, and the
safety floor exists for documents that are unsafe to open.

**Considered: a new namespace.** Rejected. A namespace exists to decide severity, and this rule
needs the severity `schema/*` already has.

**Considered: `kind/*`.** Rejected. The rule is the same in every kind, and
`the_kind_namespace_has_exactly_two_rules_and_constrains_no_upstream_kind` pins that `kind/*` has
exactly two rules.

**Considered: a projection fault, reported as `schema/projection`.** Rejected. Projection section
10.2 says a projection fault means the projection cannot produce a model, and here it can. Section
10.1 is the list of validator rules that judge a model the projection produced, and this rule
belongs on it. A new numbered projection rule would also need a mapping fixture whose expected
model pins `nothing elsestore`, which is the behavior the later change replaces.

**The name is `change-markup`**, a noun phrase for the fault, like `weak-modal` and `duplicate-id`.
**Considered: `change-markup-in-unit`**, after `rename-outside-requirement`. Rejected. It would be
wrong for a `del` around a unit (decision 3), and it would stop being right when the later change
narrows where the rule fires. Rule identifiers are stable across versions, so the name should
describe the fault and leave the place to the message.

### 2. The rule reads the tree and asks the projection's own question about units

The rule module is `crates/htmlspec-validate/src/rules/change_markup.rs`. It walks
`subject.projection.document` once, as `executable.rs` does. For each `ins` or `del` element, it
looks for a unit among the element's ancestors and then among its descendants. An element is a unit
when a whitespace-separated token of its `class` is one `UnitKind::from_class` accepts. That is
projection section 1.2's test, reached through the parse crate, so the rule cannot recognize a unit
the projection does not. It is also the test `enclosing_unit` applies: an element carrying two
reserved tokens (section 1.3) still counts, so markup inside one is still reported alongside the
projection's own fault.

The element names `ins` and `del` are constants in the rule module. They are HTML's vocabulary and
not the schema's, so `schema_rust_split.rs` has nothing to say about them.

**Considered: recording change markup in the projection, beside the model**, as
`Projection::misplaced_renames` does for renames. Rejected. That field exists because the
projection discards a `renamed-from` paragraph that the validator must still report, so the fact had
to leave the projection somewhere. Here the tree already holds everything the rule needs, and a new
public field on `Projection` would be carried by the parse crate for one rule. The later change
reshapes this part of the projection anyway.

### 3. In a unit, or around one

An element is reported when a unit encloses it, or when it encloses a unit. The second case was
measured: a requirement wrapped whole in `<del>` projects as an `unchanged` requirement, and
`validate` exits 0 (evidence, section 5). A rule that looked only at ancestors would pass it. The
vocabulary already has the statement a wrapped unit is reaching for. A withdrawn unit is
`data-change="removed"`, and a new one is `data-change="added"`.

A review of the implementation found a third case. A change element whose own `class` names a unit,
such as `<del class="decision">`, is that unit, and the projection reads it as one the document
states. It is reported at itself, before its ancestors or descendants are asked. Only HTML's own
`ins` and `del` count: an element of the same name inside SVG or MathML is that language's, and is
not change markup.

There is one diagnostic per element. It is located at the element itself when it is a unit, else at
the nearest enclosing unit, or else at the first unit the element encloses in document order, and
it names the element and quotes its normalized text. An element inside one unit and around another
is located at the enclosing unit, and its fix is the change state of the unit it wraps. A long text, such as a whole wrapped requirement, is quoted by its opening words.
Nested change elements are reported one by one. Each is its own edit to undo, and the quoted text
is how an author finds it in a long statement.

**Considered: one diagnostic per unit.** Rejected. A unit with three marked edits has three places
to fix. A single diagnostic would name one and imply the rest.

### 4. `ins` is rejected with `del`

Read against the code, an `ins` does no harm to the model. Its text is current text, and the
projection keeps it, as section 1 of the evidence shows with `store`. So `ins` is not rejected for
model damage. It is rejected for two other reasons.

- **Half a diff.** A unit that marks words as inserted and nothing as removed tells a reader that
  the words are new, and hides what they replaced. The rendered page shows one side of an edit.
- **The later change can only relax.** `mark-the-words-a-modification-changed` is planned to give
  both elements meaning together, and only in a unit marked modified in a change's specification.
  If `ins` were accepted everywhere now, documents could carry it in the places that change will
  still forbid: an accepted specification, a unit not marked modified, or a task. That change would then
  have to break documents this one let through. Rejecting both now means the later change only
  permits.

**Considered: reject `del` alone.** Rejected, for the two reasons above. The cost of rejecting
`ins` is an error for an author who marks only additions. The count in section 4 of the evidence
is zero, so no existing document pays it.

### 5. Outside every unit, the rule is silent

Section 5 of the evidence found where change markup outside every unit reaches the model:

- a paragraph of free prose reaches nothing;
- the document's summary keeps both texts, run together;
- a figure's caption keeps both texts, run together;
- a deleted fragment link stays in `links`.

None of the three that reach the model carries a contract. The summary is compared with a
template's placeholder (`rules/review.rs`, and `commands.rs` for archival), a caption is checked
for whether it exists (`rules/a11y.rs`), and a link is checked for whether its target resolves
(`rules/refs.rs`).

The rule stops at units for three reasons:

- The defect is a wrong contract, and units are where a document states one.
- Showing a wording change in prose is a legitimate use, and it corrupts nothing. A proposal
  arguing for a change to a requirement can write the before and the after with `del` and `ins`.
- The later change gives the markup meaning only inside units. A prohibition outside units would be
  a permanent rule about prose, and nothing measured here asks for one.

**Considered: the whole document.** Rejected. It forbids the prose use above, and it turns a
stopgap into a permanent rule that the later change would have to carve back out.

**Considered: every place the model holds text, meaning units, the summary, captions, and link
text.** Rejected. It ties the rule to the projection's list of text sources, which would need
updating whenever the projection reads text from a new place. The three extra places state no
contract.

**Open for the contributor: the summary.** The summary is the one place outside a unit where an
author writes the argument itself, and projection section 9.1.1 calls it "the argument a change
rests on". A summary reading "gains no new spellingone spelling" misreports that argument to
anything that reads the model. If the contributor wants it covered, the rule gains the document's
`header` as a second place, one scenario and one test are added, and the valid fixture needs no
change, because its markup is in free prose. The default here is not to cover it, for the reason
the first bullet gives.

### 6. Every kind, and every declared version

The rule applies in all four kinds. A requirement unit in a proposal is not authoritative, but its
model is read, and a decision in a design is read the same way. Tasks are included. Change markup
stays confined to where the later change gives it meaning, and a task's own way of withdrawing
words is `data-state="dropped"` with its reason.

The rule applies whatever contract version a document declares. The schema's `schemaVersion`
description says "the vocabulary accepted is the newest version's", and document-validation's
asset requirement states the principle this follows: "A document SHALL NOT be able to exempt
itself". The conformance fixtures declare the generator's frozen `0.1.0`, so the invalid fixture
is a standing check that an earlier declared version is no exemption.

**Considered: applying the rule only to documents declaring `0.3.0`.** Rejected. Declaring an older
version would switch the rule off, and a document written against an old template would keep
projecting withdrawn words as current.

### 7. The projection specification names the rule, and document-schema's requirement is unchanged

Projection section 10.1 lists the validator rules the projection does not decide, and
`record-a-renamed-requirement` added its rules there. This change adds "change markup in or around
a unit" to that sentence. Section 10.1 is already claimed by the `unjudged` mapping fixture, so
`every_projection_rule_has_a_mapping_fixture` needs nothing new.

document-schema's "The HTML-to-model projection is published as a normative specification" is not
modified. Its list asks the projection specification to state "which elements and attributes
create a semantic unit, and which are ignored" and "how presentational markup that carries no
semantic unit is discarded". Section 8.2 states both, and `del` is handled exactly as it says.

**Considered: a MODIFIED block naming change markup in that requirement.** Rejected. It would change
no obligation, because the requirement lists topics and the topic is already covered. The later
change is planned to modify that requirement, because it changes what the projection does with the
markup. A block here would make it re-base on text that changed nothing.

**Considered: a new numbered rule in section 8, saying that `ins` and `del` are presentational and
validation rejects them in a unit.** Rejected, for the mapping-fixture reason in decision 1.

### 8. Two fixtures, and a generator that keeps what it did not write

The fixtures are twins that differ in where the markup sits:

- `invalid-schema-change-markup`: the base requirement's statement with a `del` and an `ins` in it.
  It declares `schema/change-markup`.
- `valid-change-markup-outside-a-unit`: the same markup in a paragraph between the "Requirements"
  heading and the requirement, outside every unit.

Before the generator is run, it gains a table naming the four hand-authored fixtures:
`valid-runbook-machine-requirements`, `valid-runbook-machine-requirements-unsatisfiable`,
`valid-summary-quoting-the-placeholder`, and `warning-review-placeholder-summary`. Its stale-fixture
sweep keeps every name in that table and writes none of them.

**Considered: restore them with `git checkout` after each run**, as `record-a-renamed-requirement`
did. Rejected. The trap has been met twice, and it grew from two fixtures to four between those
meetings. `AGENTS.md` treats a condition that people must remember as a candidate for enforcement
by tooling, and this one has already been forgotten once.

**Considered: make them ordinary generator entries.** Rejected. They are not edits to the base,
since two are runbook documents and one is a crafted summary. The generator also writes
`expected.json` in its own layout, and `valid-runbook-machine-requirements-unsatisfiable` holds its
rule list on one line, so regenerating it would rewrite committed bytes.

## Risks / Trade-offs

- **[A project outside this repository carries change markup in a unit and stops validating.]** →
  That is the rule working, because such a document's model states withdrawn words. The error
  names the unit and quotes the text, and the `0.3.0` changelog entry names the rule and the fix:
  state the text as it now reads, and mark the unit `modified`. The measurement found no such
  document in this repository or in the staged migration.
- **[Until the later change lands, an author has no readable way to show an edit inside a unit.]**
  → The prose around the unit still takes `del` and `ins`, so a proposal or a specification can
  show the before and the after beside the unit that states the after.
- **[The summary, a caption, or a link's text still runs deleted and inserted words together.]** →
  Measured and left open under decision 5. None of the three states a contract.
- **[The later change must relax a rule this change adds.]** → The rule's name describes the fault
  and not the place, so the later change narrows where it fires without renaming it, and it
  relaxes a rule rather than breaking a document (decision 4).
- **[A mutation that confines the rule to ancestors passes every inline test.]** → The
  wrapped-unit test exists for exactly that, and task 5.2 names the mutation.

## Migration Plan

**The document contract moves from `0.2.0` to `0.3.0`.** A document that validates today can stop
validating, which the versioning table in `crates/htmlspec-schema/CHANGELOG.md` puts under a major
bump. The same file says: "While the major version is `0`, the vocabulary is provisional and any
release can break a document." A major bump from `0.x` is `1.0.0`, which commits the vocabulary,
and that is not this change's decision. A patch is for wording that leaves the model unchanged. So
the bump is minor under the `0.x` clause, and the changelog entry says in its first line that a
document can stop validating, and which.

`0.1.0` and `0.2.0` stay in `schemaVersion`'s enumeration. Removing them would invalidate every
document that declares one: 110 of the 122 HTML files the repository tracks, and all 116 staged
ones. That would reject every document to catch a markup none of them carries (evidence, section
4). The rule reaches documents declaring them anyway, per decision 6.

Rollback is a revert. No document needs editing in either direction, because none carries the
markup.

**The release is expected to be `0.19.0`, a minor increment**: a new validation error changes what
`validate` accepts, and the contract moves. The contributor confirms the increment at task 6.2.
