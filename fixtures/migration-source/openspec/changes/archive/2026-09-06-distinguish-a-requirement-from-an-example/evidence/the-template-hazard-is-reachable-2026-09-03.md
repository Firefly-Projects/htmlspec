# The template hazard is reachable

Tasks 1.5 and 1.6, recorded 2026-09-03 against `main` at `80558b5`, with
`target/release/htmlspec` built from that commit.

The README case arrived from outside the project. This one the project ships itself.

## What the templates write

Read off the templates on disk rather than from the artifacts, so the list is measured:

```console
$ for t in assets/template-*.html; do
    echo "-- $(basename $t)"; grep -oE 'id="[a-z]+-example"' "$t" | sort -u
  done
```

| Template | Example identifiers it writes |
| --- | --- |
| `template-specification.html` | `decision-example`, `evidence-example`, `figure-example`, `req-example`, `risk-example`, `scenario-example`, `verify-example` |
| `template-tasks.html` | `task-example` |
| `template-proposal.html` | none |
| `template-design.html` | none |

Seven and one, matching what the proposal states. `req-example` reads exactly like a real
requirement identifier, because the naming convention for a real one is the same minus the
suffix.

## No diagnostic covers example content

All 36 diagnostics the validator can report, at `80558b5`:

```
a11y/figure-without-caption   refs/removed-target           schema/invalid
a11y/figure-without-fallback  refs/unresolved               schema/missing-reason
a11y/hidden-figure-source     review/stale-reference        schema/missing-successor
a11y/script-supplied-content  review/stale-verification     schema/prefix-mismatch
asset/duplicate               review/unverified-requirement schema/projection
asset/mismatch                safety/concealed-request      schema/unavailable
asset/unresolved              safety/event-handler          schema/weak-modal
kind/no-tasks                 safety/executable-url         viz/chart-invalid
kind/task-in-specification    safety/live-example           viz/chart-not-json
parse/recovered-error         safety/remote-chart-data      viz/diagram-empty
refs/broken-link              safety/remote-resource        viz/diagram-unrecognized
schema/duplicate-id           safety/unapproved-script      viz/schema-unavailable
```

Not one is about a template's example content. **The count is 36 rather than the 31 the proposal
states** — five were added since the proposal was written. None covers this, so the argument is
unaffected, and the number is corrected here rather than left to be re-derived.

**This list was itself wrong when first recorded, and the correction is the kind this change is
about.** It read 33: it omitted all four `a11y/*` rules, and it included `safety/remote-script`,
which is not a rule the validator ships. That identifier is constructed inside `#[cfg(test)]`
blocks in `diagnostic.rs` and `engine.rs` as a stand-in subject for the severity and engine tests,
so a grep for `RuleId::new` finds it and a reader concludes the program can report it. Both figures
were derived by grep and neither was derived carefully. Re-measured by parsing the `pub const …:
RuleId` definitions out of every non-test region of `crates/htmlspec-validate/src/`, which is the
form a shipped rule takes. The current set is 37, the one addition being `review/example-content`.

**`safety/live-example` is the near miss and deserves naming.** Anyone who greps the rule set for
"example" finds it and can conclude the gap is closed. It is not the same rule. It lives in
`crates/htmlspec-validate/src/rules/executable.rs:23` and reports a code sample carrying live
markup where inert text belongs. It has nothing to do with a template's example units.

## The gap is reachable, demonstrated

Task 1.6. A hazard nobody has demonstrated is a hazard nobody has to fix, so this is the run
rather than the argument.

```console
$ htmlspec init --tools none
$ htmlspec new change keeps-the-example
$ htmlspec new document specification htmlspec/changes/keeps-the-example/specification.html \
    --subject "A subject"

$ grep -c 'req-example' htmlspec/changes/keeps-the-example/specification.html
4

$ htmlspec validate htmlspec/changes/keeps-the-example/specification.html
  warning: review/stale-reference at verify-example: this verification references
  req-example, which this change marks added. Reconcile it when convenient; work
  continues either way

  0 errors, 2 warnings: the document is accepted

$ htmlspec archive keeps-the-example --check
  0 errors, 2 warnings: the document is accepted
note: 1 task(s) are not done or dropped: task-example
note: 1 requirement(s) this change states appear in no accepted specification: req-example.
This change has not been synced. Run `htmlspec sync --change keeps-the-example` to see what
remains, apply the change's contract, or confirm each one was withdrawn deliberately
ready with questions: this change has not been synced, and archival would report the
findings above and ask
```

**Both pass.** The document is accepted and archival reports itself ready with questions, which is
a question about the work rather than a refusal. So `req-example` reaches the accepted
specifications and becomes an obligation nobody wrote, with every check green.

## The reports that mention it are about something else

Worth recording, because three of the four notes above contain the string `example` and none of
them is this defect:

- `review/stale-reference at verify-example` — a reference whose target is marked added. A
  reconciliation prompt, not a report about example content.
- `1 task(s) are not done or dropped: task-example` — the template's example task counted as open
  work. It names the identifier for a different reason.
- `req-example appears in no accepted specification` — the sync finding. This one is the closest
  any current report comes, and it is exactly inverted: it reports that the example requirement
  has **not** reached the contract, and the remedy it names is to put it there.

That last line is the sharpest evidence in this file. The nearest thing the project has to a
warning about `req-example` is an instruction to merge it.
