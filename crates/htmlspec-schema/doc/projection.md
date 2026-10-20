# The HTMLSpec HTML-to-model projection

**Contract version:** `1.0.0`, which also accepts documents declaring `0.1.0` to `0.7.0`
**Status:** normative. This document is part of the public contract, alongside
`schema/document.schema.json`. Neither one is the whole contract.

The version is semantic rather than a date, and the schema owns its meaning —
see `schemaVersion` there for the bump policy, and `CHANGELOG.md` for the date
each version landed. This document moves under the same version as the schema,
because a projection change and a schema change are one contract change.

The schema describes the *projected model* — the semantic units, their fields,
and their references. It says nothing about the HTML that produces that model.
This document states the mapping. An implementation written from these two
documents and the conformance fixtures reproduces the model exactly, with no
access to this project's source code.

**The contract is three pieces.** This document states the mapping from HTML to
the model. `schema/document.schema.json` states the shape of the model, and it
owns the identifier prefix table and the change-state semantics — this document
does not restate either one. The conformance fixtures in `fixtures/` pin the
mapping. A consumer that reads only one of the three has read part of the
contract.

Terminology used throughout: *unit* means one of the seven semantic units;
*reserved class* means one of the seven class names in the vocabulary; *the
projection* means the mapping this document defines.

## 1. What creates a unit

1.1 An element creates a unit when, and only when, its `class` attribute
contains one of the seven reserved class names: `requirement`, `scenario`,
`decision`, `task`, `evidence`, `risk`, `verification`.

1.2 A reserved class name is matched as a whole space-separated token, case
sensitively. `class="requirement note"` creates a requirement. `class="requirements"`
creates nothing.

1.3 An element carrying two reserved class names is invalid. The projection does
not choose one.

1.4 The element's tag name does not affect the projection. `section`, `article`,
`div`, and `li` all project identically. The template uses `section` for a
requirement and `article` for a scenario, and that is a convention rather than a
rule.

1.5 An element with no reserved class creates no unit. Its content still
projects, when a field rule below reaches through it. See section 8.

## 2. Attributes that project

| Attribute | Projects to | Absent |
| --- | --- | --- |
| `id` | the unit's `id` | invalid |
| `data-change` | the unit's `change` | `unchanged` |
| `data-state` (task only) | the task's `state` | invalid |
| `data-status` (decision only) | the decision's `status` | invalid |
| `data-outcome` (verification only) | the verification's `outcome` | invalid |
| `data-response` (risk only) | the risk's `response` | `mitigated` |
| `data-superseded-by` (decision only) | the decision's `supersededBy` | field absent |
| `data-refs` | the unit's `references`, with `origin: "attribute"` | no attribute-origin references |
| `data-field` (on a paragraph) | which field of the enclosing unit the paragraph supplies | the unit's primary field, per section 4 |

2.1 `data-refs` holds a whitespace-separated list of unit identifiers. Order is
preserved. A repeated identifier projects once, at its first position.

2.2 Every other attribute is ignored by the projection, including `class` tokens
beyond the reserved one, `style`, `lang`, `role`, and any `data-*` attribute not
named above. An unknown `data-*` attribute is ignored rather than rejected, so
that presentation can carry state the model does not model.

2.3 An attribute value is whitespace-trimmed. No other normalization applies:
`data-state=" done "` projects as `done`, and `data-state="Done"` is invalid.

## 3. Fields common to every unit

3.1 **`title`** projects from the first heading element (`h1` through `h6`) in
document order among the unit's descendants, excluding descendants of a nested
unit. Its text content projects per section 6.

3.2 A unit containing two headings at the same depth relative to the unit is
invalid, because its name is ambiguous. A heading inside a nested unit belongs to
that nested unit and does not count.

3.3 A unit with no heading is invalid.

3.4 **`unit`** projects from the reserved class name, not from any attribute.

## 4. Fields specific to one unit

