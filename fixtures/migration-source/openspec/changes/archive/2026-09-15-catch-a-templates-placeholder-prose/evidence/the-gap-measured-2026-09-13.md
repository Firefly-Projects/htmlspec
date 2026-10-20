# The gap, measured before anything changed

Section 1 of `tasks.md`. Every number here was produced against `main` at commit `4a89958`
(version 0.11.0), with `cargo build --release` and the resulting `target/release/htmlspec`.

## 1.1 The reproduction: a replaced tasks document, zero diagnostics

```
$ htmlspec init                                       # in an empty /tmp/phole
$ htmlspec new document tasks t.html --subject "A change that replaced its tasks"
t.html: created a tasks
$ # both example task units replaced with the author's own; masthead left alone
$ grep -c 'task-example\|task-dropped-example' t.html
0
$ grep -n 'Replace this paragraph' t.html
17:        <p>Replace this paragraph with what this work delivers. …</p>
$ htmlspec validate t.html
validating t.html
t.html
  no diagnostics: the document is accepted
$ echo $?
0
```

`--strict` reports the same, which is expected and is the second half of the finding: `strict`
never promotes a `review/*` rule, so the escalation path that catches an example *identifier* has
nothing to escalate here — there is no diagnostic to promote.

The structured report agrees, and it is what a consumer reads:

```json
{ "accepted": true, "documents": 1, "errors": 0, "warnings": 0,
  "results": [ { "path": "t.html",
    "report": { "accepted": true, "summary": { "errors": 0, "warnings": 0 }, "diagnostics": [] } } ] }
```

So the case is not merely under-reported. It is reported as clean, by both the human report and the
machine one.

## 1.2 What this repository's own history carried

Every `.html` ever added in any branch of this repository, read at the commit that added it, with
its `<header class="hs-masthead">` paragraphs normalized and tested for the placeholder's opening
words. Templates and conformance fixtures excluded, because a template carrying a placeholder is
the placeholder's definition and a fixture carrying one is deliberate.

- **104** `.html` files ever added, **12** of them documents rather than templates or fixtures.
- **1** (`README.html`) carries no `hs-masthead` header at all.
- **8** carried the author's own summary at the moment they were committed.
- **3** carried the placeholder. One of those, `assets/template.html` at `b6067bd1`, is the
  single template that predates the split into four, so it is a template rather than a document
  and does not count against the total.

**Two documents, both the same document, carried a placeholder summary into a commit:**

