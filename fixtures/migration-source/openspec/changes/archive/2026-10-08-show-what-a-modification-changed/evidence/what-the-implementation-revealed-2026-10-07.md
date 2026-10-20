# What the implementation revealed

**Date:** 2026-10-07
**Task:** 7.5

## The view and `sync` never disagreed, because the scope was settled before the code

`the_view_and_sync_agree_about_what_differs` failed against the stub and passed on the first build of
the view. No case was seen in which the view showed a requirement as differing and `sync` did not, or
the reverse.

That is because the one disagreement the design held was found by reading, at task 1.1, before any
code. The design compared a requirement marked `modified` and listed every other one, while the
comparison as built compares two more kinds by content: a requirement marked `added`, and one marked
`unchanged` that holds a changed scenario, which `see-a-modification-the-merge-did-not-take`'s review
added. For both, `sync` reports content not taken and the view would have said "not compared". The
contributor settled it on 2026-10-07: the view compares what the comparison compares. The parity test
holds all five kinds `sync` reports, and its first assertion checks that the fixture still produces
all five, so it cannot pass by covering fewer.

## The comparison as built, and three corrections to the design

`what-it-builds-on-2026-10-07.md` records each. The accepted index kept every holder of an identifier
and no path, the comparison and not the index chose between two holders, and the comparison named
scenarios as strings. The fix for each was to reshape the shared code rather than to have the view
work around it:

- `accepted_requirements` keeps each holder's path, and `Holders` holds at least one by
  construction, so the comparison has no empty case to handle;
- `compare_content` is the one place a holder is chosen, and both `untaken_content` and the view
  read it;
- `ContentDifference` gives positions, and `sync` names them as before. `sync --json` and
  `archive --check --json` were byte-identical before and after on the scratch project.

## The marks reader disagreed with the projection on one invalid document

The property in `tests/marks.rs` passed on the mapping fixtures as written, because only
`change-markup` carries marks and it nests nothing wrongly. A scratch harness then ran the same checks
over the staged migration, every fixture, and `openspec/`, once as written and once with an empty
`<ins></ins>` at the start of every bare `<p>` and `<dd>`, which marks every unit without changing its
text. The second pass checked the reader's choice of paragraphs and entries against the projection on
2,299 units, and found one disagreement: in `nesting-invalid`, a requirement nested inside another and
a scenario no requirement encloses were listed by the reader, and the projection places neither
(section 4.3).

Neither could reach the view, because the view refuses a specification that does not validate. The
reader now uses the projection's own `enclosing_unit` and `encloses_requirement`, so the two cannot
disagree about which units exist, and the test's independent rule counts only the units the
projection projects.

## A word is a run between whitespace, and punctuation is part of it

Two of the view's own fixtures were wrong, not the view. `The system SHALL do x.` against `The system
SHALL do x quickly.` marks `x quickly.` on the change's side and `x.` on the accepted one, because
`x.` and `x` are different words. `git diff --word-diff` marks the same way, and the requirement
defines a word this way so the view never splits the projected text. A reviewer sees the full stop
move with the word. The one-word case is unaffected: `sync,` is a word of its own on both sides.

The same rule makes a short entry fragile. A scenario entry of three words that gains two shares too
few words to pair, so it is shown whole on each side. That is the threshold working as designed, and
the fixture now edits one word inside a longer entry.

## Task 7.1's recipe needed a step

Rebuilding the one-word case from the staged migration gave a different starting state from the one
the change was reviewed in. Today's accepted `toolchain-packaging` specification already holds the two
requirements `sync-a-change-into-the-accepted-contract` adds, because later changes merged them. So
`sync` reported one of them reached and the other as content not taken. The recipe now takes both out
of the accepted text, and the view then shows each as held by none, which is the state OpenSpec's view
was measured in.

## The accepted heading is always carried

The design had the payload's `acceptedHeading` be `null` where the two heading names were equal. A
requirement whose accepted counterpart has no heading would then read as one whose heading is the
same. The payload carries the accepted heading always, `null` only where there is none, and the human
form still shows both headings only where they differ.

## The half-words threshold on real edits

The one-word case pairs one paragraph, so it says little about the threshold. A scratch harness ran
the built view over every requirement a staged archived change marks `modified`: 34 requirements in
19 changes. For each, the accepted side was the requirement as the latest earlier archived change, by
directory name, states it, which is the nearest thing to the accepted text of its day. Each case was a
copy of the staged project with the change moved back to `changes/` and the accepted requirement
replaced, and the view ran on it with `--json` and in human form.

All 34 show as differing, as each changed something. The view made 15 pairs among them. Every pair
read was an edit a reader would want paired. The two closest to the line are both one edit inside a
paragraph with a sentence appended:

| Case | Words shared, of the change's and of the accepted paragraph | What changed |
| --- | --- | --- |
| `2026-08-17-project-bootstrap-cli`, `req-an-asset-reference-must-match-the-bytes-it-names`, paragraph 5 | 36 of 71, 36 of 37 | `present,` became `present and by the shape of the reference,`, and a sentence was appended |
| `2026-08-17-project-bootstrap-cli`, `req-a-single-command-exposes-validate-format-and-init`, paragraph 6 | 31 of 55, 31 of 32 | `Initialization` became `Creation`, and a sentence was appended |

**Two cases are the same edit and fall just below the line**, so the view shows them whole on each
side rather than paired:

- `2026-09-26-record-a-renamed-requirement`, `req-archival-assesses-the-merge-and-never-performs-it`,
  paragraph 1. The change inserted ", and for a renamed requirement its heading," into the accepted
  sentence and appended a second sentence. The accepted paragraph shares almost all its words, and
  the change's paragraph shares fewer than half of its own.
- `2026-08-18-conform-generated-frontmatter-names`,
  `req-bootstrapping-installs-agent-instructions-for-the-tools-a-project-uses`. The accepted paragraph
  "The set of supported tools SHALL be data rather than logic, so that supporting an additional tool
  is an entry in that set." is the first sentence of the change's paragraph 5, which appends one more.

A reader would pair both. The rule asks that each paragraph share at least half its words, and an
appended sentence takes the longer paragraph below half while the shorter one is wholly kept. Asking
for half of the shorter paragraph's words would pair all four of these edits. In these 34
requirements, no pair of whole paragraphs exists that the looser rule would pair wrongly: these two
are the only cases with a whole paragraph on each side. But 34 requirements written by one project
are a small sample, and the looser rule can pair a short paragraph with an unrelated long one that
shares its common words. Design decision 4 calls the threshold a judgment, and task 9.1 puts it to the
contributor. This measurement is the evidence for that decision.

The harness is `threshold.py` in the session scratchpad. It found no case in which the view failed or
disagreed with `sync`.
