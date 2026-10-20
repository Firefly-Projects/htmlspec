# How a risk is settled today, measured against HEAD

**Date:** 2026-09-30
**Tasks:** 1.1 and 1.2. The sections on the archive and the migration serve the proposal's Why.
No task repeats them, because neither source changes.
**Binary:** `htmlspec 0.18.0 (6a591bdf9d92)`. `--version` prints `6a591bdf9d92-dirty`, because the
eleven change directories under `openspec/changes/` are untracked. `git status` shows no modified
tracked file. Models come from `~/projects/html-spec-migration/tools/project-model`, which links
this checkout's `crates/htmlspec-parse`. It was built on 2026-09-29 at 20:01, after the last commit
to touch that crate (`51c5aea`).

**Note, 2026-10-01.** After this measurement, the attribute was renamed `data-response` and its
model field `response`. The scratch documents below used the earlier name, `data-disposition`, and
are recorded as they were run. The measured behavior is the same for either name: HEAD ignores an
unknown `data-*` attribute, so neither name reaches the diagnostic or the model.

## 1. The release runbook's two risks

`runbooks/release/specification.html` holds the repository's only risk units outside the fixtures,
the template, and archived changes. Both are quoted verbatim:

```html
200:          <section class="risk" id="risk-this-runbook-goes-stale" data-change="unchanged">
201:            <h3>A step changes and this runbook does not <span class="hs-id">risk-this-runbook-goes-stale</span></h3>
202:            <p>Nothing compares this runbook against the scripts and commands it names. A stage renamed in the check suite, or a flag the bootstrap command stops accepting, leaves this document confidently wrong.</p>
203:            <p data-field="mitigation">Partly held down and honestly not fully. The steps name commands rather than restating what those commands do, so a renamed stage produces a command that fails loudly at the moment it is run rather than a silently wrong instruction. What is not held down is a step that becomes unnecessary: nothing would report that. The check this project already runs over its installed instruction files is the shape that would close it, and building it is not this runbook's work.</p>
204:          </section>
205:          <section class="risk" id="risk-the-inputs-are-prose" data-change="unchanged">
206:            <h3>The inputs are prose that nothing checks <span class="hs-id">risk-the-inputs-are-prose</span></h3>
207:            <p>This procedure requires a toolchain, an interpreter, a clean working tree, and a repository that is not already a project. All four are prose in the tasks document, so a run on a machine missing one discovers it partway through — at the step that needs it, with the version already advanced.</p>
208:            <p data-field="mitigation">Not mitigated, and recorded rather than hidden. A runbook has no notion of an input, which is an open finding of the change that established runbooks. The nearest existing mechanism is the pinned development shell, which supplies all four when a run happens inside it — so the practical advice is to run this procedure there, and that advice is not a check.</p>
209:          </section>
```

Line 208 is a mitigation paragraph whose first sentence says the risk is not mitigated. The model
holds that sentence in the risk's `mitigation` field.

## 2. A risk with no mitigation, in a scratch project (task 1.1)

A scratch project, and one change whose design carries one risk:

```
htmlspec init --tools none --root <scratch>/proj      exit 0
htmlspec new change settle-a-risk                      exit 0
```

The design's placeholder summary was replaced with "This design records one risk the project
bears.", so the only diagnostics are the risk's. Five variants of the design were validated with
`htmlspec validate htmlspec/changes/settle-a-risk/design.html`. Each differs from the one before it
in one way.

**Variant A: a statement and nothing else.**

```html
<section class="risk" id="risk-the-inputs-are-prose" data-change="added">
  <h3>The inputs are prose that nothing checks <span class="hs-id">risk-the-inputs-are-prose</span></h3>
  <p>A run on a machine missing one of the four inputs discovers it partway through, with the version already advanced.</p>
</section>
```

```
htmlspec/changes/settle-a-risk/design.html
  error: schema/invalid at risk-the-inputs-are-prose (/risks/0): "mitigation" is a required property

  1 error, 0 warnings: validation failed
exit 1
```

With `--json`, the one diagnostic is:

```json
{ "rule": "schema/invalid", "namespace": "schema", "severity": "error",
  "message": "\"mitigation\" is a required property",
  "unitId": "risk-the-inputs-are-prose", "schemaPath": "/risks/0" }
```

**Variant B: A plus `data-disposition="accepted"` on the risk element.**

```
  error: schema/invalid at risk-the-inputs-are-prose (/risks/0): "mitigation" is a required property

  1 error, 0 warnings: validation failed
exit 1
```

The `--json` payloads of A and B are byte-identical (`cmp` reports no difference). So are their
projected models:

```json
[{ "change": "added", "id": "risk-the-inputs-are-prose",
   "statement": "A run on a machine missing one of the four inputs discovers it partway through, with the version already advanced.",
   "title": "The inputs are prose that nothing checks risk-the-inputs-are-prose", "unit": "risk" }]
```

**The attribute is ignored.** Projection section 2.2 says: "An unknown `data-*` attribute is ignored
rather than rejected." The author's statement that the risk is accepted does not reach the model,
and the error still demands a mitigation.

**Variant C: B plus the release runbook's own workaround,**
`<p data-field="mitigation">Not mitigated, and recorded rather than hidden.</p>`.

```
  no diagnostics: the document is accepted
exit 0
```

**Variant D: C without `data-disposition`.**

```
  no diagnostics: the document is accepted
exit 0
```

The models of C and D are byte-identical. Both read:

```json
[{ "change": "added", "id": "risk-the-inputs-are-prose",
   "mitigation": "Not mitigated, and recorded rather than hidden.",
   "statement": "A run on a machine missing one of the four inputs discovers it partway through, with the version already advanced.",
   "title": "The inputs are prose that nothing checks risk-the-inputs-are-prose", "unit": "risk" }]
```

| Variant | Disposition written | Mitigation paragraph | Exit | Rule | `disposition` in the model |
| --- | --- | --- | --- | --- | --- |
| A | none | none | 1 | `schema/invalid` | absent |
| B | `accepted` | none | 1 | `schema/invalid` | absent |
| C | `accepted` | "Not mitigated, …" | 0 | none | absent |
| D | none | "Not mitigated, …" | 0 | none | absent |

**Today the only valid way to write an accepted risk is D.** It is a mitigation that says it is not
one. The attribute an author might reach for changes nothing, in validation or in the model.

**Variant E, a side measurement for design decision 6:** D plus a decision unit that carries
`data-status="accepted"`, `data-state="done"`, and `data-outcome="passed"`. Validation exits 0 with
no diagnostics. The decision projects `"status": "accepted"` and nothing from the other two
attributes. **A per-unit attribute on the wrong unit is ignored today, silently.**

## 3. Every risk unit in the repository and in the staged migration (task 1.2)

**Tracked HTML.** `git ls-files '*.html'` lists 122 files. They were projected with `project-model`,
and every projected risk was checked for a `mitigation` field. The elements were also counted with
a parser, by class token:

| Where | Files | Risk units |
| --- | --- | --- |
| `fixtures/conformance/` | 64 | 66 |
| `fixtures/projection/` (`named-fields`, `single-field-repeated`) | 2 | 2 |
| `fixtures/dogfood/foundation-specification.html` | 1 | 2 |
| `assets/template-specification.html` | 1 | 1 |
| `runbooks/release/specification.html` | 1 | 2 |
| `2026-08-10-html-spec/documents/deterministic-formatting.html` | 1 | 1 |
| `2026-08-14-separate-artifact-kinds/documents/design.html` | 1 | 3 |
| `2026-09-11-migrate-an-openspec-project/evidence/a-migrated-design-2026-09-09.html` | 1 | 6 |
| **Total** | **72** | **83** |

84 elements carry the `risk` class token. The 84th is `risk-both` in
`invalid-schema-projection-two-reserved-classes`, whose `class="risk decision"` projects no unit,
by section 1.3. **All 83 projected risks carry a mitigation. None lacks one.**

**The staged migration.** `~/projects/html-spec-migration/staged/` holds 116 HTML files. Eleven risk
units project from them, and **all eleven carry a mitigation**:

| Staged file | Risk units | What the file is |
| --- | --- | --- |
| `…/2026-08-10-html-spec/evidence/documents/deterministic-formatting.html` | 1 | carried evidence, byte-identical to the tracked file (`cmp`) |
| `…/2026-08-14-separate-artifact-kinds/evidence/documents/design.html` | 3 | carried evidence, byte-identical |
| `…/2026-09-11-migrate-an-openspec-project/evidence/a-migrated-design-2026-09-09.html` | 6 | carried evidence, byte-identical |
| `…/2026-09-06-distinguish-a-requirement-from-an-example/design.html` | **1** | a document the migration drafted |

So the migration drafted **one** risk unit. The other ten are records it carries unchanged, which
artifact-kinds' "A change's evidence is kept in the change and is not one of its documents" keeps
out of the project's documents. The drafted unit, verbatim, at lines 128-132:

```html
<section class="risk" id="risk-an-agent-that-cites-a-requirement-that-does-not-support-the-claim" data-change="added">
  <h3>Risk: an agent that cites a requirement that does not support the claim <span class="hs-id">risk-an-agent-that-cites-a-requirement-that-does-not-support-the-claim</span></h3>
  <p><strong>Accepted, and out of scope by design.</strong> A check can see that a requirement was named. It cannot see whether the requirement supports the claim, because that is reading. An agent that cites <code>project-bootstrap</code>'s statement about "each in its own place" as authority for a flat filename has satisfied the letter and misread the requirement.</p>
  <p data-field="mitigation">The mitigation is that a named requirement is <em>checkable by a person in seconds</em>, where an invented rule with a confident justification is not checkable at all. That is the whole gain, and it is worth stating as a partial repair rather than overclaiming.</p>
</section>
```

**It is accepted, and it has a mitigation.** Carried as-is under a default of `mitigated`, it would
still validate. It would also claim a disposition its author contradicts.

## 4. How the archive's designs settle their risks

The method runs over the 24 `openspec/changes/archive/*/design.md` files. A **risk entry** is a
bullet or a bold-led paragraph in a `## Risks` section, up to the next `##` heading. It is
**arrow-shaped** when it contains `→` or `->`. A **marker** is one of these words applied to the
risk itself:

- **accepted:** `Accepted`, `accepted`, or `accepts`, not followed by "specification", "contract",
  "requirement", or a backtick;
- **mitigated:** `Mitigated`, `mitigated`, `Mitigation`, or `mitigation`, excluding "not mitigated"
  and one sentence that denies a mitigation ("rather than one mitigated after the fact", in
  `catch-a-templates-placeholder-prose`);
- **not mitigated:** `Not mitigated`, `unmitigated`, or `not solved`.

Every classified entry was also read by hand, and the counts agree.