Fields are named, never positional. The pre-`0.1.0` draft projected a decision's
rationale and a risk's mitigation from paragraph position, and the dogfood
exercise in task 3.4 showed that inserting one sentence of context silently
moved the mitigation out of the model. Position no longer decides a field.

4.0 **`data-field` and the primary field.** Each unit has at most one *primary*
field, listed below. A paragraph with no `data-field` attribute supplies the
primary field of its enclosing unit. A paragraph with `data-field="x"` supplies
field `x`. `data-field="note"` supplies the unit's `notes` array, which carries
prose that is not a field. A `data-field` value that names no field of the
enclosing unit is invalid, with one exception: a `data-field="renamed-from"`
paragraph outside a requirement projects nowhere and is not invalid under this
section. The validator reports it under its own rule (section 4.1.1), so one
misplaced rename produces one diagnostic, and that diagnostic says a rename
belongs on a requirement.

| Unit | Primary field | Other fields by `data-field` |
| --- | --- | --- |
| requirement | `statements` (repeatable) | `statement`, `renamed-from` (single), `note` |
| scenario | none | `note` |
| decision | `statement` (single) | `rationale` (repeatable), `note` |
| task | none | `reason`, `deviation` (single), `note` |
| evidence | `summary` (repeatable) | `note` |
| risk | `statement` (single) | `mitigation`, `note` |
| verification | none | `method`, `unchecked` (repeatable), `note` |

4.0.1 A unit whose primary field is single-valued and which contains two
unmarked paragraphs is invalid. The projection does not choose one.

4.0.2 A unit with no primary field discards nothing: an unmarked paragraph in a
task, a scenario, or a verification projects into `notes`.

4.1 **Requirement.** `statements` projects from every unmarked paragraph and
every `data-field="statement"` paragraph among the requirement's descendants,
excluding descendants of a nested unit, in document order. A requirement with no
statement is invalid. `scenarios` projects from every element carrying the
reserved class `scenario` among the requirement's descendants, in document order.

4.1.1 **`renamedFrom`** projects from the requirement's `data-field="renamed-from"`
paragraph, as normalized text. It states the heading the requirement replaced,
and the requirement's own `title` is its current name. It is single-valued. A
`renamed-from` paragraph never supplies a statement, so a requirement whose only
paragraphs are a `renamed-from` and notes has no statement. Absent when the
requirement carries no such paragraph.

`renamedFrom` holds the first `renamed-from` paragraph whose normalized text is
not empty. Every `renamed-from` paragraph still counts for the validator's rules,
including an empty one and a second one, although neither projects into the
model. An empty paragraph is discarded everywhere else, but an empty
`renamed-from` paragraph is a rename stated without its heading, and reporting it
as empty is more useful than treating it as no rename at all.

The rules that bound a rename are validator rules (section 10.1). They reject a
`renamed-from` paragraph outside a requirement, in an `added` or `removed`
requirement, repeated, empty, or equal to the requirement's *heading name*.

4.1.2 **The heading name** of a unit is its projected `title` with the unit's own
identifier removed from the end, then trimmed. A heading commonly shows its
identifier beside the name, as
`<h3>The name <span class="hs-id">req-x</span></h3>`. Section 3.1 takes the title
from all of the heading's text, so that title is `The name req-x`, and its heading
name is `The name`. A heading written without the identifier has a heading name
equal to its title. The heading name is not a field of the model. It is derived
from two fields the model already has, and it is what a rename compares, both in
validation and when an accepted specification is checked for a rename it has not
taken.

4.2 **Scenario.** `given`, `when`, and `then` project from a description list
(`dl`) among the scenario's descendants. Each `dt` supplies a label and each
following `dd` supplies one entry. The label is matched case-insensitively
against `Given`, `When`, and `Then`, after trimming and after removing one
trailing colon. A label outside that set is invalid. A `dt` with no following
`dd` is invalid. A `dd` with no preceding `dt` is invalid.

