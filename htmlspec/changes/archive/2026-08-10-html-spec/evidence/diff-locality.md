# Diff locality is a property of line structure, not of the formatter

**Task:** 5.8. The harness is `crates/htmlspec-format/examples/diff_locality.rs`;
reproduce with `cargo run -p htmlspec-format --example diff_locality`.

**Measured on:** 2026-08-08, against the canonical serializer as of task 5.7.

## Why this is measured and not asserted

Two questions get conflated, and separating them is the whole point of this task.

**Formatter churn** asks whether the formatter rewrites lines the content edit did
not touch. That is a conformance requirement, `tests/no_churn.rs` asserts it, and
it holds: editing a unit, inserting one, and removing one each leave every line
outside the affected unit byte-identical.

**Diff locality** asks how much of a diff a reviewer actually has to read for one
content edit. It is a consequence of how the author broke their lines, the
formatter has no say in it, and there is no threshold a document must clear. So it
is measured and recorded here rather than asserted in a test.

The design already stated this, and the specs already say the churn requirement
"does not promise a minimal Git diff". This file is the number behind that
sentence.

## What was measured

Each row is one content edit applied to a canonically formatted document, with the
document formatted again afterwards. `Changed bytes` sums every line the diff shows
on both sides; `edit bytes` is the shortest differing run once the shared prefix and
suffix are removed — the floor a diff could theoretically show. The ratio is how
much more a reviewer reads than the edit strictly required.

| Edit | Paragraph lines | Removed | Added | Changed bytes | Edit bytes | Read ratio | Total lines |
| --- | --- | --- | --- | --- | --- | --- | --- |
| One word inserted near the start of a four-sentence paragraph on one line | 1 | 1 | 1 | 225 | 7 | 32.1× | 36 |
| The same edit, with the four sentences one per line | 4 | 1 | 1 | 85 | 7 | 12.1× | 39 |
| One phrase appended at the end of the same one-line paragraph | 1 | 1 | 1 | 227 | 9 | 25.2× | 36 |
| A requirement inserted between two others | 1 | 0 | 3 | 113 | 131 | 0.9× | 40 |
| A change state added to one unit | 1 | 1 | 1 | 111 | 23 | 4.8× | 36 |
| One line changed inside a four-line preserved code excerpt | 4 | 1 | 1 | 29 | 1 | 29.0× | 18 |

## What the numbers say

**Changed-line counts do not distinguish the two layouts; changed bytes do.** Rows
one and two are the *same seven-byte edit* to the *same four sentences*, and both
show one line removed and one added. By line count the layouts are identical. By
bytes they are not: 225 against 85, a 2.6× difference, because the one-line
paragraph makes the reviewer re-read all four sentences to find the changed word.
Measuring locality in lines would have reported no difference at all, which is why
the harness measures bytes.

**The cost is the line, and the author owns the line.** The formatter preserves
author line breaks exactly, so a paragraph broken at sentence boundaries diffs at
sentence granularity and a paragraph held on one line diffs whole. Neither is the
formatter's doing. An author or an agent who wants tighter diffs writes shorter
lines, and no formatter change is required — or available.

**Position within the line does not matter, which confirms nothing is rewrapped.**
Rows one and three edit the same one-line paragraph at opposite ends and cost 225
and 227 bytes. A rewrapping formatter would have shown a cascade for the early edit
and almost nothing for the late one; the design's prototype measured exactly that
asymmetry (ten changed lines against two). Here the two are within 1% of each
other, because there is no cascade to have.

**Inserting a unit is the one local edit.** Row four is the only ratio below 1×:
the diff is three added lines and nothing removed, and it is *smaller* than the
edit's byte count because the inserted markup collapses to fewer canonical lines
than were written. Structural edits are cheap; prose edits inside long lines are
not.

**Preserved content behaves the same way, for the same reason.** Row six changes
one character inside a code excerpt and shows one 29-byte line. That is a 29×
ratio and it is fine: the author chose to break that excerpt at four lines, so a
one-character edit costs one short line rather than the whole block.

## The conclusion this change acts on

Diff locality is acceptable and it is not a formatter property. Nothing here asks
for a formatter change, and specifically nothing here argues for rewrapping:
rewrapping would trade a measured, author-controlled cost for an unmeasured
cascade plus the idempotence risk the design rejected it for.

The one thing worth carrying forward is guidance rather than machinery: **break
prose lines at sentence boundaries.** It costs an author nothing, it is not
enforced, and rows one and two show it cuts what a reviewer reads by more than
half. If a later change wants that enforced, the number to beat is in this table.

No requirement or spec changes result from this measurement.