| Design | Risk entries | Arrow-shaped | Accepted | Mitigated | Not mitigated | No marker | `### Risk:` headings |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `2026-08-10-html-spec` | 8 | 8 | 0 | 6 | 0 | 2 | 0 |
| `2026-08-13-content-address-runtime-assets` | 6 | 6 | 0 | 0 | 0 | 6 | 0 |
| `2026-08-14-separate-artifact-kinds` | 7 | 7 | 1 | 0 | 0 | 6 | 0 |
| `2026-08-17-project-bootstrap-cli` | 5 | 0 | 0 | 4 | 0 | 1 | 0 |
| `2026-08-18-conform-generated-frontmatter-names` | 3 | 0 | 1 | 0 | 0 | 2 | 0 |
| `2026-08-18-install-the-full-workflow-set` | 4 | 0 | 1 | 1 | 0 | 2 | 0 |
| `2026-08-21-check-release-hygiene` | 3 | 0 | 1 | 1 | 0 | 2 | 0 |
| `2026-08-21-distinguish-command-names-from-skill-names` | 2 | 0 | 0 | 1 | 0 | 1 | 0 |
| `2026-08-22-accept-a-described-change` | 3 | 0 | 0 | 1 | 0 | 2 | 0 |
| `2026-08-24-pass-the-description-through` | 2 | 0 | 1 | 0 | 1 | 0 | 1 |
| `2026-08-24-report-in-the-contributors-terms` | 3 | 0 | 1 | 2 | 0 | 0 | 2 |
| `2026-08-30-finish-the-archival` | 0 (one prose paragraph) | 0 | 0 | 0 | 0 | 0 | 3 |
| `2026-08-31-sync-a-change-into-the-accepted-contract` | no section | | | | | | 0 |
| `2026-09-02-carry-task-progress-into-the-listing` | no section | | | | | | 0 |
| `2026-09-02-state-a-changes-place-in-a-sequence` | no section | | | | | | 0 |
| `2026-09-06-distinguish-a-requirement-from-an-example` | 4 | 0 | 2 | 2 | 0 | 0 | 3 |
| `2026-09-11-migrate-an-openspec-project` | 4 | 0 | 2 | 2 | 1 | 0 | 3 |
| `2026-09-15-catch-a-templates-placeholder-prose` | 4 | 4 | 1 | 0 | 0 | 3 | 0 |
| `2026-09-18-establish-a-runbook` | 5 | 1 | 2 | 0 | 1 | 2 | 0 |
| `2026-09-26-carry-a-runbook-to-another-machine` | 8 | 8 | 0 | 0 | 0 | 8 | 0 |
| `2026-09-26-log-a-runbook-run` | 6 | 0 | 2 | 1 | 0 | 3 | 0 |
| `2026-09-26-record-a-renamed-requirement` | 5 | 5 | 0 | 0 | 0 | 5 | 0 |
| `2026-09-28-confine-the-asset-check-to-the-project` | 3 | 3 | 0 | 0 | 0 | 3 | 0 |
| `2026-09-29-keep-evidence-out-of-document-discovery` | 3 | 3 | 0 | 0 | 0 | 3 | 0 |
| **Total** | **88** | **45** | **15** | **21** | **3** | **51** | **12** |

21 of the 24 designs have a Risks section. They hold 88 entries, and **45 of them are
arrow-shaped**. **37 entries carry a marker**: 15 accepted, 21 mitigated, and 3 not mitigated. One
entry carries two, as the next paragraph shows. None of the 88 entries has a heading of its own.

The 12 `### Risk:` subsections sit under `## Decisions`, in five designs. Eight open with
"Accepted", six carry a mitigated marker, and one says "unmitigated".

**An accepted risk often names a partial measure.** Four risks carry the accepted and the mitigated
markers together:

- `check-release-hygiene`, the entry at line 125: "That is the accepted cost of a stage with no
  warning level, and it is mitigated by the archive workflow's own instructions".
- `report-in-the-contributors-terms` line 107: "**Accepted, and mitigated by one sentence in the
  contract.**"
- `distinguish-a-requirement-from-an-example` lines 131-140: "**Accepted, and out of scope by
  design.**", then a paragraph that opens "The mitigation is that…". This is the risk the migration
  drafted as a unit.
- `migrate-an-openspec-project` line 466: "**Accepted, and mitigated by the order rather than by a
  check.**"

A fifth says the same thing without the word: `separate-artifact-kinds` line 100 has "Accepted
deliberately, and bounded by validation plus review at archival."

**No settlement word says "partly".** The nearest are `html-spec` line 144, "Partially mitigated by
the visible-source fallback", which is a mitigation, and the release runbook's "Partly held down and
honestly not fully", which is a mitigation paragraph.

## 5. What the migration's readers asked for

`~/projects/html-spec-migration/reading/2026-09-06-distinguish-a-requirement-from-an-example.json`,
at `.entities[2].open_questions[0]`:

> The design separates risks marked 'Accepted' from risks marked 'Mitigated'. A risk unit has a
> mitigation field but no disposition value. The first accepted risk also names a partial
> mitigation, so 'accepted' cannot be carried as 'has no mitigation paragraph'. The contract is
> silent on a risk's disposition. Recommend keeping it as prose, not mapping it to the nearest
> field.

The same file, at `.entities[2].format_notes[6]`, ends: "The Accepted/Mitigated disposition stays
in prose; see open_questions."