4.3 A scenario outside a requirement is invalid. A requirement nested inside
another requirement is invalid. No other nesting of units is defined, and an
undefined nesting is invalid.

4.4 **Decision.** `statement` projects from the single unmarked paragraph.
`rationale` projects from every `data-field="rationale"` paragraph, in document
order. A decision with no statement, or with no rationale, is invalid.

4.5 **Task.** `reason` projects from a `data-field="reason"` paragraph. An
unmarked paragraph in a task projects into `notes`, so a task that is not
dropped can carry prose without projecting a reason that means nothing.

`deviation` projects from a `data-field="deviation"` paragraph, as normalized
text. It states what was done, where that departs from the task's words, and the
task's title keeps the words as written. It is single-valued, and absent when the
task carries no such paragraph or only an empty one, which reads as a task done as
its words say. An unmarked paragraph never supplies it. Only a `done` task may
state one, and that is a validator rule (section 10.1): the projection produces
the field whatever the task's state.

4.5.1 A task containing more than one `deviation` paragraph whose normalized text
is not empty is invalid. The projection does not choose between two accounts of
what was done. `deviation` holds the first, so a reader of a projection that was
not validated still reads the task as done another way than its words say, rather
than as done as written.

4.6 **Evidence.** `summary` projects from every unmarked paragraph and every
`data-field="summary"` paragraph. At least one is required.

4.7 **Risk.** `statement` projects from the single unmarked paragraph, and
`mitigation` from a `data-field="mitigation"` paragraph, when one exists. A risk
with no statement is invalid. A risk with no mitigation is not invalid here:
whether it must state one depends on its response (section 4.7.1), and that
condition is a validator rule (section 10.1).

4.7.1 **`response`** projects from the risk's `data-response` attribute,
trimmed per section 2.3, and is `mitigated` when the attribute is absent. It
says how the change settled the risk: `mitigated` when the change holds it down,
and `accepted` when the change bears it as it stands. Every risk in the model
carries a response, including one whose markup declares none, so a consumer
never applies the default itself. A value outside the two survives projection,
and the validator reports it.

`mitigation` projects whatever the response. On an `accepted` risk, a mitigation
paragraph states the part of the risk that something does hold down. The response
is never inferred from whether a mitigation paragraph is present, and the response
and the change state are never inferred from each other. A `data-response`
attribute on a unit other than a risk projects nowhere (section 2).

4.8 **Verification.** `method` projects from a `data-field="method"` paragraph,
when one exists. `unchecked` projects from every `data-field="unchecked"`
paragraph among the verification's descendants, one entry per paragraph, in
document order, and is absent when there is none. Each entry states a part of
what the verification names that its check did not reach. An unmarked paragraph
in a verification projects into `notes` (section 4.0.2), never into `unchecked`.
Whether a verification must carry one is a validator rule (section 10.1): the
projection records a `partial` verification with no `unchecked` paragraph as
written.

## 5. References

5.1 A reference enters the model from exactly two places: the `data-refs`
attribute of section 2, with `origin: "attribute"`, and an anchor whose `href` is
a same-document fragment, with `origin: "link"`.

5.2 An `a` element with `href="#req-x"` inside a unit projects a reference to
`req-x` with `origin: "link"`, and also projects into the document-level `links`
array. Both projections happen: the reference expresses the relationship and the
link expresses the resolvable hyperlink.

5.3 An `a` whose `href` fragment does not match the identifier grammar of the
schema's `unitId` projects into `links` only, and not into `references`. A link
to `#section-overview` is a link and not a reference. A link to a figure
identifier is also a link only, because a figure is not a semantic unit and
`figure-` is not a unit prefix.

5.4 An `a` with an absolute or protocol-relative `href` projects into neither
array. It is a citation, and the safety rules govern it.

5.5 Reference order in the model is `data-refs` entries first, in attribute
order, then link references in document order.

## 6. Whitespace and text