| Path | Commit | Date | Commit subject |
| --- | --- | --- | --- |
| `openspec/changes/migrate-an-openspec-project/evidence/the-plan-for-this-repository-2026-09-09.html` | `c04cb449` | 2026-09-09 | Migrate this repository as far as judgment allows, and repor… |
| `openspec/changes/archive/2026-09-11-migrate-an-openspec-project/evidence/the-plan-for-this-repository-2026-09-09.html` | `4a89958d` | 2026-09-13 | Bring an existing OpenSpec project into html-spec (#25) |

The second row is the first row after archival moved it. **That is the finding that matters more
than the count**: the placeholder did not merely reach a commit, it reached
`openspec/changes/archive/`, and it is still there at `HEAD`. Archival's existing refusal did not
apply, because that refusal reads a *specification's* unit identifiers and this is an evidence
document carrying no example identifier.

So the proposal's cost paragraph — "a change archived with a placeholder summary puts 'Replace this
paragraph' into the project's permanent record" — is not a hypothetical. It has already happened
once, and the record is in the repository now.

### On "several times"

The human report behind this change says "I have seen it happen several times." This scan finds the
condition surviving into a commit twice, for one document. It cannot see the cases a human caught
and fixed before committing, which is where "several" most plausibly lives: a file written, the
placeholder noticed on reading, the paragraph replaced, and one clean commit made. Those leave no
trace in `git log` by construction.

The measured count is therefore a **floor, not the number**. It is also the wrong number to argue
from, because the argument does not rest on frequency. One placeholder in the permanent archive,
committed while every check reported the document accepted, is the whole case: a condition the
project's own tools call clean is a condition that recurs at whatever rate authors are distracted.

## 1.3 The four templates, one masthead paragraph each

Confirmed by scanning `<header class="hs-masthead">` in each of `assets/template-*.html`. Every one
holds **exactly one** `<p>`, and each template's body holds more paragraphs that are not in the
masthead — 3 more in `design`, 4 in `proposal`, 3 in `tasks`, 7 in `specification`. That asymmetry
is why task 3.6 asks for a test that the scan reads the masthead only: the body paragraphs are also
text the program wrote, and a scan that swept them up would report a document for prose that is not
its summary.

The four sentences, normalized the way the projection normalizes them:

| Kind | The masthead paragraph |
| --- | --- |
| `proposal` | Replace this paragraph with the case for the change. It projects into the document's summary, so the argument the proposal rests on lives in the model rather than in markup the model cannot address. |
| `design` | Replace this paragraph with what this design settles. It projects into the document's summary, so the reasoning the change rests on lives in the model rather than in markup the model cannot address. |
| `tasks` | Replace this paragraph with what this work delivers. It projects into the document's summary, so the argument the change rests on lives in the model rather than in markup the model cannot address. |
| `specification` | Replace this paragraph with what this document is for. It projects into the document's summary, so the argument the change rests on lives in the model rather than in markup the model cannot address. |

Four distinct first sentences and a shared second one. A rule written against one wording would
cover one kind, and the kind this was reported on is `tasks`.

## 1.4 The summary is in the model, not only in the markup

`crates/htmlspec-schema/doc/projection.md` section 9.1.1:

> `summary` projects from every `p` element inside the document's `header` element, in document
> order, as normalized text.

The implementation is `header_summary` in `crates/htmlspec-parse/src/project.rs:949`, which takes
the first `header` element and collects `normalized_text_opt` of every descendant `p`.
Normalization is section 6.1, implemented at `project.rs:879` as
`raw.split_whitespace().collect::<Vec<_>>().join(" ")`.

`DocumentModel::summary` is a `Vec<String>` (`crates/htmlspec-parse/src/model.rs:105`), one entry
per paragraph.

**This change's approach is sound**: the text the rule needs is already projected, already
normalized, and already addressable in the model. The assertion task 1.4 asks for — that a freshly
created document's projected summary equals the template's paragraph, for every kind — is written
as a test rather than recorded here, because a number in a file goes stale and a test does not. It
is `the_scanned_placeholder_is_the_projected_summary_of_a_new_document` in
`crates/htmlspec-assets/tests/placeholders.rs` (task 3.4), and it passes.

Two constraints it surfaced, both recorded because they shape the implementation:

- `htmlspec-parse` depends on `htmlspec-assets`, not the reverse, so the scan cannot call the
  projection's `normalized_text` — it is private and the dependency runs the wrong way. The two
  normalizations are therefore two implementations of one rule, which is why 3.4 pins them against
  each other rather than trusting them to agree.
- `htmlspec-assets` already carries `htmlspec-parse` as a **dev**-dependency, for the
  `rendered_save` test. So the cross-crate assertion has a home with no new production dependency
  and no inverted layering.

## The archived instance, repaired

**Repaired in this change**, at the user's direction, and the reasoning is worth recording because
editing an archived evidence file is not obviously right.

The argument against editing: evidence is worth having exactly to the degree nobody edited it, and
`project-migration`'s "Evidence crosses unchanged" says so about migration. A record somebody
rewrote cannot serve the purpose a record serves.

The argument for, which is the one taken: **the placeholder is not part of the record.** It is
absence of a record — the one paragraph in that document that states nothing, generated by a script
rather than written by anyone. Every fact the plan carries lives in its 201 task units, and all 201
are untouched. Replacing a sentence that says "replace this sentence" with what the document is for
adds a statement where there was none rather than altering one that existed.

Two limits held deliberately:

- **One line changed.** The task units, the derivations, and every other paragraph are byte-identical.
  `git diff` shows a single replaced `<p>`.
- **Not reformatted.** The file was already non-canonical before this edit, and it still is.
  Reformatting an archived document to canonical form would rewrite thousands of lines to repair
  one, which is the opposite of a minimal correction. The non-canonical form is pre-existing and is
  not this change's business.

The document validates with no diagnostics afterwards, and now reports nothing under the rule this
change adds — which is the check that the repair is real rather than cosmetic.