`…/reading/2026-08-18-conform-generated-frontmatter-names.json`, at `.entities[1].format_notes[6]`:

> Section 'Risks': three bold-led bullets, each a risk with its disposition. The first is disproved
> by task 4, and the second and third are accepted. Each becomes a risk unit. […]

`…/reading/2026-08-24-pass-the-description-through.json`, at `.entities[1].format_notes[4]`:

> […] the second Risks bullet ('An argument long enough to crowd the body. Not mitigated.') becomes
> a second risk unit. The one-line restatement is then redundant. Whether a risk is accepted or not
> mitigated has no field, so that stays prose.

The same file, at `.open_questions[1]`: "The destination has no field for a risk's status, so
'accepted, then retired for one of the two unconfirmed tools' can only be prose or a link to
pi-substitutes."

One more reader wrote the workaround down as an instruction.
`…/reading/2026-09-11-migrate-an-openspec-project.json`, at `.entities[3].format_notes[12]`: "For
'Source edited mid-migration', the mitigation field states 'Accepted, unmitigated' with its reason."

## 6. What kept the migration's risks as prose

`~/projects/html-spec-migration/brief-draft.md`, lines 282-296, verbatim:

> **Design decisions and risks: prose unless a unit fits without cutting anything.** The pilots
> found that most of this project's designs weave each decision's reasons into the same paragraphs,
> and state risks as `risk -> mitigation` lines with no heading. Keep those as the author wrote them:
> a section per decision, with the source's heading and number, and `id="hs-decision-<n>"` (or
> `hs-<slug>` where the source does not number them), so that evidence citing "decision 3" still has
> somewhere to point.
>
> Use a `decision` unit only where **all** of these hold: the source gives the decision its own
> heading; exactly one of its paragraphs states the decision and no other; and every other
> paragraph and list states only reasons or context, so it can be marked `data-field="rationale"` or
> `data-field="note"` whole. Give it `data-status="accepted"` and `data-change="added"`, keep the
> source's number in the heading, and derive its identifier as `decision-` plus the heading's name
> without its number. Use a `risk` unit only where the source gives each risk its own heading and
> states its mitigation in a paragraph of its own. Never split a paragraph or a line to make a unit,
> and never write a heading the source did not have.

**The rule has two conditions, and the missing disposition is not the main blocker.** Read against
the archive, with each headed risk named by the line of its `### Risk:` heading:

- **No heading.** All 88 Risks-section entries fail the first condition, whatever their marker. A
  disposition value does not change that. These risks stay prose after this change, as before it.
- **Headed, with the mitigation in the statement's paragraph.** Five of the 12 headed risks:
  `finish-the-archival` lines 175, 183, and 191, `distinguish-a-requirement-from-an-example` line
  142, and `migrate-an-openspec-project` line 464. Each states its measure inside the same
  paragraph as the risk, so a unit would need a paragraph split. That is also shape.
- **Headed, accepted, and stating no mitigation.** Five of the 12:
  `pass-the-description-through` line 105, `report-in-the-contributors-terms` line 117,
  `distinguish-a-requirement-from-an-example` line 148, and `migrate-an-openspec-project` lines 475
  and 482. Each has a heading, and each fails the second condition only because there is no
  mitigation to state. **For these five, the missing disposition is the blocker.** Two of them
  (lines 105 and 117) have more than one paragraph, so a unit would also mark the later paragraphs
  `data-field="note"`.
- **Headed, and marked.** One: the unit quoted in section 3.
- **Headed, and not marked although it meets both conditions on its face.** One:
  `report-in-the-contributors-terms` line 105. Its first paragraph says "Accepted, and mitigated by
  one sentence in the contract", and its second paragraph states that sentence. The draft does not
  say why it was left as prose.

Across the archive, shape keeps 93 of the 100 risk statements out of units: 88 without a heading
and 5 whose measure shares the statement's paragraph. Some list entries restate a headed
subsection, so the 100 statements count fewer distinct risks. The disposition blocks 5. **It is the
gap three readers named, and it blocks a small share of what the brief kept as prose.**