6.1 **Normalized text** applies to every field whose schema type derives from
`nonEmptyText`: titles, statements, scenario entries, rationale, summaries,
mitigations, methods, the entries of `unchecked`, reasons, deviations, figure captions,
and the entries of `deletions` (section 8.6.2). Normalization replaces every
run of whitespace, including newlines, with one space, then trims. This is what
makes an author's line breaks invisible to the model. A requirement's
`renamedFrom` is normalized text too.

6.2 **Preserved text** applies inside `pre`, `code`, and `textarea`. Content
projects byte-for-byte, including indentation and newlines, with no trimming and
no normalization. A figure's `source` is preserved text.

6.3 Character references are resolved before either rule applies. `&amp;` in the
source projects as `&`. This means the model cannot distinguish `&amp;` from a
literal `&` in the source, and the formatter, not the projection, decides which
form is written back.

6.4 Comment nodes contribute no text. A comment inside preserved text is
retained as a comment by the formatter and contributes nothing to the projected
string, so text split by a comment projects as the concatenation of its parts.

## 7. Figures and visualization sources

7.1 A `figure` element projects one entry into the document-level `figures`
array. A figure creates no unit. It takes an optional `id` using the `figure-`
prefix, which makes it addressable by a link, so that a unit can point a reader
at the chart or diagram it rests on. A figure identifier is never a `references`
target.

7.2 `caption` projects from the figure's `figcaption`, as normalized text. A
figure with no `figcaption` is invalid.

7.3 `kind` and `source` project from the figure's first `pre` element:

| First `pre` | `kind` | `source` |
| --- | --- | --- |
| `<pre class="mermaid">` | `mermaid` | preserved text of the `pre` |
| `<pre class="vega-lite">` | `vega-lite` | preserved text of the `pre` |
| any other `pre` | `code` | preserved text of the `pre` |
| no `pre`, but a `math` descendant | `math` | absent |
| no `pre` and no `math` | `prose` | absent |

7.4 A visualization source is ordinary visible content. It is never carried in a
`script` element, because a `script` is invisible without JavaScript and a
document using one would fail the no-JavaScript readability rule while appearing
to satisfy it. A `details`/`summary` wrapper around the `pre` is presentational
and projects per section 8.

7.5 The rendered output of a diagram or chart never projects. A saved rendered
page projects the same model as the file on disk, because the renderer draws into
a sibling element and never over the source.

## 8. Presentational markup

8.1 Every field rule above reaches through elements that carry no reserved class.
A `p` wrapped in `<div class="callout">` inside a requirement is still that
requirement's statement.

8.2 Presentational elements themselves are discarded: they contribute no field,
no unit, and no entry to any array. Their inline text content contributes to
whichever field encloses them. The one element with no reserved class that is not
presentational is `del`, under section 8.6.

8.3 Inline elements inside a field's text — `strong`, `em`, `code`, `a`, `abbr`,
`span` — contribute their text content to the field, per section 6. The model
holds text, not markup, so emphasis is invisible to the model and visible to a
reader.

8.4 A `header`, `footer`, `nav`, `aside`, or `details` element is presentational
for the purposes of this projection. None of them is special-cased.

8.5 The following elements contribute no text to any field, wherever they appear:
`script`, `style`, `template`, `noscript`. Their content is invisible to the
model. The safety rules, not the projection, decide whether they may appear at
all.

8.6 **Change markup.** `ins` marks text an edit added, and `del` marks text an
edit removed. Only elements in the HTML namespace are change markup: an element of
the same name inside SVG or MathML belongs to that language and projects under the
rules above. `ins` is presentational under sections 8.2 and 8.3, because the text
it marks is current text, and it contributes that text to the field that encloses
it. `del` is not presentational. Its content projects under 8.6.1 and 8.6.2. The
projection applies both rules to every document, whatever version it declares and
wherever it sits. Where the markup is permitted is a validator rule (section
10.1), so a document the validator rejects for its markup still projects this way.

8.6.1 The content of a `del` supplies no field of anything that encloses the
`del`. Its text contributes nothing to normalized text (section 6.1) or to
preserved text (section 6.2), so a title, a statement, a scenario entry, a note, a
summary paragraph, a caption, and a figure's source hold only the current text. An
element inside a `del` supplies nothing a rule above would take from it: a
paragraph supplies no field (sections 4.0 to 4.8, and 9.1.1); a heading supplies
no title and counts toward no depth (sections 3.1 and 3.2); a description list, a
`dt`, and a `dd` supply no scenario entry (section 4.2); a `figcaption`, a `pre`,
and a `math` supply nothing to the figure around the `del` (sections 7.2 and 7.3);
and an `a` projects no reference (section 5.2). That `a` still projects into
`links`, because the page still carries the hyperlink. A unit or a figure inside a
`del` still projects: the `del` around it supplies none of its fields, and its own
content projects as though the `del` were not there. A `del` whose own class names a
unit is that unit (section 1.1), and it projects as a unit inside a `del` does: from
its own content. It is no entry of the `deletions` of the unit around it, and it
records none of its own.

8.6.2 **`deletions`** on a requirement and on a scenario projects from each `del`
that belongs to the unit and that no other `del` within the unit encloses, in
document order. A `del` belongs to the nearest unit that encloses it, so a `del`
inside a scenario is the scenario's and not its requirement's. Each entry is the
normalized text of the element's whole content, read as section 6.1 reads it but
without leaving out any `del` inside it, so a nested `del`, an `ins`, and the text
of a unit inside the element are all part of the entry. A `del` whose content is
empty once normalized records nothing. Absent when the unit records no deletion. A
`del` in any other unit, or outside every unit, is recorded nowhere: the model has
no place for it, and the page still shows it.

## 9. Document metadata

9.1 `subject` projects from the `title` element of the document head, as
normalized text.

9.1.1 `summary` projects from every `p` element inside the document's `header`
element, in document order, as normalized text. This is where a document says
what it is for. Without it the argument a change rests on would live in markup
the model cannot address, which is the failure this format exists to prevent. A
document with no `header`, or a `header` with no paragraph, projects an empty
summary and stays valid, because a fragment under review is still a document.

9.2 `schemaVersion` projects from `<meta name="htmlspec-version" content="...">`.
An absent or unrecognized version is invalid, because a document that does not
declare its contract version cannot be checked against one.

9.3 `kind` projects from `<meta name="htmlspec-kind" content="...">`. An absent
kind is invalid, because nothing then determines which units belong in the
document. A kind outside the schema's enumeration is invalid, because the set of
kinds is closed and a fifth would carry no stated rules. There is no default: an
undeclared kind is reported as absent rather than being taken for the most
convenient value, which would silently make an unmigrated proposal into the
strictest kind.

9.4 Document order for every array in the model is depth-first document order of
the projecting elements, except where section 5.5 states otherwise.

## 10. What this projection does not decide

10.1 Identifier uniqueness, prefix agreement with unit type, reference
resolution, references to a `removed` unit, the modal vocabulary of a normative
statement, the conditional requirements that a `dropped` task states a reason,
that a deviation is stated only on a `done` task, that a `superseded` decision
names its replacement, that a `partial` verification states what it did not
check, and that a `mitigated` risk states its mitigation (section 4.7.1), the
placement and consistency of a
`renamed-from` paragraph (section 4.1.1), and where change markup is permitted,
meaning whether an `ins` or a `del` element may sit in or around a unit, which the
validator permits only in a requirement or a scenario marked `modified` in a
change's specification while section 8.6 says what the markup means wherever it
sits — all of these are validator rules. The projection produces the model. It
does not judge it.

10.1.1 A figure identifier shares the document's identifier namespace, so the
validator's uniqueness rule covers it even though a figure is not a unit.

10.2 Where this document says "invalid", it means the projection cannot produce a
model and the validator reports a `schema/*` diagnostic. A projection failure is
a document defect and never a silent default.
